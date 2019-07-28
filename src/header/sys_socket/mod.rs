//! socket implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xns/syssocket.h.html

use core::ptr;

use crate::platform::{types::*, PalSocket, Sys};

pub mod constants;

pub type sa_family_t = u16;
pub type socklen_t = u32;

#[repr(C)]
#[derive(Default)]
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
    trace_expr!(
        Sys::accept(socket, address, address_len),
        "accept({}, {:p}, {:p})",
        socket,
        address,
        address_len
    )
}

#[no_mangle]
pub unsafe extern "C" fn bind(
    socket: c_int,
    address: *const sockaddr,
    address_len: socklen_t,
) -> c_int {
    trace_expr!(
        Sys::bind(socket, address, address_len),
        "bind({}, {:p}, {})",
        socket,
        address,
        address_len
    )
}

#[no_mangle]
pub unsafe extern "C" fn connect(
    socket: c_int,
    address: *const sockaddr,
    address_len: socklen_t,
) -> c_int {
    trace_expr!(
        Sys::connect(socket, address, address_len),
        "connect({}, {:p}, {})",
        socket,
        address,
        address_len
    )
}

#[no_mangle]
pub unsafe extern "C" fn getpeername(
    socket: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> c_int {
    trace_expr!(
        Sys::getpeername(socket, address, address_len),
        "getpeername({}, {:p}, {:p})",
        socket,
        address,
        address_len
    )
}

#[no_mangle]
pub unsafe extern "C" fn getsockname(
    socket: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> c_int {
    trace_expr!(
        Sys::getsockname(socket, address, address_len),
        "getsockname({}, {:p}, {:p})",
        socket,
        address,
        address_len
    )
}

#[no_mangle]
pub unsafe extern "C" fn getsockopt(
    socket: c_int,
    level: c_int,
    option_name: c_int,
    option_value: *mut c_void,
    option_len: *mut socklen_t,
) -> c_int {
    trace_expr!(
        Sys::getsockopt(socket, level, option_name, option_value, option_len),
        "getsockopt({}, {}, {}, {:p}, {:p})",
        socket,
        level,
        option_name,
        option_value,
        option_len
    )
}

#[no_mangle]
pub unsafe extern "C" fn listen(socket: c_int, backlog: c_int) -> c_int {
    Sys::listen(socket, backlog)
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
    trace_expr!(
        Sys::recvfrom(socket, buffer, length, flags, address, address_len),
        "recvfrom({}, {:p}, {}, {:#x}, {:p}, {:p})",
        socket,
        buffer,
        length,
        flags,
        address,
        address_len
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
    trace_expr!(
        Sys::sendto(socket, message, length, flags, dest_addr, dest_len),
        "sendto({}, {:p}, {}, {:#x}, {:p}, {})",
        socket,
        message,
        length,
        flags,
        dest_addr,
        dest_len
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
    trace_expr!(
        Sys::setsockopt(socket, level, option_name, option_value, option_len),
        "setsockopt({}, {}, {}, {:p}, {})",
        socket,
        level,
        option_name,
        option_value,
        option_len
    )
}

#[no_mangle]
pub unsafe extern "C" fn shutdown(socket: c_int, how: c_int) -> c_int {
    Sys::shutdown(socket, how)
}

#[no_mangle]
pub unsafe extern "C" fn socket(domain: c_int, kind: c_int, protocol: c_int) -> c_int {
    trace_expr!(
        Sys::socket(domain, kind, protocol),
        "socket({}, {}, {})",
        domain,
        kind,
        protocol,
    )
}

#[no_mangle]
pub unsafe extern "C" fn socketpair(
    domain: c_int,
    kind: c_int,
    protocol: c_int,
    sv: *mut c_int,
) -> c_int {
    trace_expr!(
        Sys::socketpair(domain, kind, protocol, &mut *(sv as *mut [c_int; 2])),
        "socketpair({}, {}, {}, {:p})",
        domain,
        kind,
        protocol,
        sv
    )
}
