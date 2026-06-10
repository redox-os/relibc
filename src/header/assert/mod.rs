//! `assert.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/assert.h.html>.

use crate::{
    c_str::CStr,
    platform::types::{c_char, c_int},
};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/assert.html>.
///
/// Writes information about the function that failed to `stderr` and calls
/// `abort()`.
///
/// # Implementation
/// `assert()` is defined as a C macro in cbindgen that checks for `NDEBUG`
/// and if not found gets forwarded to this function call.
///
/// # Safety
/// `func`, `file` and `cond` are guaranteed to be non-empty and valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __assert_fail(
    func: *const c_char,
    file: *const c_char,
    line: c_int,
    cond: *const c_char,
) -> ! {
    // SAFETY: `func` corresponds to the identifier `__func__` which is
    // guaranteed to be non-empty and valid.
    let func = unsafe { CStr::from_ptr(func) }.to_string_lossy();
    // SAFETY: `file` corresponds to the macro `__FILE__` which is guaranteed
    // to be non-empty and valid.
    let file = unsafe { CStr::from_ptr(file) }.to_string_lossy();
    // SAFETY: `cond` corresponds to the condition being asserted and is
    // guaranteed to be non-empty and valid.
    let cond = unsafe { CStr::from_ptr(cond) }.to_string_lossy();

    eprintln!("{}: {}:{}: Assertion `{}` failed.", func, file, line, cond);

    core::intrinsics::abort();
}
