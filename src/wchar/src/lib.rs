//! wchar implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/string.h.html

#![no_std]

extern crate errno;
extern crate platform;
extern crate stdlib;
extern crate string;
extern crate time;

use platform::types::*;
use errno::*;
use time::*;
use core::cmp;
use core::usize;
use core::ptr;
use core::mem;
use string::*;

pub type wint_t = i32;

#[repr(C)]
pub struct mbstate_t {
    pub mbs_count: c_int,
    pub mbs_length: c_int,
    pub mbs_wch: wint_t,
}

#[no_mangle]
pub unsafe extern "C" fn btowc(c: c_int) -> wint_t {
    //Check for EOF
    if c == -1 {
        return -1;
    }

    let uc = c as u8;
    let c = uc as c_char;
    let mut ps: mbstate_t = mbstate_t {
        mbs_count: 0,
        mbs_length: 0,
        mbs_wch: 0,
    };
    let mut wc: wchar_t = 0;
    let saved_errno = platform::errno;
    let status = mbrtowc(&mut wc, &c as (*const c_char), 1, &mut ps);
    if status == usize::max_value() || status == usize::max_value() - 1 {
        platform::errno = saved_errno;
        return platform::errno;
    }
    return wc as wint_t;
}

#[no_mangle]
pub extern "C" fn getwchar() -> wint_t {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn mbsinit(ps: *const mbstate_t) -> c_int {
    if ps.is_null() || (*ps).mbs_count == 0 {
        return 1;
    } else {
        return 0;
    }
}

#[no_mangle]
pub unsafe extern "C" fn mbrlen(s: *const c_char, n: usize, ps: *mut mbstate_t) -> usize {
    static mut internal: mbstate_t = mbstate_t {
        mbs_count: 0,
        mbs_length: 0,
        mbs_wch: 0,
    };
    return mbrtowc(ptr::null_mut(), s, n, &mut internal);
}

//Only works for utf8!
#[no_mangle]
pub unsafe extern "C" fn mbrtowc(
    pwc: *mut wchar_t,
    s: *const c_char,
    n: usize,
    ps: *mut mbstate_t,
) -> usize {
    static mut internal: mbstate_t = mbstate_t {
        mbs_count: 0,
        mbs_length: 0,
        mbs_wch: 0,
    };

    if ps.is_null() {
        let ps = &mut internal;
    }
    if s.is_null() {
        let xs: [c_char; 1] = [0];
        utf8_mbrtowc(pwc, &xs[0] as *const c_char, 1, ps);
    }

    return utf8_mbrtowc(pwc, s, n, ps);
}

#[no_mangle]
unsafe extern "C" fn utf8_mbrtowc(
    pwc: *mut wchar_t,
    s: *const c_char,
    n: usize,
    ps: *mut mbstate_t,
) -> usize {
    let mut i: usize = 0;

    while !(i > 0 && (*ps).mbs_count == 0) {
        if (n <= i) {
            return -2isize as usize;
        }
        let c = s.offset(i as isize);
        let uc = c as u8;

        if (*ps).mbs_count == 0 {
            //1 byte sequence - 00–7F
            if (uc & 0b10000000) == 0b00000000 {
                (*ps).mbs_count = 0;
                (*ps).mbs_length = 1;
                (*ps).mbs_wch = (uc as wchar_t & 0b1111111) as wint_t;
            }
            //2 byte sequence - C2–DF
            else if (uc & 0b11100000) == 0b11000000 {
                (*ps).mbs_count = 1;
                (*ps).mbs_length = 2;
                (*ps).mbs_wch = (uc as wchar_t & 0b11111) as wint_t;
            }
            //3 byte sequence - E0–EF
            else if (uc & 0b11110000) == 0b11100000 {
                (*ps).mbs_count = 2;
                (*ps).mbs_length = 3;
                (*ps).mbs_wch = (uc as wchar_t & 0b1111) as wint_t;
            }
            //4 byte sequence - F0–F4
            else if (uc & 0b11111000) == 0b11110000 {
                (*ps).mbs_count = 3;
                (*ps).mbs_length = 4;
                (*ps).mbs_wch = (uc as wchar_t & 0b111) as wint_t;
            } else {
                platform::errno = errno::EILSEQ;
                return -1isize as usize;
            }
        } else {
            if (uc & 0b11000000) != 0b10000000 {
                platform::errno = errno::EILSEQ;
                return -1isize as usize;
            }

            (*ps).mbs_wch = (*ps).mbs_wch << 6 | (uc & 0b00111111) as wint_t;
            (*ps).mbs_count -= 1;
        }

        i += 1;
    }

    // Reject the character if it was produced with an overly long sequence.
    if (*ps).mbs_length == 1 && 1 << 7 <= (*ps).mbs_wch {
        platform::errno = errno::EILSEQ;
        return -1isize as usize;
    }
    if (*ps).mbs_length == 2 && 1 << (5 + 1 * 6) <= (*ps).mbs_wch {
        platform::errno = errno::EILSEQ;
        return -1isize as usize;
    }
    if (*ps).mbs_length == 3 && 1 << (5 + 2 * 6) <= (*ps).mbs_wch {
        platform::errno = errno::EILSEQ;
        return -1isize as usize;
    }
    if (*ps).mbs_length == 4 && 1 << (5 + 3 * 6) <= (*ps).mbs_wch {
        platform::errno = errno::EILSEQ;
        return -1isize as usize;
    }

    // The definition of UTF-8 prohibits encoding character numbers between
    // U+D800 and U+DFFF, which are reserved for use with the UTF-16 encoding
    // form (as surrogate pairs) and do not directly represent characters.
    if 0xD800 <= (*ps).mbs_wch && (*ps).mbs_wch <= 0xDFFF {
        platform::errno = errno::EILSEQ;
        return -1isize as usize;
    }
    // RFC 3629 limits UTF-8 to 0x0 through 0x10FFFF.
    if 0x10FFFF <= (*ps).mbs_wch {
        platform::errno = errno::EILSEQ;
        return -1isize as usize;
    }

    let result: wchar_t = (*ps).mbs_wch as wchar_t;

    if !pwc.is_null() {
        *pwc = result;
    }

    (*ps).mbs_length = 0;
    (*ps).mbs_wch = 0;

    return if result != 0 { i } else { 0 };
}

#[no_mangle]
pub extern "C" fn mbsrtowcs(
    dst: *mut wchar_t,
    src: *mut *const c_char,
    len: usize,
    ps: *mut mbstate_t,
) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn putwchar(wc: wchar_t) -> wint_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn towlower(wc: wint_t) -> wint_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn towupper(wc: wint_t) -> wint_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcrtomb(s: *mut c_char, wc: wchar_t, ps: *mut mbstate_t) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcscat(ws1: *mut wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcschr(ws1: *const wchar_t, ws2: wchar_t) -> *mut c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcscmp(ws1: *const wchar_t, ws2: *const wchar_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcscoll(ws1: *const wchar_t, ws2: *const wchar_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcscpy(ws1: *mut wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcscspn(ws1: *const wchar_t, ws2: *const wchar_t) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcsftime(
    wcs: *mut wchar_t,
    maxsize: usize,
    format: *const wchar_t,
    timptr: *mut tm,
) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcslen(ws: *const wchar_t) -> c_ulong {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcsncat(ws1: *mut wchar_t, ws2: *const wchar_t, n: usize) -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcsncmp(ws1: *const wchar_t, ws2: *const wchar_t, n: usize) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcsncpy(ws1: *mut wchar_t, ws2: *const wchar_t, n: usize) -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcspbrk(ws1: *const wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcsrchr(ws1: *const wchar_t, ws2: wchar_t) -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcsrtombs(
    dst: *mut c_char,
    src: *mut *const wchar_t,
    len: usize,
    ps: *mut mbstate_t,
) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcsspn(ws1: *const wchar_t, ws2: *const wchar_t) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcsstr(ws1: *const wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcstod(nptr: *const wchar_t, endptr: *mut *mut wchar_t) -> f64 {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcstok(
    ws1: *mut wchar_t,
    ws2: *const wchar_t,
    ptr: *mut *mut wchar_t,
) -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcstol(nptr: *const wchar_t, endptr: *mut *mut wchar_t, base: c_int) -> c_long {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcstoul(nptr: *const wchar_t, endptr: *mut *mut wchar_t, base: c_int) -> c_ulong {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcswcs(ws1: *const wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcswidth(pwcs: *const wchar_t, n: usize) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcsxfrm(ws1: *mut wchar_t, ws2: *const wchar_t, n: usize) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wctob(c: wint_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcwidth(wc: wchar_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wmemchr(ws: *const wchar_t, wc: wchar_t, n: usize) -> *mut c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wmemcmp(ws1: *const wchar_t, ws2: *const wchar_t, n: usize) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wmemcpy(ws1: *mut wchar_t, ws2: *const wchar_t, n: usize) -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wmemmove(ws1: *mut wchar_t, ws2: *const wchar_t, n: usize) -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wmemset(ws1: *mut wchar_t, ws2: wchar_t, n: usize) -> *mut wchar_t {
    unimplemented!();
}
