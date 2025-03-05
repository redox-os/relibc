use super::{fseek_locked, ftell_locked, FILE, SEEK_SET};
use crate::{
    io::Read,
    platform::types::{c_char, off_t},
};
use core::iter::Iterator;

struct BufferReader {
    buf: *const u8,
    position: usize,
}

impl From<*const u8> for BufferReader {
    fn from(buff: *const u8) -> Self {
        Self {
            buf: buff,
            position: 0,
        }
    }
}

impl Iterator for BufferReader {
    type Item = Result<u8, i32>;

    fn next(&mut self) -> Option<Self::Item> {
        let byte = unsafe { *self.buf.add(self.position) };
        if byte == 0 {
            None
        } else {
            self.position += 1;
            Some(Ok(byte))
        }
    }
}

struct FileReader<'a> {
    f: &'a mut FILE,
    position: off_t,
}

impl<'a> Iterator for FileReader<'a> {
    type Item = Result<u8, i32>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { fseek_locked(self.f, self.position, SEEK_SET) };
        let buf = &mut [0];
        match self.f.read(buf) {
            Ok(0) => None,
            Ok(_) => {
                self.position += 1;
                unsafe { fseek_locked(self.f, self.position - 1, SEEK_SET) };
                Some(Ok(buf[0]))
            }
            Err(_) => Some(Err(-1)),
        }
    }
}

impl<'a> From<&'a mut FILE> for FileReader<'a> {
    fn from(f: &'a mut FILE) -> FileReader<'a> {
        let position = unsafe { ftell_locked(f) } as off_t;
        FileReader { f, position }
    }
}

pub enum Reader<'a> {
    FILE(FileReader<'a>),
    BUFFER(BufferReader),
}

impl<'a> Iterator for Reader<'a> {
    type Item = Result<u8, i32>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::FILE(r) => r.next(),
            Self::BUFFER(r) => r.next(),
        }
    }
}

impl<'a> From<&'a mut FILE> for Reader<'a> {
    fn from(f: &'a mut FILE) -> Self {
        Self::FILE(f.into())
    }
}

impl<'a> From<*const c_char> for Reader<'a> {
    fn from(buff: *const c_char) -> Self {
        Self::BUFFER((buff as *const u8).into())
    }
}
