use core::ops::Deref;
use super::{open, dup, close, types::*};

pub struct RawFile(c_int);

impl RawFile {
    pub fn open(path: *const c_char, oflag: c_int, mode: mode_t) -> Result<RawFile, ()> {
        match open(path, oflag, mode) {
            -1 => Err(()),
            n => Ok(RawFile(n))
        }
    }

    pub fn dup(&self, _buf: &[u8]) -> Result<RawFile, ()> {
        match dup(self.0) {
            -1 => Err(()),
            n => Ok(RawFile(n))
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
        let _ = close(self.0 as i32);
    }
}

impl Deref for RawFile {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.0
    }
}
