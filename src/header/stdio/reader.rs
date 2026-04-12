use super::{FILE, SEEK_SET, fseek_locked, ftell_locked};
use crate::{c_str::CStr, io::Read, platform::types::off_t};
use core::iter::Iterator;

pub(crate) struct BufferReader<'a> {
    buf: CStr<'a>,
}

impl<'a> From<CStr<'a>> for BufferReader<'a> {
    fn from(buff: CStr<'a>) -> Self {
        Self { buf: buff }
    }
}

impl<'a> Iterator for BufferReader<'a> {
    type Item = Result<u8, i32>;

    fn next(&mut self) -> Option<Self::Item> {
        self.buf.split_first_char().map(|(c, r)| {
            self.buf = r;
            u8::try_from(c).map_err(|_| -1)
        })
    }
}

pub(crate) struct FileReader<'a> {
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
                unsafe { fseek_locked(self.f, self.position, SEEK_SET) };
                self.position += 1;
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
    BUFFER(BufferReader<'a>),
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

impl<'a> From<CStr<'a>> for Reader<'a> {
    fn from(buff: CStr<'a>) -> Self {
        Self::BUFFER(buff.into())
    }
}
