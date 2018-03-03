//! string implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/string.h.html

#![no_std]

extern crate platform;

use platform::types::*;

#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const c_char) -> size_t {
    let mut size = 0;

    loop {
        if *s.offset(size) == 0 {
            break;
        }
        size += 1;
    }

    size as size_t
}

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
