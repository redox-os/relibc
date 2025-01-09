//! strings implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/strings.h.html

// TODO: set this for entire crate when possible
#![deny(unsafe_op_in_unsafe_fn)]

use core::{
    arch,
    iter::{once, zip},
    ptr,
};

use crate::{
    header::{ctype, string},
    iter::NulTerminated,
    platform::types::*,
};

#[no_mangle]
pub unsafe extern "C" fn bcmp(first: *const c_void, second: *const c_void, n: size_t) -> c_int {
    unsafe { string::memcmp(first, second, n) }
}

#[no_mangle]
pub unsafe extern "C" fn bcopy(src: *const c_void, dst: *mut c_void, n: size_t) {
    unsafe {
        ptr::copy(src as *const u8, dst as *mut u8, n);
    }
}

#[no_mangle]
pub unsafe extern "C" fn bzero(dst: *mut c_void, n: size_t) {
    unsafe {
        ptr::write_bytes(dst as *mut u8, 0, n);
    }
}

#[no_mangle]
pub unsafe extern "C" fn explicit_bzero(s: *mut c_void, n: size_t) {
    for i in 0..n {
        unsafe {
            *(s as *mut u8).add(i) = 0 as u8;
        }
    }
    unsafe {
        arch::asm!("");
    }
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
    unsafe { string::strchr(s, c) }
}

#[no_mangle]
pub unsafe extern "C" fn rindex(s: *const c_char, c: c_int) -> *mut c_char {
    unsafe { string::strrchr(s, c) }
}

#[no_mangle]
pub unsafe extern "C" fn strcasecmp(s1: *const c_char, s2: *const c_char) -> c_int {
    // SAFETY: the caller must ensure that s1 and s2 point to nul-terminated buffers.
    let s1_iter = unsafe { NulTerminated::new(s1).unwrap() }.chain(once(&0));
    let s2_iter = unsafe { NulTerminated::new(s2).unwrap() }.chain(once(&0));

    let zipped = zip(s1_iter, s2_iter);
    inner_casecmp(zipped)
}

#[no_mangle]
pub unsafe extern "C" fn strncasecmp(s1: *const c_char, s2: *const c_char, n: size_t) -> c_int {
    // SAFETY: the caller must ensure that s1 and s2 point to nul-terminated buffers.
    let s1_iter = unsafe { NulTerminated::new(s1).unwrap() }.chain(once(&0));
    let s2_iter = unsafe { NulTerminated::new(s2).unwrap() }.chain(once(&0));

    let zipped = zip(s1_iter, s2_iter).take(n);
    inner_casecmp(zipped)
}

/// Given two zipped `&c_char` iterators, either find the first comparison != 0, or return 0.
fn inner_casecmp<'a>(iterator: impl Iterator<Item = (&'a c_char, &'a c_char)>) -> c_int {
    let mut cmp_iter =
        iterator.map(|(&c1, &c2)| ctype::tolower(c1.into()) - ctype::tolower(c2.into()));
    let mut skip_iter = cmp_iter.skip_while(|&cmp| cmp == 0);
    skip_iter.next().or(Some(0)).unwrap()
}
