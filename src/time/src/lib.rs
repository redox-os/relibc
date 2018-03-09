//! time implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/time.h.html

#![no_std]

extern crate platform;

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

#[repr(C)]
pub struct itimerspec {
    pub it_interval: timespec,
    pub it_value: timespec,
}

pub struct sigevent;

#[no_mangle]
pub extern "C" fn asctime(timeptr: *const tm) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn asctime_r(tm: *const tm, buf: *mut c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn clock() -> clock_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn clock_getres(clock_id: clockid_t, res: *mut timespec) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn clock_gettime(clock_id: clockid_t, tp: *mut timespec) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn clock_settime(clock_id: clockid_t, tp: *const timespec) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ctime(clock: *const time_t) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ctime_r(clock: *const time_t, buf: *mut c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn difftime(time1: time_t, time0: time_t) -> f64 {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getdate(string: *const c_char) -> tm {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn gmtime(timer: *const time_t) -> *mut tm {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn gmtime_r(clock: *const time_t, result: *mut tm) -> *mut tm {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn localtime(timer: *const time_t) -> *mut tm {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn localtime_r(clock: *const time_t, result: *mut tm) -> *mut tm {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mktime(timeptr: *mut tm) -> time_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> c_int {
    platform::nanosleep(rqtp, rmtp)
}

#[no_mangle]
pub extern "C" fn strftime(
    s: *mut c_char,
    maxsize: usize,
    format: *const c_char,
    timptr: *const tm,
) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strptime(buf: *const c_char, format: *const c_char, tm: *mut tm) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn time(tloc: *mut time_t) -> time_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn timer_create(
    clock_id: clockid_t,
    evp: *mut sigevent,
    timerid: *mut timer_t,
) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn timer_delete(timerid: timer_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn tzset() {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn timer_settime(
    timerid: timer_t,
    flags: c_int,
    value: *const itimerspec,
    ovalue: *mut itimerspec,
) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn timer_gettime(timerid: timer_t, value: *mut itimerspec) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn timer_getoverrun(timerid: timer_t) -> c_int {
    unimplemented!();
}

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
