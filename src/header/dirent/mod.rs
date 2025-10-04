//! `dirent.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/dirent.h.html>.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::{
    header::unistd::{SEEK_CUR, SEEK_SET},
    platform::types::{c_int, c_void, off_t, size_t, ssize_t},
};
use alloc::{boxed::Box, vec::Vec};
use core::{mem, ptr, slice};

use crate::{
    c_str::CStr,
    c_vec::CVec,
    error::{Errno, ResultExt, ResultExtPtrMut},
    fs::File,
    header::{fcntl, stdlib, string},
    out::Out,
    platform::{self, Pal, Sys, types::*},
};

use super::{
    errno::{self, EINVAL, EIO, ENOMEM, ENOTDIR},
    sys_stat,
};

const INITIAL_BUFSIZE: usize = 512;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/dirent.h.html>.
// No repr(C) needed, as this is a completely opaque struct. Being accessed as a pointer, in C it's
// just defined as `struct DIR`.
pub struct DIR {
    file: File,
    buf: Vec<u8>,
    buf_offset: usize,

    // The last value of d_off, used by telldir
    opaque_offset: u64,
}
impl DIR {
    pub fn new(path: CStr) -> Result<Box<Self>, Errno> {
        Ok(Box::new(Self {
            file: File::open(
                path,
                fcntl::O_RDONLY | fcntl::O_DIRECTORY | fcntl::O_CLOEXEC,
            )?,
            buf: Vec::with_capacity(INITIAL_BUFSIZE),
            buf_offset: 0,
            opaque_offset: 0,
        }))
    }
    pub fn from_fd(fd: c_int) -> Result<Box<Self>, Errno> {
        let mut stat = sys_stat::stat::default();
        unsafe {
            Sys::fstat(fd, Out::from_mut(&mut stat))?;
        }
        if (stat.st_mode & sys_stat::S_IFMT) != sys_stat::S_IFDIR {
            return Err(Errno(ENOTDIR));
        }
        Sys::fcntl(fd, fcntl::F_SETFD, fcntl::FD_CLOEXEC as _)?;

        // Take ownership now but not earlier so we don't close the fd on error.
        let file = File::new(fd);
        Ok(Self {
            file,
            buf: Vec::with_capacity(INITIAL_BUFSIZE),
            buf_offset: 0,
            opaque_offset: 0,
        }
        .into())
    }
    fn next_dirent(&mut self) -> Result<*mut dirent, Errno> {
        let mut this_dent = self.buf.get(self.buf_offset..).ok_or(Errno(EIO))?;
        if this_dent.is_empty() {
            let size = loop {
                self.buf.resize(self.buf.capacity(), 0_u8);
                // TODO: uninitialized memory?
                match Sys::getdents(*self.file, &mut self.buf, self.opaque_offset) {
                    Ok(size) => break size,
                    Err(Errno(EINVAL)) => {
                        self.buf
                            .try_reserve_exact(self.buf.len())
                            .map_err(|_| Errno(ENOMEM))?;
                        continue;
                    }
                    Err(Errno(other)) => return Err(Errno(other)),
                }
            };
            self.buf.truncate(size);
            self.buf_offset = 0;

            if size == 0 {
                return Ok(core::ptr::null_mut());
            }
            this_dent = &self.buf;
        }
        let (this_reclen, this_next_opaque) =
            unsafe { Sys::dent_reclen_offset(this_dent, self.buf_offset).ok_or(Errno(EIO))? };

        //println!("CDENT {} {}+{}", self.opaque_offset, self.buf_offset, this_reclen);

        let next_off = self
            .buf_offset
            .checked_add(usize::from(this_reclen))
            .ok_or(Errno(EIO))?;
        if next_off > self.buf.len() {
            return Err(Errno(EIO));
        }
        if this_dent.len() < usize::from(this_reclen) {
            // Don't want memory corruption if a scheme is adversarial.
            return Err(Errno(EIO));
        }
        let dent_ptr = this_dent.as_ptr() as *mut dirent;

        self.opaque_offset = this_next_opaque;
        self.buf_offset = next_off;
        Ok(dent_ptr)
    }
    fn seek(&mut self, off: u64) {
        let Ok(_) = Sys::dir_seek(*self.file, off) else {
            return;
        };
        self.buf.clear();
        self.buf_offset = 0;
        self.opaque_offset = off;
    }
    fn rewind(&mut self) {
        self.opaque_offset = 0;
        let Ok(_) = Sys::dir_seek(*self.file, 0) else {
            return;
        };
        self.buf.clear();
        self.buf_offset = 0;
        self.opaque_offset = 0;
    }
    fn close(mut self) -> Result<(), Errno> {
        // Reference files aren't closed when dropped
        self.file.reference = true;

        // TODO: result
        Sys::close(*self.file)
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/dirent.h.html>.
#[repr(C)]
#[derive(Clone)]
pub struct dirent {
    pub d_ino: ino_t,
    pub d_off: off_t,
    pub d_reclen: c_ushort,
    pub d_type: c_uchar,
    pub d_name: [c_char; 256],
}

#[cfg(target_os = "redox")]
const _: () = {
    use core::mem::{offset_of, size_of};
    use syscall::dirent::DirentHeader;

    if offset_of!(dirent, d_ino) != offset_of!(DirentHeader, inode) {
        panic!("struct dirent layout mismatch (inode)");
    }
    if offset_of!(dirent, d_off) != offset_of!(DirentHeader, next_opaque_id) {
        panic!("struct dirent layout mismatch (inode)");
    }
    if offset_of!(dirent, d_reclen) != offset_of!(DirentHeader, record_len) {
        panic!("struct dirent layout mismatch (len)");
    }
    if offset_of!(dirent, d_type) != offset_of!(DirentHeader, kind) {
        panic!("struct dirent layout mismatch (kind)");
    }
    if offset_of!(dirent, d_name) != size_of::<DirentHeader>() {
        panic!("struct dirent layout mismatch (name)");
    }
};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/alphasort.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn alphasort(first: *mut *const dirent, second: *mut *const dirent) -> c_int {
    unsafe { string::strcoll((**first).d_name.as_ptr(), (**second).d_name.as_ptr()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/closedir.html>.
#[unsafe(no_mangle)]
pub extern "C" fn closedir(dir: Box<DIR>) -> c_int {
    dir.close().map(|()| 0).or_minus_one_errno()
}

/// See <https://man.freebsd.org/cgi/man.cgi?query=fdopendir&sektion=3>
///
/// FreeBSD extension that transfers ownership of the directory file descriptor to the user.
///
/// It doesn't matter if DIR was opened with [`opendir`] or [`fdopendir`].
#[unsafe(no_mangle)]
pub extern "C" fn fdclosedir(dir: Box<DIR>) -> c_int {
    let mut file = dir.file;
    file.reference = true;

    *file
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/dirfd.html>.
#[unsafe(no_mangle)]
pub extern "C" fn dirfd(dir: &mut DIR) -> c_int {
    *dir.file
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fdopendir.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn opendir(path: *const c_char) -> *mut DIR {
    let path = unsafe { CStr::from_ptr(path) };

    DIR::new(path).or_errno_null_mut()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fdopendir.html>.
#[unsafe(no_mangle)]
pub extern "C" fn fdopendir(fd: c_int) -> *mut DIR {
    DIR::from_fd(fd).or_errno_null_mut()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_getdents.html>.
#[unsafe(no_mangle)]
pub extern "C" fn posix_getdents(
    fildes: c_int,
    buf: *mut c_void,
    nbyte: size_t,
    _flags: c_int,
) -> ssize_t {
    let slice = unsafe { slice::from_raw_parts_mut(buf as *mut u8, nbyte) };

    Sys::posix_getdents(fildes, slice)
        .map(|s| s as ssize_t)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/readdir.html>.
#[unsafe(no_mangle)]
pub extern "C" fn readdir(dir: &mut DIR) -> *mut dirent {
    dir.next_dirent().or_errno_null_mut()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/readdir.html>.
///
/// # Deprecation
/// The `readdir_r()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 8.
#[deprecated]
// #[unsafe(no_mangle)]
pub extern "C" fn readdir_r(
    _dir: *mut DIR,
    _entry: *mut dirent,
    _result: *mut *mut dirent,
) -> *mut dirent {
    unimplemented!(); // plus, deprecated
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/rewinddir.html>.
#[unsafe(no_mangle)]
pub extern "C" fn rewinddir(dir: &mut DIR) {
    dir.rewind();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/alphasort.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scandir(
    dirp: *const c_char,
    namelist: *mut *mut *mut dirent,
    filter: Option<extern "C" fn(_: *const dirent) -> c_int>,
    compare: Option<extern "C" fn(_: *mut *const dirent, _: *mut *const dirent) -> c_int>,
) -> c_int {
    let dir = unsafe { opendir(dirp) };
    if dir.is_null() {
        return -1;
    }

    let mut vec = match CVec::with_capacity(4) {
        Ok(vec) => vec,
        Err(err) => return -1,
    };

    let old_errno = platform::ERRNO.get();
    platform::ERRNO.set(0);

    loop {
        let entry: *mut dirent = readdir(unsafe { &mut *dir });
        if entry.is_null() {
            break;
        }

        if let Some(filter) = filter {
            if filter(entry) == 0 {
                continue;
            }
        }

        let copy = unsafe { platform::alloc(mem::size_of::<dirent>()) } as *mut dirent;
        if copy.is_null() {
            break;
        }
        unsafe { ptr::write(copy, (*entry).clone()) };
        if let Err(_) = vec.push(copy) {
            break;
        }
    }

    closedir(unsafe { Box::from_raw(dir) });

    let len = vec.len();
    if let Err(_) = vec.shrink_to_fit() {
        return -1;
    }

    if platform::ERRNO.get() != 0 {
        for ptr in &mut vec {
            unsafe { platform::free(*ptr as *mut c_void) };
        }
        -1
    } else {
        unsafe {
            // Empty CVecs use a dangling pointer which cannot be freed, return null instead
            if vec.is_empty() {
                *namelist = ptr::null_mut();
            } else {
                *namelist = vec.leak();
            }
        }

        platform::ERRNO.set(old_errno);
        unsafe {
            stdlib::qsort(
                *namelist as *mut c_void,
                len as size_t,
                mem::size_of::<*mut dirent>(),
                mem::transmute(compare),
            )
        };

        len as c_int
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/seekdir.html>.
#[unsafe(no_mangle)]
pub extern "C" fn seekdir(dir: &mut DIR, off: c_long) {
    dir.seek(
        off.try_into()
            .expect("off must come from telldir, thus never negative"),
    );
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/telldir.html>.
#[unsafe(no_mangle)]
pub extern "C" fn telldir(dir: &mut DIR) -> c_long {
    dir.opaque_offset as c_long
}
