//! sys/file.h implementation

#![no_std]

extern crate platform;

use platform::types::*;

pub const LOCK_SH: usize = 1;
pub const LOCK_EX: usize = 1 << 1;
pub const LOCK_NB: usize = 1 << 2;
pub const LOCK_UN: usize = 1 << 3;

pub const L_SET: usize = 0;
pub const L_INCR: usize = 1;
pub const L_XTND: usize = 2;

#[no_mangle]
pub extern "C" fn flock(fd: c_int, operation: c_int) -> c_int {
    platform::flock(fd, operation)
}
