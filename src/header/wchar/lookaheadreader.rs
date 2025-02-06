use super::{fseek_locked, ftell_locked, FILE, SEEK_SET};
use crate::{
    io::Read,
    platform::types::{off_t, wint_t},
};
struct LookAheadBuffer {
    buf: *const wint_t,
    pos: isize,
    look_ahead: isize,
}

impl LookAheadBuffer {
    fn look_ahead(&mut self) -> Result<Option<wint_t>, i32> {
        let wchar = unsafe { *self.buf.offset(self.look_ahead) };
        if wchar == 0 {
            Ok(None)
        } else {
            self.look_ahead += 1;
            Ok(Some(wchar))
        }
    }

    fn commit(&mut self) {
        self.pos = self.look_ahead;
    }
}

impl From<*const wint_t> for LookAheadBuffer {
    fn from(buff: *const wint_t) -> LookAheadBuffer {
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
    fn look_ahead(&mut self) -> Result<Option<wint_t>, i32> {
        let buf = &mut [0];
        let seek = unsafe { ftell_locked(self.f) };
        unsafe { fseek_locked(self.f, self.look_ahead as off_t, SEEK_SET) };
        let ret = match self.f.read(buf) {
            Ok(0) => Ok(None),
            Ok(_) => Ok(Some(buf[0])),
            Err(_) => Err(-1),
        };
        unsafe { fseek_locked(self.f, seek, SEEK_SET) };
        self.look_ahead += 1;

        // TODO: This is a 8 bit char, wee need to read a wchar
        ret.map(|c| c.map(wint_t::from))
    }

    fn commit(&mut self) {
        unsafe { fseek_locked(self.f, self.look_ahead as off_t, SEEK_SET) };
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

// pub struct LookAheadReader(LookAheadBuffer);
pub struct LookAheadReader<'a>(LookAheadReaderEnum<'a>);

impl LookAheadReader<'_> {
    pub fn lookahead1(&mut self) -> Result<Option<wint_t>, i32> {
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
}

impl<'a> From<&'a mut FILE> for LookAheadReader<'a> {
    fn from(f: &'a mut FILE) -> LookAheadReader {
        LookAheadReader(LookAheadReaderEnum::FILE(f.into()))
    }
}

impl<'a> From<*const wint_t> for LookAheadReader<'a> {
    fn from(buff: *const wint_t) -> LookAheadReader<'a> {
        LookAheadReader(LookAheadReaderEnum::BUFFER(buff.into()))
    }
}
