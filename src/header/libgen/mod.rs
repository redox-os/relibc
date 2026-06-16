//! `libgen.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/libgen.h.html>.

use crate::{byte_literal::ByteLiteral, header::string::strlen, platform::types::c_char};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/basename.html>.
///
/// Takes the pathname pointed to by `path` and returns a pointer to the final
/// component of the pathname, deleting any trailing `/` characters.
///
/// Always successful. The return value never represents an error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn basename(path: *mut c_char) -> *mut c_char {
    if path.is_null() || unsafe { strlen(path) == 0 } {
        return c".".as_ptr().cast_mut();
    }
    let mut end = unsafe { strlen(path) as isize - 1 };
    while end >= 0 && unsafe { *path.offset(end) == ByteLiteral::cast_cchar(b'/') } {
        end -= 1;
    }
    if end == -1 {
        return c"/".as_ptr().cast_mut();
    }
    let mut begin = end;
    while begin >= 0 && unsafe { *path.offset(begin) != ByteLiteral::cast_cchar(b'/') } {
        begin -= 1;
    }
    unsafe {
        *path.offset(end + 1) = 0;
        path.offset(begin + 1).cast::<c_char>()
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/dirname.html>.
///
/// Takes a pointer to a character string that contains a pathname, and return
/// a pointer to a string that is a pathname of the directory containing the
/// entry of the final pathname component.
///
/// Always successful. The return value never represents an error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dirname(path: *mut c_char) -> *mut c_char {
    if path.is_null() || unsafe { strlen(path) == 0 } {
        return c".".as_ptr().cast_mut();
    }
    let mut end = unsafe { strlen(path) as isize - 1 };
    while end > 0 && unsafe { *path.offset(end) == ByteLiteral::cast_cchar(b'/') } {
        end -= 1;
    }
    while end >= 0 && unsafe { *path.offset(end) != ByteLiteral::cast_cchar(b'/') } {
        end -= 1;
    }
    while end > 0 && unsafe { *path.offset(end) == ByteLiteral::cast_cchar(b'/') } {
        end -= 1;
    }
    if end == -1 {
        return c".".as_ptr().cast_mut();
    }
    unsafe {
        *path.offset(end + 1) = 0;
    }
    path
}
