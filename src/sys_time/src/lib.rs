//! sys/time implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/systime.h.html

#![no_std]

extern crate platform;

use platform::types::*;

pub const ITIMER_REAL: c_int = 0;
pub const ITIMER_VIRTUAL: c_int = 1;
pub const ITIMER_PROF: c_int = 2;

#[repr(C)]
#[derive(Default)]
pub struct timeval {
    pub tv_sec: time_t,
    pub tv_usec: suseconds_t,
}
#[repr(C)]
pub struct timezone {
    pub tz_minuteswest: c_int,
    pub tz_dsttime: c_int,
}

#[repr(C)]
#[derive(Default)]
pub struct itimerval {
    pub it_interval: timeval,
    pub it_value: timeval,
}

#[repr(C)]
pub struct fd_set {
    pub fds_bits: [c_long; 16usize],
}

#[no_mangle]
pub extern "C" fn getitimer(which: c_int, value: *mut itimerval) -> c_int {
    platform::getitimer(which, value as *mut platform::types::itimerval)
}

#[no_mangle]
pub extern "C" fn setitimer(
    which: c_int,
    value: *const itimerval,
    ovalue: *mut itimerval,
) -> c_int {
    platform::setitimer(
        which,
        value as *const platform::types::itimerval,
        ovalue as *mut platform::types::itimerval
    )
}

#[no_mangle]
pub extern "C" fn gettimeofday(tp: *mut timeval, tzp: *mut timezone) -> c_int {
    platform::gettimeofday(tp as *mut platform::types::timeval, tzp as *mut platform::types::timezone)
}

// #[no_mangle]
pub extern "C" fn select(
    nfds: c_int,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    errorfds: *mut fd_set,
    timeout: *mut timeval,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn utimes(path: *const c_char, times: [timeval; 2]) -> c_int {
    unimplemented!();
}

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
