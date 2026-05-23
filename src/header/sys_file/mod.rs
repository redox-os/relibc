//! `sys/file.h` implementation.
//!
//! Non-POSIX, see <https://man7.org/linux/man-pages/man2/flock.2.html>.

use crate::{
    error::ResultExt,
    platform::{Pal, Sys, types::c_int},
};

/// Place a shared lock.
pub const LOCK_SH: c_int = 1;
/// Place an exclusive lock.
pub const LOCK_EX: c_int = 2;
/// To make a nonblocking request, include `LOCK_NB` (by ORing) with any of
/// `LOCK_SH`, `LOCK_EX` and `LOCK_UN`.
pub const LOCK_NB: c_int = 4;
/// Remove an existing lock held by this process.
pub const LOCK_UN: c_int = 8;

pub const L_SET: c_int = 0;
pub const L_INCR: c_int = 1;
pub const L_XTND: c_int = 2;

/// See <https://man7.org/linux/man-pages/man2/flock.2.html>.
///
/// Apply or remove an advisory lock on an open file.
#[unsafe(no_mangle)]
pub extern "C" fn flock(fd: c_int, operation: c_int) -> c_int {
    Sys::flock(fd, operation).map(|()| 0).or_minus_one_errno()
}
