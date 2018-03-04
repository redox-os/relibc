//! ctype implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/ctype.h.html

#![no_std]

extern crate platform;

use platform::types::*;

#[no_mangle]
pub extern "C" fn isalnum(c: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn isalpha(c: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn isascii(c: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iscntrl(c: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn isdigit(c: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn isgraph(c: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn islower(c: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn isprint(c: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ispunct(c: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn isspace(c: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn isupper(c: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn isxdigit(c: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn toascii(c: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn tolower(c: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn toupper(c: c_int) -> c_int {
    unimplemented!();
}
