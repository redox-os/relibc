//! `sigset_t` for `signal.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/signal.h.html>.

use crate::platform::types::c_ulonglong;

/// Integer type of an object used to represent sets of signals.
#[allow(non_camel_case_types)]
pub type sigset_t = c_ulonglong;
