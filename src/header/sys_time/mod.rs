//! `sys/time.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_time.h.html>.

use crate::{
    c_str::CStr,
    error::ResultExt,
    header::time::timespec,
    out::Out,
    platform::{types::*, Pal, PalSignal, Sys},
};
use core::ptr::null;

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_time.h.html>.
///
/// # Deprecation
/// The `ITIMER_REAL` symbolic constant was marked obsolescent in the Open
/// Group Base Specifications Issue 7, and removed in Issue 8.
#[deprecated]
pub const ITIMER_REAL: c_int = 0;

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_time.h.html>.
///
/// # Deprecation
/// The `ITIMER_VIRTUAL` symbolic constant was marked obsolescent in the Open
/// Group Base Specifications Issue 7, and removed in Issue 8.
#[deprecated]
pub const ITIMER_VIRTUAL: c_int = 1;

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_time.h.html>.
///
/// # Deprecation
/// The `ITIMER_PROF` symbolic constant was marked obsolescent in the Open
/// Group Base Specifications Issue 7, and removed in Issue 8.
#[deprecated]
pub const ITIMER_PROF: c_int = 2;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_time.h.html>.
///
/// TODO: specified for `sys/select.h` in modern POSIX?
#[repr(C)]
pub struct fd_set {
    pub fds_bits: [c_long; 16usize],
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_time.h.html>.
///
/// # Deprecation
/// The `itimerval` struct was marked obsolescent in the Open Group Base
/// Specifications Issue 7, and removed in Issue 8.
#[deprecated]
#[repr(C)]
#[derive(Default)]
pub struct itimerval {
    pub it_interval: timeval,
    pub it_value: timeval,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_time.h.html>.
///
/// TODO: specified for `sys/select.h` in modern POSIX?
#[repr(C)]
#[derive(Default)]
pub struct timeval {
    pub tv_sec: time_t,
    pub tv_usec: suseconds_t,
}

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/gettimeofday.2.html>.
#[repr(C)]
#[derive(Default)]
pub struct timezone {
    pub tz_minuteswest: c_int,
    pub tz_dsttime: c_int,
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/getitimer.html>.
///
/// # Deprecation
/// The `getitimer()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 7, and removed in Issue 8.
#[deprecated]
#[no_mangle]
pub unsafe extern "C" fn getitimer(which: c_int, value: *mut itimerval) -> c_int {
    Sys::getitimer(which, &mut *value)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/gettimeofday.html>.
///
/// See also <https://www.man7.org/linux/man-pages/man2/gettimeofday.2.html>
/// for further details on the `tzp` argument.
///
/// # Deprecation
/// The `gettimeofday()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 7, and removed in Issue 8.
#[deprecated]
#[no_mangle]
pub unsafe extern "C" fn gettimeofday(tp: *mut timeval, tzp: *mut timezone) -> c_int {
    Sys::gettimeofday(Out::nonnull(tp), Out::nullable(tzp))
        .map(|()| 0)
        .or_minus_one_errno()
}

// `select()` declared in `sys/select.h`, as specified in modern POSIX

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/getitimer.html>.
///
/// # Deprecation
/// The `setitimer()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 7, and removed in Issue 8.
#[deprecated]
#[no_mangle]
pub unsafe extern "C" fn setitimer(
    which: c_int,
    value: *const itimerval,
    ovalue: *mut itimerval,
) -> c_int {
    // TODO setitimer is unimplemented on Redox
    Sys::setitimer(which, &*value, ovalue.as_mut())
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/utimes.html>.
///
/// # Deprecation
/// The `utimes()` function was marked legacy in the Open Group Base
/// Specifications Issue 6, and then unmarked in Issue 7.
#[no_mangle]
pub unsafe extern "C" fn utimes(path: *const c_char, times: *const timeval) -> c_int {
    let path = CStr::from_ptr(path);
    // Nullptr is valid here, it means "use current time"
    let times_spec = if times.is_null() {
        null()
    } else {
        {
            [
                timespec {
                    tv_sec: (*times.offset(0)).tv_sec,
                    tv_nsec: ((*times.offset(0)).tv_usec as c_long) * 1000,
                },
                timespec {
                    tv_sec: (*times.offset(1)).tv_sec,
                    tv_nsec: ((*times.offset(1)).tv_usec as c_long) * 1000,
                },
            ]
        }
        .as_ptr()
    };
    Sys::utimens(path, times_spec)
        .map(|()| 0)
        .or_minus_one_errno()
}
