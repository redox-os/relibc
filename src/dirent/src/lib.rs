//! dirent implementation following http://pubs.opengroup.org/onlinepubs/009695399/basedefs/dirent.h.html

#![no_std]
#![feature(alloc)]

extern crate alloc;
extern crate errno;
extern crate fcntl;
extern crate platform;
extern crate stdio;
extern crate unistd;

use alloc::boxed::Box;
use core::{mem, ptr};
use platform::types::*;

const DIR_BUF_SIZE: usize = mem::size_of::<dirent>() * 3;

// No repr(C) needed, C won't see the content
// TODO: ***THREAD SAFETY***
pub struct DIR {
    fd: c_int,
    buf: [c_char; DIR_BUF_SIZE],
    // index & len are specified in bytes
    index: usize,
    len: usize,

    // Offset is like the total index, never erased It is used as an
    // alternative to dirent's d_off, but works on redox too.
    offset: usize,
}

#[repr(C)]
pub struct dirent {
    pub d_ino: ino_t,
    pub d_off: off_t,
    pub d_reclen: c_ushort,
    pub d_type: c_uchar,
    pub d_name: [c_char; 256],
}

#[no_mangle]
pub extern "C" fn opendir(path: *const c_char) -> *mut DIR {
    let fd = platform::open(
        path,
        fcntl::O_RDONLY | fcntl::O_DIRECTORY | fcntl::O_CLOEXEC,
        0,
    );

    if fd < 0 {
        return ptr::null_mut();
    }

    Box::into_raw(Box::new(DIR {
        fd,
        buf: [0; DIR_BUF_SIZE],
        index: 0,
        len: 0,
        offset: 0,
    }))
}

#[no_mangle]
pub unsafe extern "C" fn closedir(dir: *mut DIR) -> c_int {
    let ret = platform::close((*dir).fd);
    Box::from_raw(dir);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn readdir(dir: *mut DIR) -> *mut dirent {
    if (*dir).index >= (*dir).len {
        let read = platform::getdents(
            (*dir).fd,
            (*dir).buf.as_mut_ptr() as *mut platform::types::dirent,
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

    let ptr = (*dir).buf.as_mut_ptr().offset((*dir).index as isize) as *mut dirent;

    #[cfg(target_os = "redox")]
    {
        if (*dir).index != 0 || (*dir).offset != 0 {
            // This should happen every time but the first, making the offset
            // point to the current element and not the next
            (*dir).offset += mem::size_of::<dirent>();
        }
        (*ptr).d_off = (*dir).offset as off_t;
    }
    #[cfg(not(target_os = "redox"))]
    {
        (*dir).offset = (*ptr).d_off as usize;
    }

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
    unistd::lseek((*dir).fd, off, unistd::SEEK_SET);
    (*dir).offset = off as usize;
    (*dir).index = 0;
    (*dir).len = 0;
}
#[no_mangle]
pub unsafe extern "C" fn rewinddir(dir: *mut DIR) {
    seekdir(dir, 0)
}
