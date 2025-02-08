use super::{fseek_locked, ftell_locked, FILE, SEEK_SET};
use crate::{
    header::{
        errno::EILSEQ,
        wchar::{fgetwc, get_char_encoded_length, mbrtowc, MB_CUR_MAX},
        wctype::WEOF,
    },
    io::Read,
    platform::{
        types::{c_char, off_t, wchar_t, wint_t},
        ERRNO,
    },
};
use core::ptr;

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
        let buf = &mut [0; MB_CUR_MAX as usize];
        let seek = unsafe { ftell_locked(self.f) };
        unsafe { fseek_locked(self.f, self.look_ahead as off_t, SEEK_SET) };

        let mut encoded_length = 0;
        let mut bytes_read = 0;
        loop {
            match self.f.read(&mut buf[bytes_read..bytes_read + 1]) {
                Ok(0) => {
                    ERRNO.set(EILSEQ);
                    return Ok(Some(WEOF));
                }
                Ok(_) => {}
                Err(_) => return Err(-1),
            }

            bytes_read += 1;

            if bytes_read == 1 {
                encoded_length = if let Some(el) = get_char_encoded_length(buf[0]) {
                    el
                } else {
                    ERRNO.set(EILSEQ);
                    return Ok(Some(WEOF));
                };
            }

            if bytes_read >= encoded_length {
                break;
            }
        }

        let mut wc: wchar_t = 0;
        unsafe {
            mbrtowc(
                &mut wc,
                buf.as_ptr() as *const c_char,
                encoded_length,
                ptr::null_mut(),
            );

            fseek_locked(self.f, seek, SEEK_SET);
        }

        self.look_ahead += encoded_length as i64;

        Ok(Some(wc as wint_t))
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
