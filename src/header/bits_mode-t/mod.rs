#[cfg(target_os = "linux")]
use crate::platform::types::c_uint;

#[cfg(not(target_os = "linux"))]
use crate::platform::types::c_int;

/// Used for some file attributes.
#[cfg(target_os = "linux")]
pub type mode_t = c_uint;
/// Used for some file attributes.
#[cfg(not(target_os = "linux"))]
pub type mode_t = c_int;
