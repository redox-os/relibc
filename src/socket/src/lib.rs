//! socket implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xns/syssocket.h.html

#![no_std]
#![allow(non_camel_case_types)]

extern crate platform;

use platform::types::*;

pub type sa_family_t = u16;
pub type socklen_t = u32;

#[repr(C)]
pub struct sockaddr {
    pub sa_family: sa_family_t,
    pub sa_data: [c_char; 14],
}

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
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn connect(
    socket: c_int,
    address: *const sockaddr,
    address_len: socklen_t,
) -> c_int {
    unimplemented!();
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
    unimplemented!();
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
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn send(
    socket: c_int,
    message: *const c_void,
    length: size_t,
    flags: c_int,
) -> ssize_t {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn sento(
    socket: c_int,
    message: *const c_void,
    length: size_t,
    flags: c_int,
    dest_addr: *const sockaddr,
    dest_len: socklen_t,
) -> ssize_t {
    unimplemented!();
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
pub unsafe extern "C" fn socket(domain: c_int, _type: c_int, protocol: c_int) -> c_int {
    unimplemented!();
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
