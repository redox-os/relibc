use alloc::boxed::Box;

use super::{constants::*, Buffer, FILE};
use crate::{
    c_str::CStr,
    error::Errno,
    fs::File,
    header::{
        errno::{self, EINVAL},
        fcntl::*,
    },
    io::BufWriter,
    platform::{self, types::*},
    sync::Mutex,
};
use alloc::vec::Vec;

/// Parse mode flags as a string and output a mode flags integer
pub fn parse_mode_flags(mode_str: CStr) -> i32 {
    let mut flags = if mode_str.contains(b'+') {
        O_RDWR
    } else if mode_str.first() == b'r' {
        O_RDONLY
    } else {
        O_WRONLY
    };
    if mode_str.contains(b'x') {
        flags |= O_EXCL;
    }
    if mode_str.contains(b'e') {
        flags |= O_CLOEXEC;
    }
    if mode_str.first() != b'r' {
        flags |= O_CREAT;
    }
    if mode_str.first() == b'w' {
        flags |= O_TRUNC;
    } else if mode_str.first() == b'a' {
        flags |= O_APPEND;
    }

    flags
}

/// Open a file with the file descriptor `fd` in the mode `mode`
pub fn _fdopen(fd: c_int, mode: CStr) -> Result<Box<FILE>, Errno> {
    if mode.first() != b'r' && mode.first() != b'w' && mode.first() != b'a' {
        return Err(Errno(EINVAL));
    }

    let mut flags = 0;
    if !mode.contains(b'+') {
        flags |= if mode.first() == b'r' { F_NOWR } else { F_NORD };
    }

    if mode.contains(b'e') {
        unsafe {
            fcntl(fd, F_SETFD, FD_CLOEXEC as c_ulonglong);
        }
    }

    if mode.first() == b'a' {
        let f = unsafe { fcntl(fd, F_GETFL, 0) };
        if (f & O_APPEND) == 0 {
            unsafe { fcntl(fd, F_SETFL, (f | O_APPEND) as c_ulonglong) };
        }
        flags |= F_APP;
    }

    let file = File::new(fd);
    let writer = Box::new(BufWriter::new(unsafe { file.get_ref() }));

    Ok(Box::new(FILE {
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
    }))
}
