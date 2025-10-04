//! float.h implementation for Redox, following
//! http://pubs.opengroup.org/onlinepubs/7908799/xsh/float.h.html

use crate::{
    header::_fenv::{FE_TONEAREST, fegetround},
    platform::types::*,
};

pub const FLT_RADIX: c_int = 2;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn flt_rounds() -> c_int {
    match fegetround() {
        FE_TONEAREST => 1,
        _ => -1,
    }
}
