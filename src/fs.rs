use crate::{
    c_str::CStr,
    error::{Errno, ResultExt},
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

    pub fn open(path: CStr, oflag: c_int) -> Result<Self, Errno> {
        Sys::open(path, oflag, 0).map(Self::new).map_err(Errno::sync)
    }

    pub fn create(path: CStr, oflag: c_int, mode: mode_t) -> Result<Self, Errno> {
        Sys::open(path, oflag | O_CREAT, mode).map(Self::new).map_err(Errno::sync)
    }

    pub fn sync_all(&self) -> Result<(), Errno> {
        Sys::fsync(self.fd).map_err(Errno::sync)
    }

    pub fn set_len(&self, size: u64) -> Result<(), Errno> {
        Sys::ftruncate(self.fd, size as off_t).map_err(Errno::sync)
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        match Sys::dup(self.fd) {
            -1 => Err(io::last_os_error()),
            ok => Ok(Self::new(ok)),
        }
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
        match Sys::read(self.fd, buf).or_minus_one_errno() /* TODO */ {
            -1 => Err(io::last_os_error()),
            ok => Ok(ok as usize),
        }
    }
}

impl io::Write for &File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match Sys::write(self.fd, buf).or_minus_one_errno() {
            -1 => Err(io::last_os_error()),
            ok => Ok(ok as usize),
        }
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

        match Sys::lseek(self.fd, offset, whence) {
            -1 => Err(io::last_os_error()),
            ok => Ok(ok as u64),
        }
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
            let _ = Sys::close(self.fd);
        }
    }
}
