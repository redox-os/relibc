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

#[repr(C)]
#[derive(Default)]
pub struct winsize {
    ws_row: c_ushort,
    ws_col: c_ushort,
    ws_xpixel: c_ushort,
    ws_ypixel: c_ushort,
}

impl winsize {
    pub fn get_row_col(&self) -> (c_ushort, c_ushort) {
        (self.ws_row, self.ws_col)
    }
}

#[cfg(target_os = "linux")]
pub use self::linux::*;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "redox")]
pub use self::redox::*;

#[cfg(target_os = "redox")]
pub mod redox;
