//! stdio implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/stdio.h.html

#![no_std]

extern crate platform;
extern crate va_list as vl;

use platform::types::*;
use vl::VaList as va_list;

mod printf;

pub const BUFSIZ: c_int = 4096;

pub const FILENAME_MAX: c_int = 4096;

pub struct FILE;

#[no_mangle]
pub static mut stdout: *mut FILE = 1 as *mut FILE;

#[no_mangle]
pub static mut stderr: *mut FILE = 2 as *mut FILE;

#[no_mangle]
pub unsafe extern "C" fn vfprintf(file: *mut FILE, format: *const c_char, ap: va_list) -> c_int {
    printf::printf(platform::FileWriter(file as c_int), format, ap)
}

#[no_mangle]
pub unsafe extern "C" fn vprintf(format: *const c_char, ap: va_list) -> c_int {
    vfprintf(stdout, format, ap)
}

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
