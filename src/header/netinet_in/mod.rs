//! `netinet/in.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.

#![allow(non_camel_case_types)]

use crate::{
    header::sys_socket::{sa_family_t, sockaddr_storage},
    platform::types::{c_char, c_int, uint16_t, uint32_t},
};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
pub type in_addr_t = u32;
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
pub type in_port_t = u16;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct in_addr {
    pub s_addr: in_addr_t,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
#[repr(C)]
pub struct in6_addr {
    pub s6_addr: [u8; 16],
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
#[repr(C)]
#[derive(Debug, Default)]
pub struct sockaddr_in {
    pub sin_family: sa_family_t,
    pub sin_port: in_port_t,
    pub sin_addr: in_addr,
    pub sin_zero: [c_char; 8],
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
#[repr(C)]
pub struct sockaddr_in6 {
    pub sin6_family: sa_family_t,
    pub sin6_port: in_port_t,
    pub sin6_flowinfo: u32,
    pub sin6_addr: in6_addr,
    pub sin6_scope_id: u32,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
#[repr(C)]
pub struct ipv6_mreq {
    pub ipv6mr_multiaddr: in6_addr,
    pub ipv6mr_interface: u32,
}

// Address String Lengths
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
pub const INET_ADDRSTRLEN: c_int = 16;
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
pub const INET6_ADDRSTRLEN: c_int = 46;

// IP options
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/ip.7.html>.
pub const IP_TOS: c_int = 1;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/ip.7.html>.
pub const IP_RECVTOS: c_int = 13;

// Protocol Numbers
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
pub const IPPROTO_IP: u8 = 0;
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
pub const IPPROTO_ICMP: u8 = 1;
/// Non-POSIX.
pub const IPPROTO_IGMP: u8 = 2;
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
pub const IPPROTO_TCP: u8 = 6;
/// Non-POSIX.
pub const IPPROTO_PUP: u8 = 12;
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
pub const IPPROTO_UDP: u8 = 17;
/// Non-POSIX.
pub const IPPROTO_IDP: u8 = 22;
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
pub const IPPROTO_IPV6: u8 = 41;
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
pub const IPPROTO_RAW: u8 = 0xff;
/// Non-POSIX.
pub const IPPROTO_MAX: u8 = 0xff;

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/ip.7.html>.
pub const IP_TTL: c_int = 2;
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
pub const IPV6_UNICAST_HOPS: c_int = 16;
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
pub const IPV6_MULTICAST_IF: c_int = 17;
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
pub const IPV6_MULTICAST_HOPS: c_int = 18;
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
pub const IPV6_MULTICAST_LOOP: c_int = 19;
/// Non-POSIX.
pub const IPV6_ADD_MEMBERSHIP: c_int = 20;
pub const IPV6_JOIN_GROUP: c_int = 20;
/// Non-POSIX.
pub const IPV6_DROP_MEMBERSHIP: c_int = 21;
pub const IPV6_LEAVE_GROUP: c_int = 21;
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
pub const IPV6_V6ONLY: c_int = 26;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/ip.7.html>.
pub const IP_MULTICAST_IF: c_int = 32;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/ip.7.html>.
pub const IP_MULTICAST_TTL: c_int = 33;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/ip.7.html>.
pub const IP_MULTICAST_LOOP: c_int = 34;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/ip.7.html>.
pub const IP_ADD_MEMBERSHIP: c_int = 35;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/ip.7.html>.
pub const IP_DROP_MEMBERSHIP: c_int = 36;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
pub const INADDR_ANY: u32 = 0; // Can't use in_addr_t alias because cbindgen :(
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
pub const INADDR_BROADCAST: u32 = 0xFFFF_FFFF; // Can't use core::u32::MAX because cbindgen :(
/// Non-POSIX.
pub const INADDR_NONE: u32 = 0xFFFF_FFFF;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/ip.7.html>.
pub const INADDR_LOOPBACK: u32 = 0x7F00_0001;
/// Non-POSIX.
pub const INADDR_UNSPEC_GROUP: u32 = 0xE000_0000;
/// Non-POSIX.
pub const INADDR_ALLHOSTS_GROUP: u32 = 0xE000_0001;
/// Non-POSIX.
pub const INADDR_ALLRTRS_GROUP: u32 = 0xE000_0002;
/// Non-POSIX.
pub const INADDR_MAX_LOCAL_GROUP: u32 = 0xE000_00FF;

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/ip.7.html>.
#[repr(C)]
pub struct ip_mreq_source {
    pub imr_multiaddr: in_addr,
    pub imr_interface: in_addr,
    pub imr_sourceaddr: in_addr,
}

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/ip.7.html>.
#[repr(C)]
pub struct ip_mreq {
    pub imr_multiaddr: in_addr,
    pub imr_interface: in_addr,
}

/// Non-POSIX.
#[repr(C)]
pub struct group_req {
    pub gr_interface: u32,
    pub gr_group: sockaddr_storage,
}

/// Non-POSIX.
#[repr(C)]
pub struct group_source_req {
    pub gsr_interface: u32,
    pub gsr_group: sockaddr_storage,
    pub gsr_source: sockaddr_storage,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
#[unsafe(no_mangle)]
pub static in6addr_any: in6_addr = in6_addr {
    s6_addr: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_in.h.html>.
#[unsafe(no_mangle)]
pub static in6addr_loopback: in6_addr = in6_addr {
    s6_addr: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/htonl.html>.
#[unsafe(no_mangle)]
pub extern "C" fn htonl(hostlong: uint32_t) -> uint32_t {
    hostlong.to_be()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/htonl.html>.
#[unsafe(no_mangle)]
pub extern "C" fn htons(hostshort: uint16_t) -> uint16_t {
    hostshort.to_be()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/htonl.html>.
#[unsafe(no_mangle)]
pub extern "C" fn ntohl(netlong: uint32_t) -> uint32_t {
    u32::from_be(netlong)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/htonl.html>.
#[unsafe(no_mangle)]
pub extern "C" fn ntohs(netshort: uint16_t) -> uint16_t {
    u16::from_be(netshort)
}
