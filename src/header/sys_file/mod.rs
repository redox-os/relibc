//! sys/file.h implementation

use crate::platform::{types::*, Pal, Sys};

pub const LOCK_SH: usize = 1;
pub const LOCK_EX: usize = 2;
pub const LOCK_NB: usize = 4;
pub const LOCK_UN: usize = 8;

pub const L_SET: usize = 0;
pub const L_INCR: usize = 1;
pub const L_XTND: usize = 2;

#[no_mangle]
pub extern "C" fn flock(fd: c_int, operation: c_int) -> c_int {
    Sys::flock(fd, operation)
}
