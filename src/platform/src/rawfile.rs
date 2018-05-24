use core::ops::Deref;
use super::{open, dup, close};

pub struct RawFile(usize);

impl RawFile {
    pub fn open<T: AsRef<[u8]>>(path: T, flags: usize, mode: usize) -> Result<RawFile, ()> {
        match open(path.as_ref()[0] as *const i8, flags as i32, mode as u16) {
            -1 => Err(()),
            n => Ok(RawFile(n as usize))
        }
    }

    pub fn dup(&self, _buf: &[u8]) -> Result<RawFile, ()> {
        match dup(self.0 as i32) {
            -1 => Err(()),
            n => Ok(RawFile(n as usize))
        }
    }

    pub fn as_raw_fd(&self) -> usize {
        self.0
    }

    pub fn into_raw_fd(self) -> usize {
        self.0
    }

    pub fn from_raw_fd(fd: usize) -> Self {
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
