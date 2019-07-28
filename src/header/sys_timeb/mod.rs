//! sys/time implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/systime.h.html

use crate::{
    header::sys_time::{gettimeofday, timeval, timezone},
    platform::types::*,
};

#[repr(C)]
#[derive(Default)]
pub struct timeb {
    pub time: time_t,
    pub millitm: c_ushort,
    pub timezone: c_short,
    pub dstflag: c_short,
}

#[no_mangle]
pub unsafe extern "C" fn ftime(tp: *mut timeb) -> c_int {
    let mut tv = timeval::default();
    let mut tz = timezone::default();
    if gettimeofday(&mut tv, &mut tz) < 0 {
        return -1;
    }

    (*tp).time = tv.tv_sec;
    (*tp).millitm = (tv.tv_usec / 1000) as c_ushort;
    (*tp).timezone = tz.tz_minuteswest as c_short;
    (*tp).dstflag = tz.tz_dsttime as c_short;

    0
}
