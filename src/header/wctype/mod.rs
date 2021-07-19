//! wchar implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/wctype.h.html

use crate::{
    c_str::CStr,
    platform::types::*,
};

mod casecmp;
use casecmp::casemap;

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
pub extern "C" fn towlower(wc: wint_t) -> wint_t {
    casemap(wc, 0)
}

#[no_mangle]
pub extern "C" fn towupper(wc: wint_t) -> wint_t {
    casemap(wc, 1)
}
