//! `sys/un.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_un.h.html>.

use crate::{header::sys_socket::sa_family_t, platform::types::c_char};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_un.h.html>.
#[repr(C)]
pub struct sockaddr_un {
    pub sun_family: sa_family_t,
    pub sun_path: [c_char; 108],
}

impl sockaddr_un {
    pub fn path_offset(&self) -> usize {
        let base = self as *const _ as usize;
        let path = &self.sun_path as *const _ as usize;
        log::trace!("base: {:#X}, path: {:#X}", base, path);
        path - base
    }
}
