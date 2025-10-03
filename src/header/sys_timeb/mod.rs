//! `sys/timeb.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/009695399/basedefs/sys/timeb.h.html>.
//!
//! # Deprecation
//! The `ftime()` function was marked as legacy in the Open Group Base
//! Specifications Issue 6, and the entire `sys/timeb.h` header was removed in
//! Issue 7.

// TODO: set this for entire crate when possible
#![deny(unsafe_op_in_unsafe_fn)]

use core::ptr::NonNull;

use crate::{
    header::sys_time::{gettimeofday, timeval, timezone},
    platform::types::*,
};

/// See <https://pubs.opengroup.org/onlinepubs/009695399/basedefs/sys/timeb.h.html>.
#[repr(C)]
#[derive(Default)]
pub struct timeb {
    pub time: time_t,
    pub millitm: c_ushort,
    pub timezone: c_short,
    pub dstflag: c_short,
}

/// See <https://pubs.opengroup.org/onlinepubs/009695399/functions/ftime.html>.
///
/// # Safety
/// The caller must ensure that `tp` is convertible to a `&mut
/// MaybeUninit<timeb>`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ftime(tp: *mut timeb) -> c_int {
    // SAFETY: the caller is required to ensure that the pointer is valid.
    let tp_maybe_uninit = unsafe { NonNull::new_unchecked(tp).as_uninit_mut() };

    let mut tv = timeval::default();
    let mut tz = timezone::default();

    // SAFETY: tv and tz are created above, and thus will coerce to valid
    // pointers.
    if unsafe { gettimeofday(&mut tv, &mut tz) } < 0 {
        return -1;
    }

    tp_maybe_uninit.write(timeb {
        time: tv.tv_sec,
        millitm: (tv.tv_usec / 1000) as c_ushort,
        timezone: tz.tz_minuteswest as c_short,
        dstflag: tz.tz_dsttime as c_short,
    });

    0
}
