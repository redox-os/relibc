//! string implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/string.h.html

#![no_std]

extern crate platform;

use platform::types::*;


#[no_mangle]
pub extern "C" fn memccpy(
    s1: *mut c_void,
    s2: *const c_void,
    c: c_int,
    n: usize,
) -> *mut c_void {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn memchr(
    s: *const c_void,
    c: c_int,
    n: usize,
) -> *mut c_void {
    unimplemented!();
}

// #[no_mangle]
// pub extern "C" fn memcmp(
//     s1: *const c_void,
//     s2: *const c_void,
//     n: usize,
// ) -> c_int {
//     unimplemented!();
// }

// #[no_mangle]
// pub extern "C" fn memcpy(
//     s1: *mut c_void,
//     s2: *const c_void,
//     n: usize,
// ) -> *mut c_void {
//     unimplemented!();
// }

// #[no_mangle]
// pub extern "C" fn memmove(
//     s1: *mut c_void,
//     s2: *const c_void,
//     n: usize,
// ) -> *mut c_void {
//     unimplemented!();
// }

// #[no_mangle]
// pub extern "C" fn memset(
//     s: *mut c_void,
//     c: c_int,
//     n: usize,
// ) -> *mut c_void {
//     unimplemented!();
// }

#[no_mangle]
pub extern "C" fn strcat(s1: *mut c_char, s2: *const c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strchr(s: *const c_char, c: c_int) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strcoll(s1: *const c_char, s2: *const c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strcpy(s1: *mut c_char, s2: *const c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strcspn(s1: *const c_char, s2: *const c_char) -> c_ulong {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strdup(s1: *const c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strerror(errnum: c_int) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const c_char) -> size_t {
    let mut size = 0;

    loop {
        if *s.offset(size) == 0 {
            break;
        }
        size += 1;
    }

    size as size_t
}

#[no_mangle]
pub extern "C" fn strncat(
    s1: *mut c_char,
    s2: *const c_char,
    n: usize,
) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strncmp(
    s1: *const c_char,
    s2: *const c_char,
    n: usize,
) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strncpy(
    s1: *mut c_char,
    s2: *const c_char,
    n: usize,
) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strpbrk(
    s1: *const c_char,
    s2: *const c_char,
) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strrchr(s: *const c_char, c: c_int) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strspn(s1: *const c_char, s2: *const c_char) -> c_ulong {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strstr(
    s1: *const c_char,
    s2: *const c_char,
) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strtok(s1: *mut c_char, s2: *const c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strtok_r(
    s: *mut c_char,
    sep: *const c_char,
    lasts: *mut *mut c_char,
) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strxfrm(
    s1: *mut c_char,
    s2: *const c_char,
    n: usize,
) -> c_ulong {
    unimplemented!();
}

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
