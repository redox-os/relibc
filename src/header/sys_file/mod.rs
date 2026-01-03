//! `sys/file.h` implementation.
//!
//! Non-POSIX, see <https://man7.org/linux/man-pages/man2/flock.2.html>.

use crate::{
    error::ResultExt,
    platform::{Pal, Sys, types::c_int},
};

pub const LOCK_SH: c_int = 1;
pub const LOCK_EX: c_int = 2;
pub const LOCK_NB: c_int = 4;
pub const LOCK_UN: c_int = 8;

pub const L_SET: c_int = 0;
pub const L_INCR: c_int = 1;
pub const L_XTND: c_int = 2;

/// See <https://man7.org/linux/man-pages/man2/flock.2.html>.
#[unsafe(no_mangle)]
pub extern "C" fn flock(fd: c_int, operation: c_int) -> c_int {
    Sys::flock(fd, operation).map(|()| 0).or_minus_one_errno()
}
