//! `float.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/float.h.html>.

use crate::{
    header::_fenv::{FE_TONEAREST, fegetround},
    platform::types::c_int,
};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/float.h.html>.
pub const FLT_RADIX: c_int = 2;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/float.h.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn flt_rounds() -> c_int {
    match unsafe { fegetround() } {
        FE_TONEAREST => 1,
        _ => -1,
    }
}
