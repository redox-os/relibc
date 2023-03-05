//! sched.h implementation for Redox, following https://pubs.opengroup.org/onlinepubs/7908799/xsh/sched.h.html

use crate::platform::{Pal, Sys, types::*};
use crate::header::time::timespec;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct sched_param {
    pub sched_priority: c_int,
}

pub const SCHED_FIFO: c_int = 0;
pub const SCHED_RR: c_int = 1;
pub const SCHED_OTHER: c_int = 2;

// #[no_mangle]
pub extern "C" fn sched_get_priority_max(policy: c_int) -> c_int {
    todo!()
}
// #[no_mangle]
pub extern "C" fn sched_get_priority_min(policy: c_int) -> c_int {
    todo!()
}
// #[no_mangle]
pub extern "C" fn sched_getparam(pid: pid_t, param: *mut sched_param) -> c_int {
    todo!()
}
// #[no_mangle]
pub extern "C" fn sched_rr_get_interval(pid: pid_t, time: *const timespec) -> c_int {
    todo!()
}
// #[no_mangle]
pub extern "C" fn sched_setparam(pid: pid_t, param: *const sched_param) -> c_int {
    todo!()
}
// #[no_mangle]
pub extern "C" fn sched_setscheduler(pid: pid_t, policy: c_int, param: *const sched_param) -> c_int {
    todo!()
}
#[no_mangle]
pub extern "C" fn sched_yield() -> c_int {
    Sys::sched_yield()
}
