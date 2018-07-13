//! wctype implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/wctype.h.html

#![no_std]

extern crate platform;

use platform::types::*;

// #[no_mangle]
pub extern "C" fn iswalnum(wc: wint_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswalpha(wc: wint_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswcntrl(wc: wint_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswdigit(wc: wint_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswgraph(wc: wint_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswlower(wc: wint_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswprint(wc: wint_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswpunct(wc: wint_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswspace(wc: wint_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswupper(wc: wint_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswxdigit(wc: wint_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswctype(wc: wint_t, charclass: wctype_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn towlower(wc: wint_t) -> wint_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn towupper(wc: wint_t) -> wint_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wctype(property: *const c_char) -> c_int {
    unimplemented!();
}
