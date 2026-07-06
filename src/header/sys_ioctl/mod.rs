//! `sys/ioctl.h` implementation.
//!
//! Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/ioctl.2.html>.

use crate::{
    error::ResultExt,
    platform::types::{c_char, c_int, c_ulong, c_ushort, c_void},
};

pub mod constants;

pub use constants::*;

// This is used from sgtty
#[repr(C)]
pub struct sgttyb {
    sg_ispeed: c_char,
    sg_ospeed: c_char,
    sg_erase: c_char,
    sg_kill: c_char,
    sg_flags: c_ushort,
}

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/ioctl.2.html>.
///
/// Manipulates the underlying device parameters of special files.
///
/// Upon success, usually returns `0`. A few operations use the return value
/// as an output parameter and returns a nonnegative value on success. Upon
/// failure, `-1` is returned and errno set to indicate the error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ioctl(fd: c_int, request: c_ulong, out: *mut c_void) -> c_int {
    // TODO: Somehow support varargs to syscall??
    #[cfg(target_os = "linux")]
    unsafe {
        crate::platform::Sys::ioctl(fd, request, out).or_minus_one_errno()
    }
    #[cfg(target_os = "redox")]
    unsafe { self::redox::ioctl_inner(fd, request, out) }.or_minus_one_errno()
}

#[cfg(target_os = "linux")]
pub use self::linux::*;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "redox")]
pub use self::redox::*;

#[cfg(target_os = "redox")]
pub mod redox;
