//! sys/resource.h implementation for Redox, following
//! http://pubs.opengroup.org/onlinepubs/7908799/xsh/sysresource.h.html

#![no_std]

extern crate platform;
extern crate sys_time;

use platform::types::*;
use sys_time::timeval;

pub const RUSAGE_SELF: c_int = 0;
pub const RUSAGE_CHILDREN: c_int = -1;
pub const RUSAGE_BOTH: c_int = -2;
pub const RUSAGE_THREAD: c_int = 1;

type rlim_t = u64;

#[repr(C)]
pub struct rlimit {
    pub rlim_cur: rlim_t,
    pub rlim_max: rlim_t,
}

#[repr(C)]
pub struct rusage {
    pub ru_utime: timeval,
    pub ru_stime: timeval,
}

#[no_mangle]
pub unsafe extern "C" fn getpriority(which: c_int, who: id_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn getrlimit(resource: c_int, rlp: *mut rlimit) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn getrusage(who: c_int, r_usage: *mut rusage) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn setpriority(which: c_int, who: id_t, nice: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn setrlimit(resource: c_int, rlp: *const rlimit) -> c_int {
    unimplemented!();
}
