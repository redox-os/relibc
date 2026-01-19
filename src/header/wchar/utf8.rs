//UTF implementation parts for wchar.h.
//Partially ported from the Sortix libc

// TODO: set this for entire crate when possible
#![deny(unsafe_op_in_unsafe_fn)]

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
    let size = utf8_char_width(unsafe { *s } as u8);
    if size > n {
        platform::ERRNO.set(errno::EILSEQ);
        return -2isize as usize;
    }
    if size == 0 {
        platform::ERRNO.set(errno::EILSEQ);
        return unsafe { -1isize as usize };
    }

    let slice = unsafe { slice::from_raw_parts(s as *const u8, size) };
    let decoded = str::from_utf8(slice);
    if decoded.is_err() {
        platform::ERRNO.set(errno::EILSEQ);
        return -1isize as usize;
    }

    let wc = decoded.unwrap();

    let result: wchar_t = wc.chars().next().unwrap() as wchar_t;

    if !pwc.is_null() {
        unsafe { *pwc = result };
    }

    if result != 0 { size } else { 0 }
}

//It's guaranteed that we don't have any nullpointers here
pub unsafe fn wcrtomb(s: *mut c_char, wc: wchar_t, ps: *mut mbstate_t) -> usize {
    let dc = char::from_u32(wc as u32);

    if dc.is_none() {
        platform::ERRNO.set(errno::EILSEQ);
        return -1isize as usize;
    }

    let c = dc.unwrap();
    let size = c.len_utf8();
    let slice = unsafe { slice::from_raw_parts_mut(s as *mut u8, size) };

    c.encode_utf8(slice);

    size
}

/// Gets the encoded length of a character. It is used to recognize wide characters
pub fn get_char_encoded_length(first_byte: u8) -> Option<usize> {
    if first_byte >> 7 == 0 {
        Some(1)
    } else if first_byte >> 5 == 6 {
        Some(2)
    } else if first_byte >> 4 == 0xe {
        Some(3)
    } else if first_byte >> 3 == 0x1e {
        Some(4)
    } else {
        None
    }
}
