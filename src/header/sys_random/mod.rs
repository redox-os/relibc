//! `sys/random.h` implementation.
//!
//! Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrandom.2.html>.

use core::slice;

use crate::{
    error::ResultExt,
    platform::{
        Pal, Sys,
        types::{c_uint, c_void, size_t, ssize_t},
    },
};

/// See <https://www.man7.org/linux/man-pages/man2/getrandom.2.html>.
pub const GRND_NONBLOCK: c_uint = 1;
/// See <https://www.man7.org/linux/man-pages/man2/getrandom.2.html>.
pub const GRND_RANDOM: c_uint = 2;

/// See <https://www.man7.org/linux/man-pages/man2/getrandom.2.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getrandom(buf: *mut c_void, buflen: size_t, flags: c_uint) -> ssize_t {
    Sys::getrandom(
        unsafe { slice::from_raw_parts_mut(buf as *mut u8, buflen as usize) },
        flags,
    )
    .map(|read| read as ssize_t)
    .or_minus_one_errno()
}
