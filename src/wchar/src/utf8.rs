//UTF implementation parts for wchar.h.
//Partially ported from the Sortix libc

extern crate errno;
extern crate platform;

use platform::types::*;
use mbstate_t;
use core::{slice,str,usize,char};

//It's guaranteed that we don't have any nullpointers here
pub unsafe fn mbrtowc(pwc: *mut wchar_t, s: *const c_char, n: usize, ps: *mut mbstate_t) -> usize {
    let mut size = str::utf8_char_width(*s as u8);
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

    return if result != 0 { size } else { 0 };
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
    let mut slice = slice::from_raw_parts_mut(s as *mut u8, size);

    c.encode_utf8(slice);

    size
}
