//! float.h implementation for Redox, following
//! http://pubs.opengroup.org/onlinepubs/7908799/xsh/float.h.html

#![no_std]

extern crate fenv;
extern crate platform;

use platform::types::*;
use fenv::{fegetround, FE_TONEAREST};

pub const FLT_RADIX: c_int = 2;

pub unsafe extern "C" fn flt_rounds() -> c_int {
    match fegetround() {
        FE_TONEAREST => 1,
        _ => -1,
    }
}
