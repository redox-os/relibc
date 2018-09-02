//! sys/times.h implementation

use platform;
use platform::types::*;
use platform::{Pal, Sys};

#[repr(C)]
pub struct tms {
    tms_utime: clock_t,
    tms_stime: clock_t,
    tms_cutime: clock_t,
    tms_cstime: clock_t,
}

#[no_mangle]
pub extern "C" fn times(out: *mut tms) -> clock_t {
    Sys::times(out)
}
