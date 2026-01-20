//! `assert.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/assert.h.html>.

use crate::{
    c_str::CStr,
    platform::types::{c_char, c_int},
};

#[unsafe(no_mangle)]
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
