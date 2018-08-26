//! strings implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/strings.h.html

use alloc::Vec;
use core::ptr;

use platform;
use platform::types::*;

#[no_mangle]
pub unsafe extern "C" fn bcmp(
    mut first: *const c_void,
    mut second: *const c_void,
    n: size_t,
) -> c_int {
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
    let src = src as *mut c_char;
    let dst = dst as *mut c_char;

    let mut tmp = Vec::with_capacity(n);
    for i in 0..n as isize {
        tmp.push(*src.offset(i));
    }
    for (i, val) in tmp.into_iter().enumerate() {
        *dst.offset(i as isize) = val;
    }
}

#[no_mangle]
pub unsafe extern "C" fn bzero(src: *mut c_void, n: size_t) {
    let src = src as *mut c_char;

    for i in 0..n as isize {
        *src.offset(i) = 0;
    }
}

#[no_mangle]
pub extern "C" fn ffs(mut i: c_int) -> c_int {
    if i == 0 {
        return 0;
    }
    let mut n = 1;
    while i & 1 == 0 {
        i >>= 1;
        n += 1;
    }
    n
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
    while *first != 0 && *second != 0 {
        let mut i = *first;
        let mut j = *second;

        if i >= b'A' as c_char && i <= b'Z' as c_char {
            i += (b'a' - b'A') as c_char;
        }
        if j >= b'A' as c_char && j <= b'Z' as c_char {
            j += (b'a' - b'A') as c_char;
        }

        if i != j {
            return -1;
        }

        first = first.offset(1);
        second = second.offset(1);
    }
    // Both strings didn't end with NUL bytes
    if *first != *second {
        return -1;
    }
    0
}
#[no_mangle]
pub unsafe extern "C" fn strncasecmp(
    mut first: *const c_char,
    mut second: *const c_char,
    mut n: size_t,
) -> c_int {
    while *first != 0 && *second != 0 && n > 0 {
        let mut i = *first;
        let mut j = *second;

        if i >= b'A' as c_char && i <= b'Z' as c_char {
            i += (b'a' - b'A') as c_char;
        }
        if j >= b'A' as c_char && j <= b'Z' as c_char {
            j += (b'a' - b'A') as c_char;
        }

        if i != j {
            return -1;
        }

        first = first.offset(1);
        second = second.offset(1);
        n -= 1;
    }
    // Both strings didn't end with NUL bytes (unless we reached the limit)
    if n != 0 && *first != *second {
        return -1;
    }
    0
}
