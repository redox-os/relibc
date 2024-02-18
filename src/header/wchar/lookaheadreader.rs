use super::{fseek_locked, ftell_locked, FILE, SEEK_SET};
use crate::{io::Read, platform::types::off_t};
struct LookAheadBuffer {
    buf: *const u32,
    pos: isize,
    look_ahead: isize,
}
impl LookAheadBuffer {
    fn look_ahead(&mut self) -> Result<Option<u32>, i32> {
        // TODO: byte is not an accurate name
        let byte = unsafe { *self.buf.offset(self.look_ahead) };
        if byte == 0 {
            Ok(None)
        } else {
            self.look_ahead += 1;
            Ok(Some(byte))
        }
    }

    fn commit(&mut self) {
        self.pos = self.look_ahead;
    }
}

impl From<*const u32> for LookAheadBuffer {
    fn from(buff: *const u32) -> LookAheadBuffer {
        LookAheadBuffer {
            buf: buff,
            pos: 0,
            look_ahead: 0,
        }
    }
}

pub struct LookAheadReader(LookAheadBuffer);

impl LookAheadReader {
    pub fn lookahead1(&mut self) -> Result<Option<u32>, i32> {
        self.0.look_ahead()
    }

    pub fn commit(&mut self) {
        self.0.commit()
    }
}

impl From<*const u32> for LookAheadReader {
    fn from(buff: *const u32) -> LookAheadReader {
        LookAheadReader(buff.into())
    }
}
