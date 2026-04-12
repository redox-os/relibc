use crate::{
    error::Result,
    header::{
        bits_socklen_t::socklen_t,
        sys_socket::{msghdr, sockaddr},
    },
    platform::{
        Pal,
        types::{c_int, c_void, size_t},
    },
};

/// Platform abstraction of socket functionality.
pub trait PalSocket: Pal {
    /// Platform implementation of [`accept()`](crate::header::sys_socket::accept) from [`sys/socket.h`](crate::header::sys_socket).
    unsafe fn accept(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> Result<c_int>;

    /// Platform implementation of [`bind()`](crate::header::sys_socket::bind) from [`sys/socket.h`](crate::header::sys_socket).
    unsafe fn bind(socket: c_int, address: *const sockaddr, address_len: socklen_t) -> Result<()>;

    /// Platform implementation of [`connect()`](crate::header::sys_socket::connect) from [`sys/socket.h`](crate::header::sys_socket).
    unsafe fn connect(
        socket: c_int,
        address: *const sockaddr,
        address_len: socklen_t,
    ) -> Result<c_int>;

    /// Platform implementation of [`getpeername()`](crate::header::sys_socket::getpeername) from [`sys/socket.h`](crate::header::sys_socket).
    unsafe fn getpeername(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> Result<()>;

    /// Platform implementation of [`getsockname()`](crate::header::sys_socket::getsockname) from [`sys/socket.h`](crate::header::sys_socket).
    unsafe fn getsockname(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> Result<()>;

    /// Platform implementation of [`getsockopt()`](crate::header::sys_socket::getsockopt) from [`sys/socket.h`](crate::header::sys_socket).
    unsafe fn getsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: *mut c_void,
        option_len: *mut socklen_t,
    ) -> Result<()>;

    /// Platform implementation of [`listen()`](crate::header::sys_socket::listen) from [`sys/socket.h`](crate::header::sys_socket).
    fn listen(socket: c_int, backlog: c_int) -> Result<()>;

    /// Platform implementation of [`recvfrom()`](crate::header::sys_socket::recvfrom) from [`sys/socket.h`](crate::header::sys_socket).
    unsafe fn recvfrom(
        socket: c_int,
        buf: *mut c_void,
        len: size_t,
        flags: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> Result<usize>;

    /// Platform implementation of [`recvmsg()`](crate::header::sys_socket::recvmsg) from [`sys/socket.h`](crate::header::sys_socket).
    unsafe fn recvmsg(socket: c_int, msg: *mut msghdr, flags: c_int) -> Result<usize>;

    /// Platform implementation of [`sendmsg()`](crate::header::sys_socket::sendmsg) from [`sys/socket.h`](crate::header::sys_socket).
    unsafe fn sendmsg(socket: c_int, msg: *const msghdr, flags: c_int) -> Result<usize>;

    /// Platform implementation of [`sendto()`](crate::header::sys_socket::sendto) from [`sys/socket.h`](crate::header::sys_socket).
    unsafe fn sendto(
        socket: c_int,
        buf: *const c_void,
        len: size_t,
        flags: c_int,
        dest_addr: *const sockaddr,
        dest_len: socklen_t,
    ) -> Result<usize>;

    /// Platform implementation of [`setsockopt()`](crate::header::sys_socket::setsockopt) from [`sys/socket.h`](crate::header::sys_socket).
    unsafe fn setsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: *const c_void,
        option_len: socklen_t,
    ) -> Result<()>;

    /// Platform implementation of [`shutdown()`](crate::header::sys_socket::shutdown) from [`sys/socket.h`](crate::header::sys_socket).
    fn shutdown(socket: c_int, how: c_int) -> Result<()>;

    /// Platform implementation of [`socket()`](crate::header::sys_socket::socket) from [`sys/socket.h`](crate::header::sys_socket).
    unsafe fn socket(domain: c_int, kind: c_int, protocol: c_int) -> Result<c_int>;

    /// Platform implementation of [`socketpair()`](crate::header::sys_socket::socketpair) from [`sys/socket.h`](crate::header::sys_socket).
    fn socketpair(domain: c_int, kind: c_int, protocol: c_int, sv: &mut [c_int; 2]) -> Result<()>;
}
