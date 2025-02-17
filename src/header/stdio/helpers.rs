use alloc::boxed::Box;

use super::{constants::*, Buffer, FILE};
use crate::{
    fs::File,
    header::{errno, fcntl::*, string::strchr},
    io::BufWriter,
    platform::{self, types::*},
    sync::Mutex,
};
use alloc::vec::Vec;

/// Parse mode flags as a string and output a mode flags integer
pub unsafe fn parse_mode_flags(mode_str: *const c_char) -> i32 {
    let mut flags = if !unsafe { strchr(mode_str, b'+' as i32) }.is_null() {
        O_RDWR
    } else if unsafe { *mode_str } == b'r' as i8 {
        O_RDONLY
    } else {
        O_WRONLY
    };
    if !unsafe { strchr(mode_str, b'x' as i32) }.is_null() {
        flags |= O_EXCL;
    }
    if !unsafe { strchr(mode_str, b'e' as i32) }.is_null() {
        flags |= O_CLOEXEC;
    }
    if unsafe { *mode_str } != b'r' as i8 {
        flags |= O_CREAT;
    }
    if unsafe { *mode_str } == b'w' as i8 {
        flags |= O_TRUNC;
    } else if unsafe { *mode_str } == b'a' as i8 {
        flags |= O_APPEND;
    }

    flags
}

/// Open a file with the file descriptor `fd` in the mode `mode`
pub unsafe fn _fdopen(fd: c_int, mode: *const c_char) -> Option<*mut FILE> {
    if unsafe { *mode } != b'r' as i8
        && unsafe { *mode } != b'w' as i8
        && unsafe { *mode } != b'a' as i8
    {
        platform::ERRNO.set(errno::EINVAL);
        return None;
    }

    let mut flags = 0;
    if unsafe { strchr(mode, b'+' as i32) }.is_null() {
        flags |= if unsafe { *mode } == b'r' as i8 {
            F_NOWR
        } else {
            F_NORD
        };
    }

    if !unsafe { strchr(mode, b'e' as i32) }.is_null() {
        unsafe { fcntl(fd, F_SETFD, FD_CLOEXEC as c_ulonglong) };
    }

    if unsafe { *mode } == 'a' as i8 {
        let f = unsafe { fcntl(fd, F_GETFL, 0) };
        if (f & O_APPEND) == 0 {
            unsafe { fcntl(fd, F_SETFL, (f | O_APPEND) as c_ulonglong) };
        }
        flags |= F_APP;
    }

    let file = File::new(fd);
    let writer = Box::new(BufWriter::new(unsafe { file.get_ref() }));

    Some(Box::into_raw(Box::new(FILE {
        lock: Mutex::new(()),

        file,
        flags,
        read_buf: Buffer::Owned(vec![0; BUFSIZ as usize]),
        read_pos: 0,
        read_size: 0,
        unget: Vec::new(),
        writer,

        pid: None,

        orientation: 0,
    })))
}
