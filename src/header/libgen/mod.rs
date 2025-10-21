//! `libgen.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/libgen.h.html>.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::platform::types::c_char;

use crate::header::string::strlen;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/basename.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn basename(str: *mut c_char) -> *mut c_char {
    if str.is_null() || unsafe { strlen(str) == 0 } {
        return ".\0".as_ptr() as *mut c_char;
    }
    let mut end = unsafe { strlen(str) as isize - 1 };
    while end >= 0 && unsafe { *str.offset(end) == b'/' as c_char } {
        end -= 1;
    }
    if end == -1 {
        return "/\0".as_ptr() as *mut c_char;
    }
    let mut begin = end;
    while begin >= 0 && unsafe { *str.offset(begin) != b'/' as c_char } {
        begin -= 1;
    }
    unsafe {
        *str.offset(end + 1) = 0;
        str.offset(begin + 1) as *mut c_char
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/dirname.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dirname(str: *mut c_char) -> *mut c_char {
    if str.is_null() || unsafe { strlen(str) == 0 } {
        return ".\0".as_ptr() as *mut c_char;
    }
    let mut end = unsafe { strlen(str) as isize - 1 };
    while end > 0 && unsafe { *str.offset(end) == b'/' as c_char } {
        end -= 1;
    }
    while end >= 0 && unsafe { *str.offset(end) != b'/' as c_char } {
        end -= 1;
    }
    while end > 0 && unsafe { *str.offset(end) == b'/' as c_char } {
        end -= 1;
    }
    if end == -1 {
        return ".\0".as_ptr() as *mut c_char;
    }
    unsafe {
        *str.offset(end + 1) = 0;
    }
    str
}
