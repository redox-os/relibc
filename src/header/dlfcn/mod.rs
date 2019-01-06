//! dlfcn implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/dlfcn.h.html

use core::{ptr, str};

use c_str::CStr;
use platform::types::*;

pub const RTLD_LAZY: c_int = 0x0001;
pub const RTLD_NOW: c_int = 0x0002;
pub const RTLD_GLOBAL: c_int = 0x0100;
pub const RTLD_LOCAL: c_int = 0x0000;

#[no_mangle]
pub unsafe extern "C" fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void {
    let filename_cstr = CStr::from_ptr(filename);
    let filename_str = str::from_utf8_unchecked(filename_cstr.to_bytes());
    eprintln!("dlopen({}, {:#>04X})", filename_str, flags);
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void {
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn dlclose(handle: *mut c_void) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn dlerror() -> *mut c_char {
    ptr::null_mut()
}
