//! dirent implementation following http://pubs.opengroup.org/onlinepubs/009695399/basedefs/dirent.h.html

use alloc::boxed::Box;
use core::{mem, ptr};

use c_str::CStr;
use fs::File;
use header::{errno, fcntl, stdlib, string};
use io::{Seek, SeekFrom};
use platform;
use platform::types::*;
use platform::{Pal, Sys};

const DIR_BUF_SIZE: usize = mem::size_of::<dirent>() * 3;

// No repr(C) needed, C won't see the content
pub struct DIR {
    file: File,
    buf: [c_char; DIR_BUF_SIZE],
    // index and len are specified in bytes
    index: usize,
    len: usize,

    // The last value of d_off, used by telldir
    offset: usize,
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

#[no_mangle]
pub unsafe extern "C" fn opendir(path: *const c_char) -> *mut DIR {
    let path = CStr::from_ptr(path);
    let file = match File::open(
        path,
        fcntl::O_RDONLY | fcntl::O_DIRECTORY | fcntl::O_CLOEXEC,
    ) {
        Ok(file) => file,
        Err(_) => return ptr::null_mut(),
    };

    Box::into_raw(Box::new(DIR {
        file,
        buf: [0; DIR_BUF_SIZE],
        index: 0,
        len: 0,
        offset: 0,
    }))
}

#[no_mangle]
pub unsafe extern "C" fn closedir(dir: *mut DIR) -> c_int {
    let mut dir = Box::from_raw(dir);

    let ret = Sys::close(*dir.file);

    // Reference files aren't closed when dropped
    dir.file.reference = true;

    ret
}

#[no_mangle]
pub unsafe extern "C" fn readdir(dir: *mut DIR) -> *mut dirent {
    if (*dir).index >= (*dir).len {
        let read = Sys::getdents(
            *(*dir).file,
            (*dir).buf.as_mut_ptr() as *mut dirent,
            (*dir).buf.len(),
        );
        if read <= 0 {
            if read != 0 && read != -errno::ENOENT {
                platform::errno = -read;
            }
            return ptr::null_mut();
        }

        (*dir).index = 0;
        (*dir).len = read as usize;
    }

    let ptr = (*dir).buf.as_mut_ptr().add((*dir).index) as *mut dirent;

    (*dir).offset = (*ptr).d_off as usize;
    (*dir).index += (*ptr).d_reclen as usize;
    ptr
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
pub unsafe extern "C" fn telldir(dir: *mut DIR) -> c_long {
    (*dir).offset as c_long
}
#[no_mangle]
pub unsafe extern "C" fn seekdir(dir: *mut DIR, off: c_long) {
    let _ = (*dir).file.seek(SeekFrom::Start(off as u64));
    (*dir).offset = off as usize;
    (*dir).index = 0;
    (*dir).len = 0;
}
#[no_mangle]
pub unsafe extern "C" fn rewinddir(dir: *mut DIR) {
    seekdir(dir, 0)
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

    let old_errno = platform::errno;

    let mut len: isize = 0;
    let mut cap: isize = 4;
    *namelist = platform::alloc(cap as usize * mem::size_of::<*mut dirent>()) as *mut *mut dirent;

    loop {
        platform::errno = 0;
        let entry = readdir(dir);
        if entry.is_null() {
            break;
        }

        if let Some(filter) = filter {
            if filter(entry) == 0 {
                continue;
            }
        }

        if len >= cap {
            cap *= 2;
            *namelist = platform::realloc(
                *namelist as *mut c_void,
                cap as usize * mem::size_of::<*mut dirent>(),
            ) as *mut *mut dirent;
        }

        let copy = platform::alloc(mem::size_of::<dirent>()) as *mut dirent;
        *copy = (*entry).clone();
        *(*namelist).offset(len) = copy;
        len += 1;
    }

    closedir(dir);

    if platform::errno != 0 {
        while len > 0 {
            len -= 1;
            platform::free(*(*namelist).offset(len) as *mut c_void);
        }
        platform::free(*namelist as *mut c_void);
        -1
    } else {
        platform::errno = old_errno;
        stdlib::qsort(
            *namelist as *mut c_void,
            len as size_t,
            mem::size_of::<*mut dirent>(),
            mem::transmute(compare),
        );
        len as c_int
    }
}
