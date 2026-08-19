use crate::{
    error::Result,
    header::sys_socket::{msghdr, socklen_t},
    out::Out,
    platform::{Pal, types::c_int},
};

/// Platform abstraction of socket functionality.
pub trait PalSocket: Pal {
    /// Platform implementation of [`accept()`](crate::header::sys_socket::accept) from [`sys/socket.h`](crate::header::sys_socket).
    fn accept(socket: c_int, address_dst: Option<&mut [u8]>) -> Result<(c_int, socklen_t)>;

    /// Platform implementation of [`bind()`](crate::header::sys_socket::bind) from [`sys/socket.h`](crate::header::sys_socket).
    fn bind(socket: c_int, address_raw: &[u8]) -> Result<()>;

    /// Platform implementation of [`connect()`](crate::header::sys_socket::connect) from [`sys/socket.h`](crate::header::sys_socket).
    fn connect(socket: c_int, address_raw: &[u8]) -> Result<c_int>;

    /// Platform implementation of [`getpeername()`](crate::header::sys_socket::getpeername) from [`sys/socket.h`](crate::header::sys_socket).
    fn getpeername(socket: c_int, address_dst: &mut [u8]) -> Result<socklen_t>;

    /// Platform implementation of [`getsockname()`](crate::header::sys_socket::getsockname) from [`sys/socket.h`](crate::header::sys_socket).
    fn getsockname(socket: c_int, address_dst: &mut [u8]) -> Result<socklen_t>;

    /// Platform implementation of [`getsockopt()`](crate::header::sys_socket::getsockopt) from [`sys/socket.h`](crate::header::sys_socket).
    fn getsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: &mut [u8],
    ) -> Result<socklen_t>;

    /// Platform implementation of [`listen()`](crate::header::sys_socket::listen) from [`sys/socket.h`](crate::header::sys_socket).
    fn listen(socket: c_int, backlog: c_int) -> Result<()>;

    /// Platform implementation of [`recvfrom()`](crate::header::sys_socket::recvfrom) from [`sys/socket.h`](crate::header::sys_socket).
    fn recvfrom(
        socket: c_int,
        buf: Out<[u8]>,
        flags: c_int,
        address_raw: Option<&mut [u8]>,
    ) -> Result<(usize, socklen_t)>;

    /// Platform implementation of [`recvmsg()`](crate::header::sys_socket::recvmsg) from [`sys/socket.h`](crate::header::sys_socket).
    // TODO: Make this safe. We can pass mutable references and use I/O slices etc, while checking
    // types are compatible.
    unsafe fn recvmsg(socket: c_int, msg: *mut msghdr, flags: c_int) -> Result<usize>;

    /// Platform implementation of [`sendmsg()`](crate::header::sys_socket::sendmsg) from [`sys/socket.h`](crate::header::sys_socket).
    // TODO: Make this safe. We can pass immutable references and use I/O slices etc, while checking
    // types are compatible.
    unsafe fn sendmsg(socket: c_int, msg: *const msghdr, flags: c_int) -> Result<usize>;

    /// Platform implementation of [`sendto()`](crate::header::sys_socket::sendto) from [`sys/socket.h`](crate::header::sys_socket).
    fn sendto(socket: c_int, buf: &[u8], flags: c_int, dest_addr: Option<&[u8]>) -> Result<usize>;

    /// Platform implementation of [`setsockopt()`](crate::header::sys_socket::setsockopt) from [`sys/socket.h`](crate::header::sys_socket).
    fn setsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: &[u8],
    ) -> Result<()>;

    /// Platform implementation of [`shutdown()`](crate::header::sys_socket::shutdown) from [`sys/socket.h`](crate::header::sys_socket).
    fn shutdown(socket: c_int, how: c_int) -> Result<()>;

    /// Platform implementation of [`socket()`](crate::header::sys_socket::socket) from [`sys/socket.h`](crate::header::sys_socket).
    fn socket(domain: c_int, kind: c_int, protocol: c_int) -> Result<c_int>;

    /// Platform implementation of [`socketpair()`](crate::header::sys_socket::socketpair) from [`sys/socket.h`](crate::header::sys_socket).
    fn socketpair(domain: c_int, kind: c_int, protocol: c_int, sv: &mut [c_int; 2]) -> Result<()>;
}
