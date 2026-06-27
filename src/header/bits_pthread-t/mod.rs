//! `pthread_t` from `sys/types.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_types.h.html>.

use crate::platform::types::c_void;

/// Used to identify a thread.
pub type pthread_t = *mut c_void;
