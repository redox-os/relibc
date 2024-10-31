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
#[no_mangle]
pub unsafe extern "C" fn ftime(tp: *mut timeb) -> c_int {
    let tp_mut = unsafe { &mut *tp };

    let mut tv = timeval::default();
    let mut tz = timezone::default();
    if unsafe { gettimeofday(&mut tv, &mut tz) } < 0 {
        return -1;
    }

    tp_mut.time = tv.tv_sec;
    tp_mut.millitm = (tv.tv_usec / 1000) as c_ushort;
    tp_mut.timezone = tz.tz_minuteswest as c_short;
    tp_mut.dstflag = tz.tz_dsttime as c_short;

    0
}
