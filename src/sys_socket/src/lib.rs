//! socket implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xns/syssocket.h.html

#![no_std]
#![allow(non_camel_case_types)]

extern crate platform;

use core::ptr;
use platform::types::*;

pub type in_addr_t = [u8; 4];
pub type in_port_t = u16;
pub type sa_family_t = u16;
pub type socklen_t = u32;

#[repr(C)]
pub struct sockaddr {
    pub sa_family: sa_family_t,
    data: [c_char; 14],
}

pub const AF_INET: c_int = 2;
pub const SOCK_STREAM: c_int = 1;
pub const SOCK_DGRAM: c_int = 2;
pub const SOCK_NONBLOCK: c_int = 0o4000;
pub const SOCK_CLOEXEC: c_int = 0o2000000;

#[no_mangle]
pub unsafe extern "C" fn accept(
    socket: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn bind(
    socket: c_int,
    address: *const sockaddr,
    address_len: socklen_t,
) -> c_int {
    platform::bind(socket, address as *const platform::sockaddr, address_len)
}

#[no_mangle]
pub unsafe extern "C" fn connect(
    socket: c_int,
    address: *const sockaddr,
    address_len: socklen_t,
) -> c_int {
    platform::connect(socket, address as *const platform::sockaddr, address_len)
}

#[no_mangle]
pub unsafe extern "C" fn getpeername(
    socket: c_int,
    address: *const sockaddr,
    address_len: socklen_t,
) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn getsockname(
    socket: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn getsockopt(
    socket: c_int,
    level: c_int,
    option_name: c_int,
    option_value: *mut c_void,
    option_len: *mut socklen_t,
) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn listen(socket: c_int, backlog: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn recv(
    socket: c_int,
    buffer: *mut c_void,
    length: size_t,
    flags: c_int,
) -> ssize_t {
    recvfrom(
        socket,
        buffer,
        length,
        flags,
        ptr::null_mut(),
        ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "C" fn recvfrom(
    socket: c_int,
    buffer: *mut c_void,
    length: size_t,
    flags: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> ssize_t {
    platform::recvfrom(
        socket,
        buffer,
        length,
        flags,
        address as *mut platform::sockaddr,
        address_len,
    )
}

#[no_mangle]
pub unsafe extern "C" fn send(
    socket: c_int,
    message: *const c_void,
    length: size_t,
    flags: c_int,
) -> ssize_t {
    sendto(socket, message, length, flags, ptr::null(), 0)
}

#[no_mangle]
pub unsafe extern "C" fn sendto(
    socket: c_int,
    message: *const c_void,
    length: size_t,
    flags: c_int,
    dest_addr: *const sockaddr,
    dest_len: socklen_t,
) -> ssize_t {
    platform::sendto(
        socket,
        message,
        length,
        flags,
        dest_addr as *const platform::sockaddr,
        dest_len,
    )
}

#[no_mangle]
pub unsafe extern "C" fn setsockopt(
    socket: c_int,
    level: c_int,
    option_name: c_int,
    option_value: *const c_void,
    option_len: socklen_t,
) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn shutdown(socket: c_int, how: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn socket(domain: c_int, kind: c_int, protocol: c_int) -> c_int {
    platform::socket(domain, kind, protocol)
}

#[no_mangle]
pub unsafe extern "C" fn socketpair(
    domain: c_int,
    _type: c_int,
    protocol: c_int,
    socket_vector: [c_int; 2],
) -> c_int {
    unimplemented!();
}
