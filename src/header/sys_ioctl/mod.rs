//! ioctl implementation for linux

use crate::platform::types::*;

// This is used from sgtty
#[repr(C)]
pub struct sgttyb {
    sg_ispeed: c_char,
    sg_ospeed: c_char,
    sg_erase: c_char,
    sg_kill: c_char,
    sg_flags: c_ushort,
}

#[repr(C)]
#[derive(Default)]
pub struct winsize {
    ws_row: c_ushort,
    ws_col: c_ushort,
    ws_xpixel: c_ushort,
    ws_ypixel: c_ushort,
}

pub use self::sys::*;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;
