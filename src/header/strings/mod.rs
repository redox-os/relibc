//! strings implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/strings.h.html

use core::ptr;

use crate::{
    header::{ctype, string},
    platform::types::*,
};

#[no_mangle]
pub unsafe extern "C" fn bcmp(first: *const c_void, second: *const c_void, n: size_t) -> c_int {
    string::memcmp(first, second, n)
}

#[no_mangle]
pub unsafe extern "C" fn bcopy(src: *const c_void, dst: *mut c_void, n: size_t) {
    ptr::copy(src as *const u8, dst as *mut u8, n);
}

#[no_mangle]
pub unsafe extern "C" fn bzero(dst: *mut c_void, n: size_t) {
    ptr::write_bytes(dst as *mut u8, 0, n);
}

#[no_mangle]
pub extern "C" fn ffs(i: c_int) -> c_int {
    if i == 0 {
        return 0;
    }
    1 + i.trailing_zeros() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn index(s: *const c_char, c: c_int) -> *mut c_char {
    string::strchr(s, c)
}

#[no_mangle]
pub unsafe extern "C" fn rindex(s: *const c_char, c: c_int) -> *mut c_char {
    string::strrchr(s, c)
}

#[no_mangle]
pub unsafe extern "C" fn strcasecmp(first: *const c_char, second: *const c_char) -> c_int {
    strncasecmp(first, second, size_t::max_value())
}

#[no_mangle]
pub unsafe extern "C" fn strncasecmp(
    mut first: *const c_char,
    mut second: *const c_char,
    mut n: size_t,
) -> c_int {
    while n > 0 && (*first != 0 || *second != 0) {
        let cmp = ctype::tolower(*first as c_int) - ctype::tolower(*second as c_int);
        if cmp != 0 {
            return cmp;
        }

        first = first.offset(1);
        second = second.offset(1);
        n -= 1;
    }
    0
}
