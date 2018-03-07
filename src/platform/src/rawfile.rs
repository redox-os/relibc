use core::ops::Deref;

pub struct RawFile(usize);

impl RawFile {
    pub fn open<T: AsRef<[u8]>>(path: T, flags: usize) -> Result<RawFile> {
        open(path, flags).map(RawFile)
    }

    pub fn dup(&self, buf: &[u8]) -> Result<RawFile> {
        dup(self.0, buf).map(RawFile)
    }
}

impl Drop for RawFile {
    fn drop(&mut self) {
        let _ = close(self.0);
    }
}

impl Deref for RawFile {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.0
    }
}
