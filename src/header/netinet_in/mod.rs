#![allow(non_camel_case_types)]

use crate::{header::sys_socket::sa_family_t, platform::types::*};

pub type in_addr_t = u32;
pub type in_port_t = u16;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct in_addr {
    pub s_addr: in_addr_t,
}

#[repr(C)]
pub struct in6_addr {
    pub s6_addr: [u8; 16],
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct sockaddr_in {
    pub sin_family: sa_family_t,
    pub sin_port: in_port_t,
    pub sin_addr: in_addr,
    pub sin_zero: [c_char; 8],
}

#[repr(C)]
pub struct sockaddr_in6 {
    pub sin6_family: sa_family_t,
    pub sin6_port: in_port_t,
    pub sin6_flowinfo: u32,
    pub sin6_addr: in6_addr,
    pub sin6_scope_id: u32,
}

#[repr(C)]
pub struct ipv6_mreq {
    pub ipv6mr_multiaddr: in6_addr,
    pub ipv6mr_interface: u32,
}

// Address String Lengths
pub const INET_ADDRSTRLEN: c_int = 16;
pub const INET6_ADDRSTRLEN: c_int = 46;

// Protocol Numbers
pub const IPPROTO_IP: u8 = 0x00;
pub const IPPROTO_ICMP: u8 = 0x01;
pub const IPPROTO_TCP: u8 = 0x06;
pub const IPPROTO_UDP: u8 = 0x11;
pub const IPPROTO_IPV6: u8 = 0x29;
pub const IPPROTO_RAW: u8 = 0xff;
pub const IPPROTO_MAX: u8 = 0xff;

pub const INADDR_ANY: u32 = 0; // Can't use in_addr_t alias because cbindgen :(
pub const INADDR_BROADCAST: u32 = 0xFFFF_FFFF; // Can't use core::u32::MAX because cbindgen :(
pub const INADDR_NONE: u32 = 0xFFFF_FFFF;
pub const INADDR_LOOPBACK: u32 = 0x7F00_0001;

pub const INADDR_UNSPEC_GROUP: u32 = 0xE000_0000;
pub const INADDR_ALLHOSTS_GROUP: u32 = 0xE000_0001;
pub const INADDR_ALLRTRS_GROUP: u32 = 0xE000_0002;
pub const INADDR_MAX_LOCAL_GROUP: u32 = 0xE000_00FF;

#[no_mangle]
pub static in6addr_any: in6_addr = in6_addr {
    s6_addr: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

#[no_mangle]
pub static in6addr_loopback: in6_addr = in6_addr {
    s6_addr: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
};
