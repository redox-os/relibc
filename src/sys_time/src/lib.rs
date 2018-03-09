//! sys/time implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/systime.h.html

#![no_std]

extern crate platform;

use platform::types::*;

#[repr(C)]
pub struct timeval {
    pub tv_sec: time_t,
    pub tv_usec: suseconds_t,
}

#[repr(C)]
pub struct itimerval {
    pub it_interval: timeval,
    pub it_value: timeval,
}

#[repr(C)]
pub struct fd_set {
    pub fds_bits: [c_long; 16usize],
}

pub extern "C" fn getitimer(which: c_int, value: *mut itimerval) -> c_int {
    unimplemented!();
}

pub extern "C" fn setitimer(
    which: c_int,
    value: *const itimerval,
    ovalue: *mut itimerval,
) -> c_int {
    unimplemented!();
}

pub extern "C" fn gettimeofday(tp: *mut timeval, tzp: *const c_void) -> c_int {
    unimplemented!();
}

pub extern "C" fn select(
    nfds: c_int,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    errorfds: *mut fd_set,
    timeout: *mut timeval,
) -> c_int {
    unimplemented!();
}

pub extern "C" fn utimes(path: *const c_char, times: [timeval; 2]) -> c_int {
    unimplemented!();
}

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
