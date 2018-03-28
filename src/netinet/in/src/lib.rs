#![no_std]

#![allow(non_camel_case_types)]

extern crate platform;
extern crate sys_socket;

use platform::types::*;
use sys_socket::sa_family_t;

pub type in_addr_t = u32;
pub type in_port_t = u16;

#[repr(C)]
#[derive(Debug)]
pub struct in_addr {
    pub s_addr: in_addr_t
}

#[repr(C)]
pub struct in6_addr {
    pub s6_addr: [u8; 16]
}

#[repr(C)]
pub struct sockaddr_in {
    pub sa_family: sa_family_t,
    pub sin_port: in_port_t,
    pub sin_addr: in_addr
}

#[repr(C)]
pub struct sockaddr_in6 {
    pub sin6_family: sa_family_t,
    pub sin6_port: in_port_t,
    pub sin6_flowinfo: u32,
    pub sin6_addr: in6_addr,
    pub sin6_scope_id: u32
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
