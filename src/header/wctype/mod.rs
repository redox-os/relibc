//! wchar implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/wctype.h.html

use crate::{
    c_str::CStr,
    header::ctype,
    platform::types::*,
};
use self::casecmp::casemap;

mod alpha;
mod casecmp;
mod punct;

pub type wctype_t = u32;

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

#[no_mangle]
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

#[no_mangle]
pub unsafe extern "C" fn wctype(name: *const c_char) -> wctype_t {
    let name_cstr = CStr::from_ptr(name);
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
        _ => 0
    }
}

#[no_mangle]
pub extern "C" fn iswalnum(wc: wint_t) -> c_int {
    c_int::from(iswdigit(wc) != 0 || iswalpha(wc) != 0)
}

#[no_mangle]
pub extern "C" fn iswalpha(wc: wint_t) -> c_int {
    c_int::from(alpha::is(wc as usize))
}

#[no_mangle]
pub extern "C" fn iswblank(wc: wint_t) -> c_int {
    ctype::isblank(wc as c_int)
}

#[no_mangle]
pub extern "C" fn iswcntrl(wc: wint_t) -> c_int {
    c_int::from(
        wc < 32 ||
        wc.wrapping_sub(0x7f) < 33 ||
        wc.wrapping_sub(0x2028) < 2 ||
        wc.wrapping_sub(0xfff9) < 3
    )
}

#[no_mangle]
pub extern "C" fn iswdigit(wc: wint_t) -> c_int {
    c_int::from(wc.wrapping_sub('0' as wint_t) < 10)
}

#[no_mangle]
pub extern "C" fn iswgraph(wc: wint_t) -> c_int {
    c_int::from(iswspace(wc) == 0 && iswprint(wc) != 0)
}

#[no_mangle]
pub extern "C" fn iswlower(wc: wint_t) -> c_int {
    c_int::from(towupper(wc) != wc)
}

#[no_mangle]
pub extern "C" fn iswprint(wc: wint_t) -> c_int {
    if wc < 0xff {
        c_int::from((wc+1 & 0x7f) >= 0x21)
    } else if wc < 0x2028
        || wc.wrapping_sub(0x202a) < 0xd800-0x202a
        || wc.wrapping_sub(0xe000) < 0xfff9-0xe000 {
        1
    } else if wc.wrapping_sub(0xfffc) > 0x10ffff-0xfffc
        || (wc&0xfffe)==0xfffe {
        0
    } else {
        1
    }
}

#[no_mangle]
pub extern "C" fn iswpunct(wc: wint_t) -> c_int {
    c_int::from(punct::is(wc as usize))
}

#[no_mangle]
pub extern "C" fn iswspace(wc: wint_t) -> c_int {
    c_int::from([
        ' ' as wint_t, '\t' as wint_t, '\n' as wint_t, '\r' as wint_t,
        11, 12, 0x0085,
        0x2000, 0x2001, 0x2002, 0x2003, 0x2004, 0x2005,
        0x2006, 0x2008, 0x2009, 0x200a,
        0x2028, 0x2029, 0x205f, 0x3000
    ].contains(&wc))
}

#[no_mangle]
pub extern "C" fn iswupper(wc: wint_t) -> c_int {
    c_int::from(towlower(wc) != wc)
}

#[no_mangle]
pub extern "C" fn iswxdigit(wc: wint_t) -> c_int {
    c_int::from(
        wc.wrapping_sub('0' as wint_t) < 10 ||
        (wc|32).wrapping_sub('a' as wint_t) < 6
    )
}

#[no_mangle]
pub extern "C" fn towlower(wc: wint_t) -> wint_t {
    casemap(wc, 0)
}

#[no_mangle]
pub extern "C" fn towupper(wc: wint_t) -> wint_t {
    casemap(wc, 1)
}
