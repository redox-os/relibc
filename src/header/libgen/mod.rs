//! `libgen.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/libgen.h.html>.

use crate::{byte_literal::ByteLiteral, header::string::strlen, platform::types::c_char};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/basename.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn basename(str: *mut c_char) -> *mut c_char {
    if str.is_null() || unsafe { strlen(str) == 0 } {
        return c".".as_ptr().cast_mut();
    }
    let mut end = unsafe { strlen(str) as isize - 1 };
    while end >= 0 && unsafe { *str.offset(end) == ByteLiteral::cast_cchar(b'/') } {
        end -= 1;
    }
    if end == -1 {
        return c"/".as_ptr().cast_mut();
    }
    let mut begin = end;
    while begin >= 0 && unsafe { *str.offset(begin) != ByteLiteral::cast_cchar(b'/') } {
        begin -= 1;
    }
    unsafe {
        *str.offset(end + 1) = 0;
        str.offset(begin + 1).cast::<c_char>()
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/dirname.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dirname(str: *mut c_char) -> *mut c_char {
    if str.is_null() || unsafe { strlen(str) == 0 } {
        return c".".as_ptr().cast_mut();
    }
    let mut end = unsafe { strlen(str) as isize - 1 };
    while end > 0 && unsafe { *str.offset(end) == ByteLiteral::cast_cchar(b'/') } {
        end -= 1;
    }
    while end >= 0 && unsafe { *str.offset(end) != ByteLiteral::cast_cchar(b'/') } {
        end -= 1;
    }
    while end > 0 && unsafe { *str.offset(end) == ByteLiteral::cast_cchar(b'/') } {
        end -= 1;
    }
    if end == -1 {
        return c".".as_ptr().cast_mut();
    }
    unsafe {
        *str.offset(end + 1) = 0;
    }
    str
}
