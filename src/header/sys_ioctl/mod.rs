//! ioctl implementation for linux

use crate::platform::types::{c_char, c_ushort};

// This is used from sgtty
#[repr(C)]
pub struct sgttyb {
    sg_ispeed: c_char,
    sg_ospeed: c_char,
    sg_erase: c_char,
    sg_kill: c_char,
    sg_flags: c_ushort,
}

#[cfg(target_os = "linux")]
pub use self::linux::*;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "redox")]
pub use self::redox::*;

#[cfg(target_os = "redox")]
pub mod redox;
