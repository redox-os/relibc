//! `sched.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sched.h.html>.

use crate::{
    error::ResultExt,
    header::time::timespec,
    platform::{
        Pal, Sys,
        types::{c_int, pid_t},
    },
};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sched.h.html>.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct sched_param {
    pub sched_priority: c_int,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sched.h.html>.
pub const SCHED_FIFO: c_int = 0;
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sched.h.html>.
pub const SCHED_RR: c_int = 1;
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sched.h.html>.
pub const SCHED_OTHER: c_int = 2;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sched_get_priority_max.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn sched_get_priority_max(policy: c_int) -> c_int {
    todo!()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sched_get_priority_max.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn sched_get_priority_min(policy: c_int) -> c_int {
    todo!()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sched_getparam.html>.
// #[unsafe(no_mangle)]
pub unsafe extern "C" fn sched_getparam(pid: pid_t, param: *mut sched_param) -> c_int {
    todo!()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sched_rr_get_interval.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn sched_rr_get_interval(pid: pid_t, time: *const timespec) -> c_int {
    todo!()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sched_setparam.html>.
// #[unsafe(no_mangle)]
pub unsafe extern "C" fn sched_setparam(pid: pid_t, param: *const sched_param) -> c_int {
    todo!()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sched_setscheduler.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn sched_setscheduler(
    pid: pid_t,
    policy: c_int,
    param: *const sched_param,
) -> c_int {
    todo!()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sched_yield.html>.
#[unsafe(no_mangle)]
pub extern "C" fn sched_yield() -> c_int {
    Sys::sched_yield().map(|()| 0).or_minus_one_errno()
}
