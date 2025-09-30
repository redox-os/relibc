//! sys/resource.h implementation for Redox, following
//! http://pubs.opengroup.org/onlinepubs/7908799/xsh/sysresource.h.html

use crate::{
    error::ResultExt,
    header::sys_time::timeval,
    out::Out,
    platform::{types::*, Pal, Sys},
};

// Exported in bits file
// const RUSAGE_SELF: c_int = 0;
// const RUSAGE_CHILDREN: c_int = -1;
// const RUSAGE_BOTH: c_int = -2;
// const RUSAGE_THREAD: c_int = 1;

pub const RLIM_INFINITY: u64 = 0xFFFF_FFFF_FFFF_FFFF;
pub const RLIM_SAVED_CUR: u64 = RLIM_INFINITY;
pub const RLIM_SAVED_MAX: u64 = RLIM_INFINITY;

pub const RLIMIT_CPU: c_int = 0;
pub const RLIMIT_FSIZE: c_int = 1;
pub const RLIMIT_DATA: c_int = 2;
pub const RLIMIT_STACK: c_int = 3;
pub const RLIMIT_CORE: c_int = 4;
pub const RLIMIT_RSS: c_int = 5;
pub const RLIMIT_NPROC: c_int = 6;
pub const RLIMIT_NOFILE: c_int = 7;
pub const RLIMIT_MEMLOCK: c_int = 8;
pub const RLIMIT_AS: c_int = 9;
pub const RLIMIT_LOCKS: c_int = 10;
pub const RLIMIT_SIGPENDING: c_int = 11;
pub const RLIMIT_MSGQUEUE: c_int = 12;
pub const RLIMIT_NICE: c_int = 13;
pub const RLIMIT_RTPRIO: c_int = 14;
pub const RLIMIT_NLIMITS: c_int = 15;

pub type rlim_t = u64;

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

pub const PRIO_PROCESS: c_int = 0;
pub const PRIO_PGRP: c_int = 1;
pub const PRIO_USER: c_int = 2;

#[no_mangle]
pub unsafe extern "C" fn getpriority(which: c_int, who: id_t) -> c_int {
    let r = Sys::getpriority(which, who).or_minus_one_errno();
    if r < 0 {
        return r;
    }
    20 - r
}

#[no_mangle]
pub unsafe extern "C" fn setpriority(which: c_int, who: id_t, nice: c_int) -> c_int {
    Sys::setpriority(which, who, nice)
        .map(|()| 0)
        .or_minus_one_errno()
}

#[no_mangle]
pub unsafe extern "C" fn getrlimit(resource: c_int, rlp: *mut rlimit) -> c_int {
    let rlp = Out::nonnull(rlp);

    Sys::getrlimit(resource, rlp)
        .map(|()| 0)
        .or_minus_one_errno()
}

#[no_mangle]
pub unsafe extern "C" fn setrlimit(resource: c_int, rlp: *const rlimit) -> c_int {
    Sys::setrlimit(resource, rlp)
        .map(|()| 0)
        .or_minus_one_errno()
}

#[no_mangle]
pub unsafe extern "C" fn getrusage(who: c_int, r_usage: *mut rusage) -> c_int {
    Sys::getrusage(who, Out::nonnull(r_usage))
        .map(|()| 0)
        .or_minus_one_errno()
}
