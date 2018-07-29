#![no_std]

extern crate platform;
extern crate sys_socket;

use platform::types::*;
use sys_socket::sa_family_t;

#[repr(C)]
pub struct sockaddr_un {
    sun_family: sa_family_t,
    sun_path: [c_char; 108],
}
