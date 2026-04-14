use super::{FILE, SEEK_SET, fseek_locked, ftell_locked};
use crate::{
    c_str::{CStr, Kind, NulStr, Thin, WStr, Wide},
    header::{
        errno::EILSEQ,
        stdlib::MB_CUR_MAX,
        wchar::{get_char_encoded_length, mbrtowc},
        wctype::WEOF,
    },
    io::Read,
    platform::{
        ERRNO,
        types::{c_char, off_t, wchar_t, wint_t},
    },
};
use core::{
    iter::Iterator,
    marker::PhantomData,
    ptr::{self},
};

pub struct BufferReader<'a, T: Kind> {
    buf: NulStr<'a, T>,
}

impl<'a> From<WStr<'a>> for BufferReader<'a, Wide> {
    fn from(buff: WStr<'a>) -> Self {
        BufferReader { buf: buff }
    }
}

impl<'a> From<CStr<'a>> for BufferReader<'a, Thin> {
    fn from(buff: CStr<'a>) -> Self {
        BufferReader { buf: buff }
    }
}

impl<'a, T: Kind> Iterator for BufferReader<'a, T> {
    type Item = Result<T::Char, i32>;

    fn next(&mut self) -> Option<Self::Item> {
        self.buf.split_first().map(|(c, r)| {
            self.buf = r;
            Ok(c)
        })
    }
}

pub struct FileReader<'a, T: Kind> {
    f: &'a mut FILE,
    position: off_t,
    phantom: PhantomData<T>,
}

impl<'a, T: Kind> FileReader<'a, T> {
    // Gets the wchar at the current position
    #[inline]
    fn get_curret_char(&mut self) -> Result<Option<(T::Char, usize)>, i32> {
        if T::IS_THIN_NOT_WIDE {
            let mut buf: [u8; 1] = [0];
            match self.f.read(&mut buf) {
                Ok(0) => Ok(None),
                Ok(n) => Ok(Some((T::Char::from(buf[0]), n))),
                Err(_) => Err(-1),
            }
        } else {
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
                        return Self::get_char_from_wint(WEOF).map(|c| Some((c, 0)));
                    };
                }

                if bytes_read >= encoded_length {
                    break;
                }
            }

            let mut wc: wchar_t = 0;
            unsafe {
                mbrtowc(
                    &raw mut wc,
                    buf.as_ptr().cast::<c_char>(),
                    encoded_length,
                    ptr::null_mut(),
                );
            }

            Self::get_char_from_wint(wc as wint_t).map(|c| Some((c, encoded_length)))
        }
    }

    fn get_char_from_wint(wc: wint_t) -> Result<T::Char, i32> {
        if let Some(wc_char) = T::chars_from_bytes(&wc.to_be_bytes())
            && wc_char.len() == 1
        {
            Ok(wc_char[0])
        } else {
            Err(-1)
        }
    }
}

impl<'a, T: Kind> Iterator for FileReader<'a, T> {
    type Item = Result<T::Char, i32>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { fseek_locked(self.f, self.position, SEEK_SET) };

        match self.get_curret_char() {
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

impl<'a, T: Kind> From<&'a mut FILE> for FileReader<'a, T> {
    fn from(f: &'a mut FILE) -> Self {
        let position = unsafe { ftell_locked(f) } as off_t;
        FileReader {
            f,
            position,
            phantom: PhantomData::<T>,
        }
    }
}

pub enum Reader<'a, T: Kind> {
    FILE(FileReader<'a, T>, PhantomData<T>),
    BUFFER(BufferReader<'a, T>),
}

impl<'a, T: Kind> Iterator for Reader<'a, T> {
    type Item = Result<T::Char, i32>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::FILE(r, _) => r.next(),
            Self::BUFFER(r) => r.next(),
        }
    }
}

impl<'a, T: Kind> From<&'a mut FILE> for Reader<'a, T> {
    fn from(f: &'a mut FILE) -> Self {
        Self::FILE(f.into(), PhantomData::<T>)
    }
}

impl<'a> From<WStr<'a>> for Reader<'a, Wide> {
    fn from(buff: WStr<'a>) -> Self {
        Self::BUFFER(buff.into())
    }
}

impl<'a> From<CStr<'a>> for Reader<'a, Thin> {
    fn from(buff: CStr<'a>) -> Self {
        Self::BUFFER(buff.into())
    }
}
