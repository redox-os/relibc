//! `utime.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/utime.h.html>.
//!
//! The `utime.h` header was marked obsolescent in the Open Group Base
//! Specifications Issue 7, and removed in Issue 8.

// TODO: set this for entire crate when possible
#![deny(unsafe_op_in_unsafe_fn)]

use crate::{
    c_str::CStr,
    error::ResultExt,
    header::time::timespec,
    platform::{types::*, Pal, Sys},
};

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/utime.h.html>.
#[deprecated]
#[repr(C)]
#[derive(Clone)]
pub struct utimbuf {
    pub actime: time_t,
    pub modtime: time_t,
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/utime.html>.
#[deprecated]
#[no_mangle]
pub unsafe extern "C" fn utime(filename: *const c_char, times: *const utimbuf) -> c_int {
    let filename_cstr = unsafe { CStr::from_ptr(filename) };
    let times_ref = unsafe { &*times };
    let times_spec = [
        timespec {
            tv_sec: times_ref.actime,
            tv_nsec: 0,
        },
        timespec {
            tv_sec: times_ref.modtime,
            tv_nsec: 0,
        },
    ];
    unsafe { Sys::utimens(filename_cstr, times_spec.as_ptr()) }
        .map(|()| 0)
        .or_minus_one_errno()
}
