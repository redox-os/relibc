use crate::{header::sys_socket::sa_family_t, platform::types::*};

#[repr(C)]
pub struct sockaddr_un {
    pub sun_family: sa_family_t,
    pub sun_path: [c_char; 108],
}

impl sockaddr_un {
    pub fn path_offset(&self) -> usize {
        let base = self as *const _ as usize;
        let path = &self.sun_path as *const _ as usize;
        trace!("base: {:#X}, path: {:#X}", base, path);
        path - base
    }
}
