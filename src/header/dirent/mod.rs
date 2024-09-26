//! dirent implementation following http://pubs.opengroup.org/onlinepubs/009695399/basedefs/dirent.h.html

use alloc::{boxed::Box, vec::Vec};
use core::{mem, ptr};

use crate::{
    c_str::CStr,
    c_vec::CVec,
    error::{Errno, ResultExt, ResultExtPtrMut},
    fs::File,
    header::{fcntl, stdlib, string},
    platform::{self, types::*, Pal, Sys},
};

use super::errno::{EINVAL, EIO, ENOMEM};

const INITIAL_BUFSIZE: usize = 512;

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

#[no_mangle]
pub unsafe extern "C" fn opendir(path: *const c_char) -> *mut DIR {
    let path = CStr::from_ptr(path);

    DIR::new(path).or_errno_null_mut()
}

#[no_mangle]
pub extern "C" fn closedir(dir: Box<DIR>) -> c_int {
    dir.close().map(|()| 0).or_minus_one_errno()
}

#[no_mangle]
pub extern "C" fn dirfd(dir: &mut DIR) -> c_int {
    *dir.file
}

#[no_mangle]
pub extern "C" fn readdir(dir: &mut DIR) -> *mut dirent {
    dir.next_dirent().or_errno_null_mut()
}

// #[no_mangle]
pub extern "C" fn readdir_r(
    _dir: *mut DIR,
    _entry: *mut dirent,
    _result: *mut *mut dirent,
) -> *mut dirent {
    unimplemented!(); // plus, deprecated
}

#[no_mangle]
pub extern "C" fn telldir(dir: &mut DIR) -> c_long {
    dir.opaque_offset as c_long
}
#[no_mangle]
pub extern "C" fn seekdir(dir: &mut DIR, off: c_long) {
    dir.seek(
        off.try_into()
            .expect("off must come from telldir, thus never negative"),
    );
}
#[no_mangle]
pub extern "C" fn rewinddir(dir: &mut DIR) {
    dir.rewind();
}

#[no_mangle]
pub unsafe extern "C" fn alphasort(first: *mut *const dirent, second: *mut *const dirent) -> c_int {
    string::strcoll((**first).d_name.as_ptr(), (**second).d_name.as_ptr())
}

#[no_mangle]
pub unsafe extern "C" fn scandir(
    dirp: *const c_char,
    namelist: *mut *mut *mut dirent,
    filter: Option<extern "C" fn(_: *const dirent) -> c_int>,
    compare: Option<extern "C" fn(_: *mut *const dirent, _: *mut *const dirent) -> c_int>,
) -> c_int {
    let dir = opendir(dirp);
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
        let entry: *mut dirent = readdir(&mut *dir);
        if entry.is_null() {
            break;
        }

        if let Some(filter) = filter {
            if filter(entry) == 0 {
                continue;
            }
        }

        let copy = platform::alloc(mem::size_of::<dirent>()) as *mut dirent;
        if copy.is_null() {
            break;
        }
        ptr::write(copy, (*entry).clone());
        if let Err(_) = vec.push(copy) {
            break;
        }
    }

    closedir(Box::from_raw(dir));

    let len = vec.len();
    if let Err(_) = vec.shrink_to_fit() {
        return -1;
    }

    if platform::ERRNO.get() != 0 {
        for ptr in &mut vec {
            platform::free(*ptr as *mut c_void);
        }
        -1
    } else {
        *namelist = vec.leak();

        platform::ERRNO.set(old_errno);
        stdlib::qsort(
            *namelist as *mut c_void,
            len as size_t,
            mem::size_of::<*mut dirent>(),
            mem::transmute(compare),
        );

        len as c_int
    }
}
