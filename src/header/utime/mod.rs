//! `utime.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/utime.h.html>.
//!
//! The `utime.h` header was marked obsolescent in the Open Group Base
//! Specifications Issue 7, and removed in Issue 8.

use crate::{
    c_str::CStr,
    error::ResultExt,
    header::time::timespec,
    platform::{
        Pal, Sys,
        types::{c_char, c_int, time_t},
    },
};

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/utime.h.html>.
///
/// A structure representing both access and modification times in seconds.
///
/// Times are measured in seconds since the Epoch.
#[deprecated]
#[repr(C)]
#[derive(Clone)]
pub struct utimbuf {
    /// Access time.
    pub actime: time_t,
    /// Modification time.
    pub modtime: time_t,
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/utime.html>.
///
/// Sets the access and modification times of the file named by the `path`
/// argument.
///
/// Upon success, returns `0`. Upon failure, returns `-1`, sets errno to
/// indicate the error, and the file times shall not be affected.
///
/// # Deprecated
/// Marked obsolete in issue 7, removed in issue 8.
///
/// Should use `utimensat()` instead for greater accuracy because `utimebuf`
/// uses `time_t` which represents whole seconds only.
#[deprecated]
#[expect(deprecated, reason = "utimbuf struct")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utime(path: *const c_char, times: *const utimbuf) -> c_int {
    let filename_cstr = unsafe { CStr::from_ptr(path) };
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
