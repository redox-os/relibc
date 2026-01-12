use super::{FILE, SEEK_SET, fseek_locked, ftell_locked};
use crate::{
    header::{
        errno::EILSEQ,
        wchar::{MB_CUR_MAX, fgetwc, get_char_encoded_length, mbrtowc},
        wctype::WEOF,
    },
    io::Read,
    platform::{
        ERRNO,
        types::{c_char, off_t, wchar_t, wint_t},
    },
};
use core::{iter::Iterator, ptr};

struct BufferReader {
    buf: *const wint_t,
    position: usize,
}

impl From<*const wint_t> for BufferReader {
    fn from(buff: *const wint_t) -> Self {
        BufferReader {
            buf: buff,
            position: 0,
        }
    }
}

impl Iterator for BufferReader {
    type Item = Result<wint_t, i32>;

    fn next(&mut self) -> Option<Self::Item> {
        let wchar = unsafe { *self.buf.add(self.position) };
        if wchar == 0 {
            None
        } else {
            self.position += 1;
            Some(Ok(wchar))
        }
    }
}

struct FileReader<'a> {
    f: &'a mut FILE,
    position: off_t,
}

impl<'a> FileReader<'a> {
    // Gets the wchar at the current position
    #[inline]
    fn get_curret_wchar(&mut self) -> Result<Option<(wint_t, usize)>, i32> {
        let buf = &mut [0; MB_CUR_MAX as usize];
        let mut encoded_length = 0;
        let mut bytes_read = 0;

        loop {
            match self.f.read(&mut buf[bytes_read..bytes_read + 1]) {
                Ok(0) => return Ok(None),
                Ok(_) => {}
                Err(_) => return Err(-1),
            }

            bytes_read += 1;

            if bytes_read == 1 {
                encoded_length = if let Some(el) = get_char_encoded_length(buf[0]) {
                    el
                } else {
                    ERRNO.set(EILSEQ);
                    return Ok(Some((WEOF, 0)));
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
        }

        Ok(Some((wc as wint_t, encoded_length)))
    }
}

impl<'a> Iterator for FileReader<'a> {
    type Item = Result<wint_t, i32>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { fseek_locked(self.f, self.position, SEEK_SET) };

        match self.get_curret_wchar() {
            Ok(Some((wc, encoded_length))) => {
                unsafe { fseek_locked(self.f, self.position, SEEK_SET) };
                self.position += encoded_length as off_t;
                Some(Ok(wc))
            }
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'a> From<&'a mut FILE> for FileReader<'a> {
    fn from(f: &'a mut FILE) -> Self {
        let position = unsafe { ftell_locked(f) } as i64;
        FileReader { f, position }
    }
}

pub enum Reader<'a> {
    FILE(FileReader<'a>),
    BUFFER(BufferReader),
}

impl<'a> Iterator for Reader<'a> {
    type Item = Result<wint_t, i32>;

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

impl<'a> From<*const wchar_t> for Reader<'a> {
    fn from(buff: *const wchar_t) -> Self {
        Self::BUFFER((buff as *const wint_t).into())
    }
}
