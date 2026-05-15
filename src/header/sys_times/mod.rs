//! `sys/times.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_times.h.html>.

use crate::platform::types::clock_t;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_times.h.html>.
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct tms {
    tms_utime: clock_t,
    tms_stime: clock_t,
    tms_cutime: clock_t,
    tms_cstime: clock_t,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/times.html>.
#[unsafe(no_mangle)]
pub extern "C" fn times(out: *mut tms) -> clock_t {
    todo_panic!(0, "TODO: times not implemented");
}
