#[cfg(target_os = "linux")]
use crate::platform::types::c_long;

#[cfg(not(target_os = "linux"))]
use crate::platform::types::c_int;

#[cfg(target_os = "linux")]
#[allow(non_camel_case_types)]
/// Used for time in microseconds.
pub type suseconds_t = c_long;
#[cfg(not(target_os = "linux"))]
#[allow(non_camel_case_types)]
/// Used for time in microseconds.
pub type suseconds_t = c_int;
