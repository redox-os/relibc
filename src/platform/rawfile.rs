use alloc::Vec;
use core::ops::Deref;

use super::{types::*, Pal, Sys};
use c_str::CStr;

pub struct RawFile(c_int);

impl RawFile {
    pub fn open(path: &CStr, oflag: c_int, mode: mode_t) -> Result<RawFile, ()> {
        match Sys::open(path, oflag, mode) {
            -1 => Err(()),
            n => Ok(RawFile(n)),
        }
    }

    pub fn dup(&self) -> Result<RawFile, ()> {
        match Sys::dup(self.0) {
            -1 => Err(()),
            n => Ok(RawFile(n)),
        }
    }

    pub fn as_raw_fd(&self) -> c_int {
        self.0
    }

    pub fn into_raw_fd(self) -> c_int {
        self.0
    }

    pub fn from_raw_fd(fd: c_int) -> Self {
        RawFile(fd)
    }
}

impl Drop for RawFile {
    fn drop(&mut self) {
        let _ = Sys::close(self.0);
    }
}

impl Deref for RawFile {
    type Target = c_int;

    fn deref(&self) -> &c_int {
        &self.0
    }
}

pub fn file_read_all(path: &CStr) -> Result<Vec<u8>, ()> {
    let file = RawFile::open(path, 0, 0o644)?;

    let mut buf = Vec::new();
    let mut len = 0;

    loop {
        if len >= buf.capacity() {
            buf.reserve(32);

            unsafe {
                let capacity = buf.capacity();
                buf.set_len(capacity);
            }
        }

        let read = Sys::read(*file, &mut buf[len..]);

        len += read as usize;

        if read == 0 {
            unsafe { buf.set_len(len); }
            return Ok(buf);
        }
        if read < 0 {
            unsafe { buf.set_len(len); }
            return Err(());
        }
    }
}
