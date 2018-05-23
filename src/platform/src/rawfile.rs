use core::ops::Deref;
use sys::{open, dup, close};

pub struct RawFile(usize);

impl RawFile {
    pub fn open<T: AsRef<[u8]>>(path: T, flags: usize) -> Result<RawFile, ()> {
        open(path, flags).map(RawFile)
    }

    pub fn dup(&self, _buf: &[u8]) -> Result<RawFile, ()> {
        match dup(self.0 as i32) {
            0 => Err(()),
            n => Ok(RawFile(n as usize))
        }
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
