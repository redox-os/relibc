//! `wctype.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/wctype.h.html>.

// TODO: set this for entire crate when possible
#![deny(unsafe_op_in_unsafe_fn)]

// TODO: *_l functions

use self::casecmp::casemap;
use crate::{
    c_str::CStr,
    header::ctype,
    platform::types::{c_char, c_int, wint_t},
};

mod alpha;
mod casecmp;
mod punct;

pub type wctype_t = u32;
pub type wctrans_t = *const i32;

pub const WEOF: wint_t = 0xFFFF_FFFFu32;

pub const WCTYPE_ALNUM: wctype_t = 1;
pub const WCTYPE_ALPHA: wctype_t = 2;
pub const WCTYPE_BLANK: wctype_t = 3;
pub const WCTYPE_CNTRL: wctype_t = 4;
pub const WCTYPE_DIGIT: wctype_t = 5;
pub const WCTYPE_GRAPH: wctype_t = 6;
pub const WCTYPE_LOWER: wctype_t = 7;
pub const WCTYPE_PRINT: wctype_t = 8;
pub const WCTYPE_PUNCT: wctype_t = 9;
pub const WCTYPE_SPACE: wctype_t = 10;
pub const WCTYPE_UPPER: wctype_t = 11;
pub const WCTYPE_XDIGIT: wctype_t = 12;

const WCTRANSUP: wctrans_t = 1 as wctrans_t;
const WCTRANSLW: wctrans_t = 2 as wctrans_t;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/iswalnum.html>.
#[unsafe(no_mangle)]
pub extern "C" fn iswalnum(wc: wint_t) -> c_int {
    c_int::from(iswdigit(wc) != 0 || iswalpha(wc) != 0)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/iswalpha.html>.
#[unsafe(no_mangle)]
pub extern "C" fn iswalpha(wc: wint_t) -> c_int {
    c_int::from(alpha::is(wc as usize))
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/iswblank.html>.
#[unsafe(no_mangle)]
pub extern "C" fn iswblank(wc: wint_t) -> c_int {
    ctype::isblank(wc as c_int)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/iswcntrl.html>.
#[unsafe(no_mangle)]
pub extern "C" fn iswcntrl(wc: wint_t) -> c_int {
    c_int::from(
        wc < 32
            || wc.wrapping_sub(0x7f) < 33
            || wc.wrapping_sub(0x2028) < 2
            || wc.wrapping_sub(0xfff9) < 3,
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/iswctype.html>.
#[unsafe(no_mangle)]
pub extern "C" fn iswctype(wc: wint_t, desc: wctype_t) -> c_int {
    match desc {
        WCTYPE_ALNUM => iswalnum(wc),
        WCTYPE_ALPHA => iswalpha(wc),
        WCTYPE_BLANK => iswblank(wc),
        WCTYPE_CNTRL => iswcntrl(wc),
        WCTYPE_DIGIT => iswdigit(wc),
        WCTYPE_GRAPH => iswgraph(wc),
        WCTYPE_LOWER => iswlower(wc),
        WCTYPE_PRINT => iswprint(wc),
        WCTYPE_PUNCT => iswpunct(wc),
        WCTYPE_SPACE => iswspace(wc),
        WCTYPE_UPPER => iswupper(wc),
        WCTYPE_XDIGIT => iswxdigit(wc),
        _ => 0,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/iswdigit.html>.
#[unsafe(no_mangle)]
pub extern "C" fn iswdigit(wc: wint_t) -> c_int {
    c_int::from(wc.wrapping_sub('0' as wint_t) < 10)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/iswgraph.html>.
#[unsafe(no_mangle)]
pub extern "C" fn iswgraph(wc: wint_t) -> c_int {
    c_int::from(iswspace(wc) == 0 && iswprint(wc) != 0)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/iswlower.html>.
#[unsafe(no_mangle)]
pub extern "C" fn iswlower(wc: wint_t) -> c_int {
    c_int::from(towupper(wc) != wc)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/iswprint.html>.
#[unsafe(no_mangle)]
pub extern "C" fn iswprint(wc: wint_t) -> c_int {
    if wc < 0xff {
        c_int::from((wc + 1 & 0x7f) >= 0x21)
    } else if wc < 0x2028
        || wc.wrapping_sub(0x202a) < 0xd800 - 0x202a
        || wc.wrapping_sub(0xe000) < 0xfff9 - 0xe000
    {
        1
    } else if wc.wrapping_sub(0xfffc) > 0x10ffff - 0xfffc || (wc & 0xfffe) == 0xfffe {
        0
    } else {
        1
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/iswpunct.html>.
#[unsafe(no_mangle)]
pub extern "C" fn iswpunct(wc: wint_t) -> c_int {
    c_int::from(punct::is(wc as usize))
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/iswspace.html>.
#[unsafe(no_mangle)]
pub extern "C" fn iswspace(wc: wint_t) -> c_int {
    c_int::from(
        [
            ' ' as wint_t,
            '\t' as wint_t,
            '\n' as wint_t,
            '\r' as wint_t,
            11,
            12,
            0x0085,
            0x2000,
            0x2001,
            0x2002,
            0x2003,
            0x2004,
            0x2005,
            0x2006,
            0x2008,
            0x2009,
            0x200a,
            0x2028,
            0x2029,
            0x205f,
            0x3000,
        ]
        .contains(&wc),
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/iswupper.html>.
#[unsafe(no_mangle)]
pub extern "C" fn iswupper(wc: wint_t) -> c_int {
    c_int::from(towlower(wc) != wc)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/iswxdigit.html>.
#[unsafe(no_mangle)]
pub extern "C" fn iswxdigit(wc: wint_t) -> c_int {
    c_int::from(wc.wrapping_sub('0' as wint_t) < 10 || (wc | 32).wrapping_sub('a' as wint_t) < 6)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/towctrans.html>.
#[unsafe(no_mangle)]
pub extern "C" fn towctrans(wc: wint_t, trans: wctrans_t) -> wint_t {
    match trans {
        WCTRANSUP => towupper(wc),
        WCTRANSLW => towlower(wc),
        _ => wc,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/towlower.html>.
#[unsafe(no_mangle)]
pub extern "C" fn towlower(wc: wint_t) -> wint_t {
    casemap(wc, 0)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/towupper.html>.
#[unsafe(no_mangle)]
pub extern "C" fn towupper(wc: wint_t) -> wint_t {
    casemap(wc, 1)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wctrans.html>.
///
/// # Safety
/// The caller must ensure that `class` is convertible to a slice reference, up
/// to and including a terminating nul.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wctrans(class: *const c_char) -> wctrans_t {
    let class_cstr = unsafe { CStr::from_ptr(class) };
    match class_cstr.to_bytes() {
        b"toupper" => WCTRANSUP,
        b"tolower" => WCTRANSLW,
        _ => 0 as wctrans_t,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wctype.html>.
///
/// # Safety
/// The caller must ensure that `name` is convertible to a slice reference, up
/// to and including a terminating nul.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wctype(name: *const c_char) -> wctype_t {
    let name_cstr = unsafe { CStr::from_ptr(name) };
    match name_cstr.to_bytes() {
        b"alnum" => WCTYPE_ALNUM,
        b"alpha" => WCTYPE_ALPHA,
        b"blank" => WCTYPE_BLANK,
        b"cntrl" => WCTYPE_CNTRL,
        b"digit" => WCTYPE_DIGIT,
        b"graph" => WCTYPE_GRAPH,
        b"lower" => WCTYPE_LOWER,
        b"print" => WCTYPE_PRINT,
        b"punct" => WCTYPE_PUNCT,
        b"space" => WCTYPE_SPACE,
        b"upper" => WCTYPE_UPPER,
        b"xdigit" => WCTYPE_XDIGIT,
        _ => 0,
    }
}
