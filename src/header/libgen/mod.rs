//! libgen implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/libgen.h.html

use crate::platform::types::c_char;

use crate::header::string::strlen;

#[no_mangle]
pub unsafe extern "C" fn basename(str: *mut c_char) -> *mut c_char {
    if str.is_null() || strlen(str) == 0 {
        return ".\0".as_ptr() as *mut c_char;
    }
    let mut end = strlen(str) as isize - 1;
    while end >= 0 && *str.offset(end) == b'/' as c_char {
        end -= 1;
    }
    if end == -1 {
        return "/\0".as_ptr() as *mut c_char;
    }
    let mut begin = end;
    while begin >= 0 && *str.offset(begin) != b'/' as c_char {
        begin -= 1;
    }
    *str.offset(end + 1) = 0;
    str.offset(begin + 1) as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn dirname(str: *mut c_char) -> *mut c_char {
    if str.is_null() || strlen(str) == 0 {
        return ".\0".as_ptr() as *mut c_char;
    }
    let mut end = strlen(str) as isize - 1;
    while end > 0 && *str.offset(end) == b'/' as c_char {
        end -= 1;
    }
    while end >= 0 && *str.offset(end) != b'/' as c_char {
        end -= 1;
    }
    while end > 0 && *str.offset(end) == b'/' as c_char {
        end -= 1;
    }
    if end == -1 {
        return ".\0".as_ptr() as *mut c_char;
    }
    *str.offset(end + 1) = 0;
    str
}
