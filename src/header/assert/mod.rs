//! assert implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/assert.h.html

// TODO: set this for entire crate when possible
#![deny(unsafe_op_in_unsafe_fn)]

use crate::{c_str::CStr, platform::types::*};

#[no_mangle]
pub unsafe extern "C" fn __assert_fail(
    func: *const c_char,
    file: *const c_char,
    line: c_int,
    cond: *const c_char,
) -> ! {
    let func = unsafe { CStr::from_ptr(func) }.to_str().unwrap();
    let file = unsafe { CStr::from_ptr(file) }.to_str().unwrap();
    let cond = unsafe { CStr::from_ptr(cond) }.to_str().unwrap();

    eprintln!("{}: {}:{}: Assertion `{}` failed.", func, file, line, cond);

    core::intrinsics::abort();
}
