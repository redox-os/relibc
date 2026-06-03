//! `sys/timeb.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/009695399/basedefs/sys/timeb.h.html>.
//!
//! # Deprecation
//! The `ftime()` function was marked as legacy in the Open Group Base
//! Specifications Issue 6, and the entire `sys/timeb.h` header was removed in
//! Issue 7.

#[allow(deprecated)]
use crate::header::sys_time::gettimeofday;
use crate::{
    header::{sys_select::timeval, sys_time::timezone},
    out::Out,
    platform::types::{c_int, c_short, c_ushort, time_t},
};

/// See <https://pubs.opengroup.org/onlinepubs/009695399/basedefs/sys/timeb.h.html>.
#[repr(C)]
#[derive(Default)]
pub struct timeb {
    /// The seconds portion of the current time.
    pub time: time_t,
    /// The milliseconds portion of the current time.
    pub millitm: c_ushort,
    /// The local timezone in minutes west of Greenwich.
    pub timezone: c_short,
    /// TRUE if Daylight Savings Time is in effect.
    pub dstflag: c_short,
}

/// See <https://pubs.opengroup.org/onlinepubs/009695399/functions/ftime.html>.
///
/// Sets the `time` and `millitm` members of the `timeb` structure pointed to
/// by `tp` to contain the seconds and milliseconds portions, respectively,
/// of the current time in seconds since the Epoch.
///
/// # Safety
/// The caller must ensure that `tp` is convertible to an [`Out<timeb>`].
///
/// # Deprecation
/// The `ftime()` function was marked as legacy in the Open Group Base
/// Specifications Issue 6, and the entire `sys/timeb.h` header was removed in
/// Issue 7.
#[allow(deprecated)]
#[deprecated]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ftime(tp: *mut timeb) -> c_int {
    // SAFETY: the caller is required to ensure that the pointer is valid.
    let mut tp_out = unsafe { Out::nonnull(tp) };

    let mut tv = timeval::default();
    let mut tz = timezone::default();

    // SAFETY: tv and tz are created above, and thus will coerce to valid
    // pointers.
    if unsafe {
        #[allow(deprecated)]
        gettimeofday(&raw mut tv, &raw mut tz)
    } < 0
    {
        return -1;
    }

    #[allow(deprecated)]
    tp_out.write(timeb {
        time: tv.tv_sec,
        millitm: (tv.tv_usec / 1000) as c_ushort,
        timezone: tz.tz_minuteswest as c_short,
        dstflag: tz.tz_dsttime as c_short,
    });

    0
}
