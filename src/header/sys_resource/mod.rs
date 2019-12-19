//! sys/resource.h implementation for Redox, following
//! http://pubs.opengroup.org/onlinepubs/7908799/xsh/sysresource.h.html

use crate::{
    header::sys_time::timeval,
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

pub const RLIMIT_CPU: u64 = 0;
pub const RLIMIT_FSIZE: u64 = 1;
pub const RLIMIT_DATA: u64 = 2;
pub const RLIMIT_STACK: u64 = 3;
pub const RLIMIT_CORE: u64 = 4;
pub const RLIMIT_RSS: u64 = 5;
pub const RLIMIT_NPROC: u64 = 6;
pub const RLIMIT_NOFILE: u64 = 7;
pub const RLIMIT_MEMLOCK: u64 = 8;
pub const RLIMIT_AS: u64 = 9;
pub const RLIMIT_LOCKS: u64 = 10;
pub const RLIMIT_SIGPENDING: u64 = 11;
pub const RLIMIT_MSGQUEUE: u64 = 12;
pub const RLIMIT_NICE: u64 = 13;
pub const RLIMIT_RTPRIO: u64 = 14;
pub const RLIMIT_NLIMITS: u64 = 15;

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

// #[no_mangle]
// pub unsafe extern "C" fn getpriority(which: c_int, who: id_t) -> c_int {
//     unimplemented!();
// }

#[no_mangle]
pub unsafe extern "C" fn getrlimit(resource: c_int, rlp: *mut rlimit) -> c_int {
    Sys::getrlimit(resource, rlp)
}

// #[no_mangle]
// pub unsafe extern "C" fn getrusage(who: c_int, r_usage: *mut rusage) -> c_int {
//     // Sys::getrusage(who, r_usage)
//     unimplemented!();
// }
//
// #[no_mangle]
// pub unsafe extern "C" fn setpriority(which: c_int, who: id_t, nice: c_int) -> c_int {
//     unimplemented!();
// }
//
// #[no_mangle]
// pub unsafe extern "C" fn setrlimit(resource: c_int, rlp: *const rlimit) -> c_int {
//     unimplemented!();
// }
