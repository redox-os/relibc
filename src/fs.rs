use c_str::CStr;
use header::fcntl::O_CREAT;
use header::unistd::{SEEK_SET, SEEK_CUR, SEEK_END};
use io;
use platform;
use platform::{Pal, Sys};
use platform::types::*;

fn last_os_error() -> io::Error {
    let errno = unsafe { platform::errno };
    io::Error::from_raw_os_error(errno)
}

pub struct File(c_int);

impl File {
    pub fn open(path: &CStr, oflag: c_int) -> io::Result<Self> {
        match Sys::open(path, oflag, 0) {
            -1 => Err(last_os_error()),
            ok => Ok(File(ok)),
        }
    }

    pub fn create(path: &CStr, oflag: c_int, mode: mode_t) -> io::Result<Self> {
        match Sys::open(path, oflag | O_CREAT, mode) {
            -1 => Err(last_os_error()),
            ok => Ok(File(ok)),
        }
    }

    pub fn sync_all(&self) -> io::Result<()> {
        match Sys::fsync(self.0) {
            -1 => Err(last_os_error()),
            _ok => Ok(()),
        }
    }

    pub fn set_len(&self, size: u64) -> io::Result<()> {
        match Sys::ftruncate(self.0, size as off_t) {
            -1 => Err(last_os_error()),
            _ok => Ok(()),
        }
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        match Sys::dup(self.0) {
            -1 => Err(last_os_error()),
            ok => Ok(File(ok)),
        }
    }
}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match Sys::read(self.0, buf) {
            -1 => Err(last_os_error()),
            ok => Ok(ok as usize),
        }
    }
}

impl io::Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match Sys::write(self.0, buf) {
            -1 => Err(last_os_error()),
            ok => Ok(ok as usize),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Seek for File {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let (offset, whence) = match pos {
            io::SeekFrom::Start(start) => (start as off_t, SEEK_SET),
            io::SeekFrom::Current(current) => (current as off_t, SEEK_CUR),
            io::SeekFrom::End(end) => (end as off_t, SEEK_END),
        };

        match Sys::lseek(self.0, offset, whence) {
            -1 => Err(last_os_error()),
            ok => Ok(ok as u64),
        }
    }
}
