use super::super::{types::*, Pal};
use crate::header::sys_socket::{sockaddr, socklen_t};

pub trait PalSocket: Pal {
    unsafe fn accept(socket: c_int, address: *mut sockaddr, address_len: *mut socklen_t) -> c_int;

    unsafe fn bind(socket: c_int, address: *const sockaddr, address_len: socklen_t) -> c_int;

    unsafe fn connect(socket: c_int, address: *const sockaddr, address_len: socklen_t) -> c_int;

    unsafe fn getpeername(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> c_int;

    unsafe fn getsockname(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> c_int;

    fn getsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: *mut c_void,
        option_len: *mut socklen_t,
    ) -> c_int;

    fn listen(socket: c_int, backlog: c_int) -> c_int;

    unsafe fn recvfrom(
        socket: c_int,
        buf: *mut c_void,
        len: size_t,
        flags: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> ssize_t;

    unsafe fn sendto(
        socket: c_int,
        buf: *const c_void,
        len: size_t,
        flags: c_int,
        dest_addr: *const sockaddr,
        dest_len: socklen_t,
    ) -> ssize_t;

    fn setsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: *const c_void,
        option_len: socklen_t,
    ) -> c_int;

    fn shutdown(socket: c_int, how: c_int) -> c_int;

    unsafe fn socket(domain: c_int, kind: c_int, protocol: c_int) -> c_int;

    fn socketpair(domain: c_int, kind: c_int, protocol: c_int, sv: &mut [c_int; 2]) -> c_int;
}
