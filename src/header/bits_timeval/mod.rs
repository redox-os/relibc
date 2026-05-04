use crate::platform::types::{suseconds_t, time_t};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_select.h.html>.
///
/// Note that the `timeval` struct was specified for
/// [`sys/time.h`](crate::header::sys_time) in the Open Group Base
/// Specifications Issue 7 and prior, see
/// <https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_time.h.html>.
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Default)]
pub struct timeval {
    pub tv_sec: time_t,
    pub tv_usec: suseconds_t,
}
