use syscall;
use core::ops::Deref;

pub struct RawFile(usize);

impl RawFile {
    pub fn open<T: AsRef<[u8]>>(path: T, flags: usize) -> syscall::Result<RawFile> {
        syscall::open(path, flags).map(RawFile)
    }

    pub fn dup(&self, buf: &[u8]) -> syscall::Result<RawFile> {
        syscall::dup(self.0, buf).map(RawFile)
    }
}

impl Drop for RawFile {
    fn drop(&mut self) {
        let _ = syscall::close(self.0);
    }
}

impl Deref for RawFile {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.0
    }
}
