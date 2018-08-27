use super::super::types::*;
use super::super::Pal;

pub trait PalSocket: Pal {
    unsafe fn accept(socket: c_int, address: *mut sockaddr, address_len: *mut socklen_t) -> c_int {
        Self::no_pal("accept")
    }

    unsafe fn bind(socket: c_int, address: *const sockaddr, address_len: socklen_t) -> c_int {
        Self::no_pal("bind")
    }

    unsafe fn connect(socket: c_int, address: *const sockaddr, address_len: socklen_t) -> c_int {
        Self::no_pal("connect")
    }

    unsafe fn getpeername(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> c_int {
        Self::no_pal("getpeername")
    }

    unsafe fn getsockname(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> c_int {
        Self::no_pal("getsockname")
    }

    fn getsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: *mut c_void,
        option_len: *mut socklen_t,
    ) -> c_int {
        Self::no_pal("getsockopt")
    }

    fn listen(socket: c_int, backlog: c_int) -> c_int {
        Self::no_pal("listen")
    }

    unsafe fn recvfrom(
        socket: c_int,
        buf: *mut c_void,
        len: size_t,
        flags: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> ssize_t {
        Self::no_pal("recvfrom") as ssize_t
    }

    unsafe fn sendto(
        socket: c_int,
        buf: *const c_void,
        len: size_t,
        flags: c_int,
        dest_addr: *const sockaddr,
        dest_len: socklen_t,
    ) -> ssize_t {
        Self::no_pal("sendto") as ssize_t
    }

    fn setsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: *const c_void,
        option_len: socklen_t,
    ) -> c_int {
        Self::no_pal("setsockopt")
    }

    fn shutdown(socket: c_int, how: c_int) -> c_int {
        Self::no_pal("shutdown")
    }

    unsafe fn socket(domain: c_int, kind: c_int, protocol: c_int) -> c_int {
        Self::no_pal("socket")
    }

    fn socketpair(domain: c_int, kind: c_int, protocol: c_int, socket_vector: *mut c_int) -> c_int {
        Self::no_pal("socketpair")
    }
}
