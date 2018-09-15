use super::super::types::*;
use super::super::PalSocket;
use super::{e, Sys};
use header::sys_socket::{sockaddr, socklen_t};

impl Sys {
    fn getsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: *mut c_void,
        option_len: *mut socklen_t,
    ) -> c_int {
        e(unsafe {
            syscall!(
                GETSOCKOPT,
                socket,
                level,
                option_name,
                option_value,
                option_len
            )
        }) as c_int
    }

    fn listen(socket: c_int, backlog: c_int) -> c_int {
        e(unsafe { syscall!(LISTEN, socket, backlog) }) as c_int
    }

    fn setsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: *const c_void,
        option_len: socklen_t,
    ) -> c_int {
        e(unsafe {
            syscall!(
                SETSOCKOPT,
                socket,
                level,
                option_name,
                option_value,
                option_len
            )
        }) as c_int
    }

    fn shutdown(socket: c_int, how: c_int) -> c_int {
        e(unsafe { syscall!(SHUTDOWN, socket, how) }) as c_int
    }

    fn socketpair(domain: c_int, kind: c_int, protocol: c_int, socket_vector: *mut c_int) -> c_int {
        e(unsafe { syscall!(SOCKETPAIR, domain, kind, protocol, socket_vector) }) as c_int
    }
}

impl PalSocket for Sys {
    unsafe fn accept(socket: c_int, address: *mut sockaddr, address_len: *mut socklen_t) -> c_int {
        e(syscall!(ACCEPT, socket, address, address_len)) as c_int
    }

    unsafe fn bind(socket: c_int, address: *const sockaddr, address_len: socklen_t) -> c_int {
        e(syscall!(BIND, socket, address, address_len)) as c_int
    }

    unsafe fn connect(socket: c_int, address: *const sockaddr, address_len: socklen_t) -> c_int {
        e(syscall!(CONNECT, socket, address, address_len)) as c_int
    }

    unsafe fn getpeername(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> c_int {
        e(syscall!(GETPEERNAME, socket, address, address_len)) as c_int
    }

    unsafe fn getsockname(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> c_int {
        e(syscall!(GETSOCKNAME, socket, address, address_len)) as c_int
    }

    unsafe fn recvfrom(
        socket: c_int,
        buf: *mut c_void,
        len: size_t,
        flags: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> ssize_t {
        e(syscall!(
            RECVFROM,
            socket,
            buf,
            len,
            flags,
            address,
            address_len
        )) as ssize_t
    }

    unsafe fn sendto(
        socket: c_int,
        buf: *const c_void,
        len: size_t,
        flags: c_int,
        dest_addr: *const sockaddr,
        dest_len: socklen_t,
    ) -> ssize_t {
        e(syscall!(
            SENDTO, socket, buf, len, flags, dest_addr, dest_len
        )) as ssize_t
    }

    unsafe fn socket(domain: c_int, kind: c_int, protocol: c_int) -> c_int {
        e(syscall!(SOCKET, domain, kind, protocol)) as c_int
    }
}
