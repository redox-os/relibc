//! float.h implementation for Redox, following
//! http://pubs.opengroup.org/onlinepubs/7908799/xsh/float.h.html

use header::fenv::{fegetround, FE_TONEAREST};
use platform;
use platform::types::*;

pub const FLT_RADIX: c_int = 2;

#[no_mangle]
pub unsafe extern "C" fn flt_rounds() -> c_int {
    match fegetround() {
        FE_TONEAREST => 1,
        _ => -1,
    }
}
