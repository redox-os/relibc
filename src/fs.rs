use crate::{
    c_str::CStr,
    errno::IntoPosix,
    header::{
        fcntl::O_CREAT,
        unistd::{SEEK_CUR, SEEK_END, SEEK_SET},
    },
    io,
    platform::{types::*, Pal, Sys},
};
use core::ops::Deref;

pub struct File {
    pub fd: c_int,
    /// To avoid self referential FILE struct that needs both a reader and a writer,
    /// make "reference" files that share fd but don't close on drop.
    pub reference: bool,
}

impl File {
    pub fn new(fd: c_int) -> Self {
        Self {
            fd,
            reference: false,
        }
    }

    pub fn open(path: CStr, oflag: c_int) -> io::Result<Self> {
        Ok(Self::new(Sys::open(path, oflag, 0)?))
    }

    pub fn create(path: CStr, oflag: c_int, mode: mode_t) -> io::Result<Self> {
        Ok(Self::new(Sys::open(path, oflag | O_CREAT, mode)?))
    }

    pub fn sync_all(&self) -> io::Result<()> {
        Ok(Sys::fsync(self.fd)?)
    }

    pub fn set_len(&self, size: u64) -> io::Result<()> {
        Ok(Sys::ftruncate(self.fd, size as off_t)?)
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        Ok(Self::new(Sys::dup(self.fd)?))
    }

    /// Create a new file pointing to the same underlying descriptor. This file
    /// will know it's a "reference" and won't close the fd. It will, however,
    /// not prevent the original file from closing the fd.
    pub unsafe fn get_ref(&self) -> Self {
        Self {
            fd: self.fd,
            reference: true,
        }
    }
}

impl io::Read for &File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(Sys::read(self.fd, buf)? as usize)
    }
}

impl io::Write for &File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(Sys::write(self.fd, buf)? as usize)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Seek for &File {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let (offset, whence) = match pos {
            io::SeekFrom::Start(start) => (start as off_t, SEEK_SET),
            io::SeekFrom::Current(current) => (current as off_t, SEEK_CUR),
            io::SeekFrom::End(end) => (end as off_t, SEEK_END),
        };

        Ok(Sys::lseek(self.fd, offset, whence)? as u64)
    }
}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&mut &*self).read(buf)
    }
}

impl io::Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&mut &*self).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        (&mut &*self).flush()
    }
}

impl io::Seek for File {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        (&mut &*self).seek(pos)
    }
}

impl Deref for File {
    type Target = c_int;

    fn deref(&self) -> &Self::Target {
        &self.fd
    }
}

impl Drop for File {
    fn drop(&mut self) {
        if !self.reference {
            Sys::close(self.fd).into_posix_style();
        }
    }
}
