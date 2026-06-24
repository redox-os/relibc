//! `size_t` from `stddef.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stddef.h.html>.

use crate::platform::types::c_ulong;

/// Unsigned integer type of the result of the sizeof operator.
#[allow(non_camel_case_types)]
pub type size_t = c_ulong;
