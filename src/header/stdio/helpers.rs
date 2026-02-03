use alloc::boxed::Box;

use super::{
    Buffer, FILE,
    constants::{BUFSIZ, F_APP, F_NORD, F_NOWR},
};
use crate::{
    c_str::CStr,
    error::Errno,
    fs::File,
    header::{
        errno::{self, EINVAL},
        fcntl::{
            F_GETFL, F_SETFD, F_SETFL, FD_CLOEXEC, O_APPEND, O_CLOEXEC, O_CREAT, O_EXCL, O_RDONLY,
            O_RDWR, O_TRUNC, O_WRONLY, fcntl,
        },
        pthread,
    },
    io::BufWriter,
    platform::{
        self,
        types::{c_int, c_ulonglong},
    },
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
    let mut mutex_attr = pthread::RlctMutexAttr {
        ty: pthread::PTHREAD_MUTEX_RECURSIVE,
        ..Default::default()
    };
    Ok(Box::new(FILE {
        lock: pthread::RlctMutex::new(&mutex_attr).unwrap(),

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
