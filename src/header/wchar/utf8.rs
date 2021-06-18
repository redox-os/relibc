//UTF implementation parts for wchar.h.
//Partially ported from the Sortix libc

use core::{char, slice, str, usize};

use crate::{
    header::errno,
    platform::{self, types::*},
};

use super::mbstate_t;

// Based on
// https://github.com/rust-lang/rust/blob/f24ce9b/library/core/src/str/validations.rs#L232-L257,
// because apparently somebody removed the `pub use` statement from `core::str`.

// https://tools.ietf.org/html/rfc3629
static UTF8_CHAR_WIDTH: [u8; 256] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x1F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x3F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x5F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x7F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, // 0x9F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, // 0xBF
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, // 0xDF
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // 0xEF
    4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xFF
];

// Given a first byte, determines how many bytes are in this UTF-8 character.
#[inline]
fn utf8_char_width(b: u8) -> usize {
    UTF8_CHAR_WIDTH[usize::from(b)].into()
}

//It's guaranteed that we don't have any nullpointers here
pub unsafe fn mbrtowc(pwc: *mut wchar_t, s: *const c_char, n: usize, ps: *mut mbstate_t) -> usize {
    let size = utf8_char_width(*s as u8);
    if size > n {
        platform::errno = errno::EILSEQ;
        return -2isize as usize;
    }
    if size == 0 {
        platform::errno = errno::EILSEQ;
        return -1isize as usize;
    }

    let slice = slice::from_raw_parts(s as *const u8, size);
    let decoded = str::from_utf8(slice);
    if decoded.is_err() {
        platform::errno = errno::EILSEQ;
        return -1isize as usize;
    }

    let wc = decoded.unwrap();

    let result: wchar_t = wc.chars().next().unwrap() as wchar_t;

    if !pwc.is_null() {
        *pwc = result;
    }

    if result != 0 {
        size
    } else {
        0
    }
}

//It's guaranteed that we don't have any nullpointers here
pub unsafe fn wcrtomb(s: *mut c_char, wc: wchar_t, ps: *mut mbstate_t) -> usize {
    let dc = char::from_u32(wc as u32);

    if dc.is_none() {
        platform::errno = errno::EILSEQ;
        return -1isize as usize;
    }

    let c = dc.unwrap();
    let size = c.len_utf8();
    let slice = slice::from_raw_parts_mut(s as *mut u8, size);

    c.encode_utf8(slice);

    size
}
