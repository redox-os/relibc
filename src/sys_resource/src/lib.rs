//! sys/resource.h implementation for Redox, following
//! http://pubs.opengroup.org/onlinepubs/7908799/xsh/sysresource.h.html

#![no_std]

extern crate platform;
extern crate sys_time;

use platform::types::*;
use sys_time::timeval;

// Exported in bits file
const RUSAGE_SELF: c_int = 0;
const RUSAGE_CHILDREN: c_int = -1;
const RUSAGE_BOTH: c_int = -2;
const RUSAGE_THREAD: c_int = 1;

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
    pub ru_maxrss: c_long,
    pub ru_ixrss: c_long,
    pub ru_idrss: c_long,
    pub ru_isrss: c_long,
    pub ru_minflt: c_long,
    pub ru_majflt: c_long,
    pub ru_nswap: c_long,
    pub ru_inblock: c_long,
    pub ru_oublock: c_long,
    pub ru_msgsnd: c_long,
    pub ru_msgrcv: c_long,
    pub ru_nsignals: c_long,
    pub ru_nvcsw: c_long,
    pub ru_nivcsw: c_long,
}

// #[no_mangle]
pub unsafe extern "C" fn getpriority(which: c_int, who: id_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub unsafe extern "C" fn getrlimit(resource: c_int, rlp: *mut rlimit) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn getrusage(who: c_int, r_usage: *mut rusage) -> c_int {
    platform::getrusage(who, r_usage as *mut platform::types::rusage)
}

// #[no_mangle]
pub unsafe extern "C" fn setpriority(which: c_int, who: id_t, nice: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub unsafe extern "C" fn setrlimit(resource: c_int, rlp: *const rlimit) -> c_int {
    unimplemented!();
}
