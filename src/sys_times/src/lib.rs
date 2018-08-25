//! sys/times.h implementation

#![no_std]

extern crate platform;

use platform::{Pal, Sys};
use platform::types::*;

#[repr(C)]
pub struct tms {
    tms_utime: clock_t,
    tms_stime: clock_t,
    tms_cutime: clock_t,
    tms_cstime: clock_t,
}

#[no_mangle]
pub extern "C" fn times(out: *mut tms) -> clock_t {
    Sys::times(out as *mut platform::types::tms)
}
