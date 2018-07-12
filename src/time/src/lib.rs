//! time implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/time.h.html

#![no_std]
#![feature(const_fn)]

extern crate errno;
extern crate platform;

pub mod constants;
mod helpers;

use constants::*;
use core::mem::transmute;
use errno::EIO;
use helpers::*;
use platform::types::*;

/*
 *#[repr(C)]
 *pub struct timespec {
 *    pub tv_sec: time_t,
 *    pub tv_nsec: c_long,
 *}
 */

#[repr(C)]
pub struct tm {
    pub tm_sec: c_int,
    pub tm_min: c_int,
    pub tm_hour: c_int,
    pub tm_mday: c_int,
    pub tm_mon: c_int,
    pub tm_year: c_int,
    pub tm_wday: c_int,
    pub tm_yday: c_int,
    pub tm_isdst: c_int,
    pub tm_gmtoff: c_long,
    pub tm_zone: *const c_char,
}

unsafe impl Sync for tm {}

// The C Standard says that localtime and gmtime return the same pointer.
static mut TM: tm = tm {
    tm_sec: 0,
    tm_min: 0,
    tm_hour: 0,
    tm_mday: 0,
    tm_mon: 0,
    tm_year: 0,
    tm_wday: 0,
    tm_yday: 0,
    tm_isdst: 0,
    tm_gmtoff: 0,
    tm_zone: UTC,
};

// The C Standard says that ctime and asctime return the same pointer.
static mut ASCTIME: [c_char; 26] = [0; 26];

#[repr(C)]
pub struct itimerspec {
    pub it_interval: timespec,
    pub it_value: timespec,
}

pub struct sigevent;

#[no_mangle]
pub extern "C" fn asctime(timeptr: *const tm) -> *mut c_char {
    unsafe { asctime_r(timeptr, transmute::<&mut _, *mut c_char>(&mut ASCTIME)) }
}

#[no_mangle]
pub extern "C" fn asctime_r(tm: *const tm, buf: *mut c_char) -> *mut c_char {
    let tm = unsafe { &*tm };
    let result = core::fmt::write(
        &mut platform::UnsafeStringWriter(buf as *mut u8),
        format_args!(
            "{:.3} {:.3}{:3} {:02}:{:02}:{:02} {}\n",
            DAY_NAMES[tm.tm_wday as usize],
            MON_NAMES[tm.tm_mon as usize],
            tm.tm_mday as usize,
            tm.tm_hour as usize,
            tm.tm_min as usize,
            tm.tm_sec as usize,
            (1900 + tm.tm_year)
        ),
    );
    match result {
        Ok(_) => buf,
        Err(_) => {
            unsafe { platform::errno = EIO };
            core::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn clock() -> clock_t {
    let mut ts: timespec = unsafe { core::mem::uninitialized() };

    if clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &mut ts) != 0 {
        return -1;
    }

    if ts.tv_sec > time_t::max_value() / CLOCKS_PER_SEC
        || ts.tv_nsec / (1_000_000_000 / CLOCKS_PER_SEC)
            > time_t::max_value() - CLOCKS_PER_SEC * ts.tv_sec
    {
        return -1;
    }

    return ts.tv_sec * CLOCKS_PER_SEC + ts.tv_nsec / (1_000_000_000 / CLOCKS_PER_SEC);
}

// #[no_mangle]
pub extern "C" fn clock_getres(clock_id: clockid_t, res: *mut timespec) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn clock_gettime(clock_id: clockid_t, tp: *mut timespec) -> c_int {
    platform::clock_gettime(clock_id, tp)
}

// #[no_mangle]
pub extern "C" fn clock_settime(clock_id: clockid_t, tp: *const timespec) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn ctime(clock: *const time_t) -> *mut c_char {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn ctime_r(clock: *const time_t, buf: *mut c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn difftime(time1: time_t, time0: time_t) -> c_double {
    (time1 - time0) as c_double
}

// #[no_mangle]
pub extern "C" fn getdate(string: *const c_char) -> tm {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn gmtime(timer: *const time_t) -> *mut tm {
    unsafe { gmtime_r(timer, &mut TM) }
}

#[no_mangle]
pub extern "C" fn gmtime_r(clock: *const time_t, result: *mut tm) -> *mut tm {
    let (mut days, mut rem): (c_long, c_long);
    let mut weekday: c_int;
    let lcltime = unsafe { *clock };

    days = lcltime / SECSPERDAY + EPOCH_ADJUSTMENT_DAYS;
    rem = lcltime % SECSPERDAY;
    if rem < 0 {
        rem += SECSPERDAY;
        days -= 1;
    }
    unsafe {
        (*result).tm_hour = (rem / SECSPERHOUR) as c_int;
        rem %= SECSPERHOUR;
        (*result).tm_min = (rem / SECSPERMIN) as c_int;
        (*result).tm_sec = (rem % SECSPERMIN) as c_int;
    }

    weekday = ((ADJUSTED_EPOCH_WDAY + days) % DAYSPERWEEK as c_long) as c_int;
    if weekday < 0 {
        weekday += DAYSPERWEEK;
    }
    unsafe { (*result).tm_wday = weekday };

    let (year, month, day, yearday) = civil_from_days(days);
    unsafe {
        (*result).tm_yday = yearday;
        (*result).tm_year = year - YEAR_BASE;
        (*result).tm_mon = month;
        (*result).tm_mday = day;

        (*result).tm_isdst = 0;
        (*result).tm_gmtoff = 0;
        (*result).tm_zone = UTC;
    }
    result
}

// #[no_mangle]
pub extern "C" fn localtime(timer: *const time_t) -> *mut tm {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn localtime_r(clock: *const time_t, result: *mut tm) -> *mut tm {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn mktime(timeptr: *mut tm) -> time_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> c_int {
    platform::nanosleep(rqtp, rmtp)
}

// #[no_mangle]
pub extern "C" fn strftime(
    s: *mut c_char,
    maxsize: usize,
    format: *const c_char,
    timptr: *const tm,
) -> usize {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn strptime(buf: *const c_char, format: *const c_char, tm: *mut tm) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn time(tloc: *mut time_t) -> time_t {
    let mut ts: timespec = Default::default();
    platform::clock_gettime(CLOCK_REALTIME, &mut ts);
    unsafe {
        if !tloc.is_null() {
            *tloc = ts.tv_sec
        };
    }
    ts.tv_sec
}

// #[no_mangle]
pub extern "C" fn timer_create(
    clock_id: clockid_t,
    evp: *mut sigevent,
    timerid: *mut timer_t,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn timer_delete(timerid: timer_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn tzset() {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn timer_settime(
    timerid: timer_t,
    flags: c_int,
    value: *const itimerspec,
    ovalue: *mut itimerspec,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn timer_gettime(timerid: timer_t, value: *mut itimerspec) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn timer_getoverrun(timerid: timer_t) -> c_int {
    unimplemented!();
}

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
