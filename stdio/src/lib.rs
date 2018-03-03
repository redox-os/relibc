//! stdio implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/stdio.h.html

#![no_std]

extern crate platform;

use platform::types::*;

pub const BUFSIZ: c_int = 4096;

pub const FILENAME_MAX: c_int = 4096;

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
