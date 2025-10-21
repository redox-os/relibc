use super::{Sys, e_raw};
use crate::{
    error::Result,
    header::sys_socket::{msghdr, sockaddr, socklen_t},
    platform::{PalSocket, types::*},
};

impl PalSocket for Sys {
    unsafe fn accept(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> Result<c_int> {
        Ok(e_raw(syscall!(ACCEPT, socket, address, address_len))? as c_int)
    }

    unsafe fn bind(socket: c_int, address: *const sockaddr, address_len: socklen_t) -> Result<()> {
        e_raw(syscall!(BIND, socket, address, address_len))?;
        Ok(())
    }

    unsafe fn connect(
        socket: c_int,
        address: *const sockaddr,
        address_len: socklen_t,
    ) -> Result<c_int> {
        Ok(e_raw(syscall!(CONNECT, socket, address, address_len))? as c_int)
    }

    unsafe fn getpeername(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> Result<()> {
        e_raw(syscall!(GETPEERNAME, socket, address, address_len))?;
        Ok(())
    }

    unsafe fn getsockname(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> Result<()> {
        e_raw(syscall!(GETSOCKNAME, socket, address, address_len))?;
        Ok(())
    }

    unsafe fn getsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: *mut c_void,
        option_len: *mut socklen_t,
    ) -> Result<()> {
        e_raw(unsafe {
            syscall!(
                GETSOCKOPT,
                socket,
                level,
                option_name,
                option_value,
                option_len
            )
        })?;
        Ok(())
    }

    fn listen(socket: c_int, backlog: c_int) -> Result<()> {
        e_raw(unsafe { syscall!(LISTEN, socket, backlog) })?;
        Ok(())
    }

    unsafe fn recvfrom(
        socket: c_int,
        buf: *mut c_void,
        len: size_t,
        flags: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> Result<usize> {
        e_raw(syscall!(
            RECVFROM,
            socket,
            buf,
            len,
            flags,
            address,
            address_len
        ))
    }

    unsafe fn recvmsg(socket: c_int, msg: *mut msghdr, flags: c_int) -> Result<usize> {
        e_raw(syscall!(RECVMSG, socket, msg, flags))
    }

    unsafe fn sendmsg(socket: c_int, msg: *const msghdr, flags: c_int) -> Result<usize> {
        e_raw(syscall!(SENDMSG, socket, msg, flags))
    }

    unsafe fn sendto(
        socket: c_int,
        buf: *const c_void,
        len: size_t,
        flags: c_int,
        dest_addr: *const sockaddr,
        dest_len: socklen_t,
    ) -> Result<usize> {
        e_raw(syscall!(
            SENDTO, socket, buf, len, flags, dest_addr, dest_len
        ))
    }

    unsafe fn setsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: *const c_void,
        option_len: socklen_t,
    ) -> Result<()> {
        e_raw(unsafe {
            syscall!(
                SETSOCKOPT,
                socket,
                level,
                option_name,
                option_value,
                option_len
            )
        })?;
        Ok(())
    }

    fn shutdown(socket: c_int, how: c_int) -> Result<()> {
        e_raw(unsafe { syscall!(SHUTDOWN, socket, how) })?;
        Ok(())
    }

    unsafe fn socket(domain: c_int, kind: c_int, protocol: c_int) -> Result<c_int> {
        Ok(e_raw(syscall!(SOCKET, domain, kind, protocol))? as c_int)
    }

    fn socketpair(domain: c_int, kind: c_int, protocol: c_int, sv: &mut [c_int; 2]) -> Result<()> {
        e_raw(unsafe { syscall!(SOCKETPAIR, domain, kind, protocol, sv.as_mut_ptr()) })?;
        Ok(())
    }
}
