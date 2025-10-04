//! `sys/file.h` implementation.
//!
//! Non-POSIX, see <https://man7.org/linux/man-pages/man2/flock.2.html>.

use crate::{
    error::ResultExt,
    platform::{Pal, Sys, types::*},
};

pub const LOCK_SH: usize = 1;
pub const LOCK_EX: usize = 2;
pub const LOCK_NB: usize = 4;
pub const LOCK_UN: usize = 8;

pub const L_SET: usize = 0;
pub const L_INCR: usize = 1;
pub const L_XTND: usize = 2;

/// See <https://man7.org/linux/man-pages/man2/flock.2.html>.
#[unsafe(no_mangle)]
pub extern "C" fn flock(fd: c_int, operation: c_int) -> c_int {
    Sys::flock(fd, operation).map(|()| 0).or_minus_one_errno()
}
