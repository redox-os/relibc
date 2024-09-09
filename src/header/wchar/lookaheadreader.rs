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

pub struct LookAheadReader(LookAheadBuffer);

impl LookAheadReader {
    pub fn lookahead1(&mut self) -> Result<Option<wint_t>, i32> {
        self.0.look_ahead()
    }

    pub fn commit(&mut self) {
        self.0.commit()
    }
}

impl From<*const wint_t> for LookAheadReader {
    fn from(buff: *const wint_t) -> LookAheadReader {
        LookAheadReader(buff.into())
    }
}
