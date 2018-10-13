//! strings implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/strings.h.html

use alloc::Vec;
use core::ptr;

use platform::types::*;

#[no_mangle]
pub unsafe extern "C" fn bcmp(first: *const c_void, second: *const c_void, n: size_t) -> c_int {
    let first = first as *const c_char;
    let second = second as *const c_char;

    for i in 0..n as isize {
        if *first.offset(i) != *second.offset(i) {
            return -1;
        }
    }
    0
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
pub unsafe extern "C" fn index(mut s: *const c_char, c: c_int) -> *mut c_char {
    while *s != 0 {
        if *s == c as c_char {
            // Input is const but output is mutable. WHY C, WHY DO THIS?
            return s as *mut c_char;
        }
        s = s.offset(1);
    }
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn rindex(mut s: *const c_char, c: c_int) -> *mut c_char {
    let original = s;
    while *s != 0 {
        s = s.offset(1);
    }

    while s != original {
        s = s.offset(-1);
        if *s == c as c_char {
            // Input is const but output is mutable. WHY C, WHY DO THIS?
            return s as *mut c_char;
        }
    }
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn strcasecmp(mut first: *const c_char, mut second: *const c_char) -> c_int {
    strncasecmp(first, second, size_t::max_value())
}

#[no_mangle]
pub unsafe extern "C" fn strncasecmp(
    mut first: *const c_char,
    mut second: *const c_char,
    mut n: size_t,
) -> c_int {
    while n > 0 && (*first != 0 || *second != 0) {
        if *first & !32 != *second & !32 {
            return -1;
        }

        first = first.offset(1);
        second = second.offset(1);
        n -= 1;
    }
    0
}
