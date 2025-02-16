use super::{fseek_locked, ftell_locked, FILE, SEEK_SET};
use crate::{
    io::Read,
    platform::types::{c_char, off_t},
};

struct LookAheadBuffer {
    buf: *const u8,
    pos: isize,
    look_ahead: isize,
}

impl LookAheadBuffer {
    fn look_ahead(&mut self) -> Result<Option<u8>, i32> {
        let byte = unsafe { *self.buf.offset(self.look_ahead) };
        if byte == 0 {
            Ok(None)
        } else {
            self.look_ahead += 1;
            unsafe { *self.buf.sub((self.look_ahead - self.pos) as usize) };
            Ok(Some(byte))
        }
    }

    fn commit(&mut self) {
        self.pos = self.look_ahead;
        unsafe { *self.buf.offset(self.pos) };
    }

    fn current(&mut self) -> Result<Option<u8>, i32> {
        let ch = unsafe { self.buf.read() };
        if ch == 0 {
            Ok(None)
        } else {
            Ok(Some(ch))
        }
    }
}

impl From<*const u8> for LookAheadBuffer {
    fn from(buff: *const u8) -> LookAheadBuffer {
        LookAheadBuffer {
            buf: buff,
            pos: 0,
            look_ahead: 0,
        }
    }
}

struct LookAheadFile<'a> {
    f: &'a mut FILE,
    look_ahead: i64,
}

impl<'a> LookAheadFile<'a> {
    fn look_ahead(&mut self) -> Result<Option<u8>, i32> {
        let seek = unsafe { ftell_locked(self.f) };
        unsafe { fseek_locked(self.f, self.look_ahead as off_t, SEEK_SET) };
        let ret = self.current();
        unsafe { fseek_locked(self.f, seek, SEEK_SET) };
        self.look_ahead += 1;
        ret
    }

    fn commit(&mut self) {
        unsafe { fseek_locked(self.f, self.look_ahead as off_t, SEEK_SET) };
    }

    fn current(&mut self) -> Result<Option<u8>, i32> {
        let buf = &mut [0];
        match self.f.read(buf) {
            Ok(0) => Ok(None),
            Ok(_) => Ok(Some(buf[0])),
            Err(_) => Err(-1),
        }
    }
}

impl<'a> From<&'a mut FILE> for LookAheadFile<'a> {
    fn from(f: &'a mut FILE) -> LookAheadFile<'a> {
        let look_ahead = unsafe { ftell_locked(f) } as i64;
        LookAheadFile { f, look_ahead }
    }
}

enum LookAheadReaderEnum<'a> {
    FILE(LookAheadFile<'a>),
    // (buffer, location)
    BUFFER(LookAheadBuffer),
}

pub struct LookAheadReader<'a>(LookAheadReaderEnum<'a>);

impl<'a> LookAheadReader<'a> {
    pub fn lookahead1(&mut self) -> Result<Option<u8>, i32> {
        match &mut self.0 {
            LookAheadReaderEnum::FILE(f) => f.look_ahead(),
            LookAheadReaderEnum::BUFFER(b) => b.look_ahead(),
        }
    }

    pub fn commit(&mut self) {
        match &mut self.0 {
            LookAheadReaderEnum::FILE(f) => f.commit(),
            LookAheadReaderEnum::BUFFER(b) => b.commit(),
        }
    }

    pub fn current(&mut self) -> Result<Option<u8>, i32> {
        match &mut self.0 {
            LookAheadReaderEnum::FILE(f) => f.current(),
            LookAheadReaderEnum::BUFFER(b) => b.current(),
        }
    }
}

impl<'a> From<&'a mut FILE> for LookAheadReader<'a> {
    fn from(f: &'a mut FILE) -> LookAheadReader {
        LookAheadReader(LookAheadReaderEnum::FILE(f.into()))
    }
}

impl<'a> From<*const c_char> for LookAheadReader<'a> {
    fn from(buff: *const c_char) -> LookAheadReader<'a> {
        LookAheadReader(LookAheadReaderEnum::BUFFER((buff as *const u8).into()))
    }
}
