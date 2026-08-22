use super::{Sys, e_raw};
use crate::{
    error::Result,
    header::sys_socket::{msghdr, socklen_t},
    out::Out,
    platform::{PalSocket, types::c_int},
};

impl PalSocket for Sys {
    fn accept(socket: c_int, address_dst: Option<&mut [u8]>) -> Result<(c_int, socklen_t)> {
        let mut len = address_dst.as_ref().map_or(0, |a| a.len()) as socklen_t;

        let socket = e_raw(unsafe {
            syscall!(
                ACCEPT,
                socket,
                address_dst.map_or(core::ptr::null_mut(), |a| a.as_mut_ptr()),
                &raw mut len
            )
        })? as c_int;
        Ok((socket, len))
    }

    fn bind(socket: c_int, address_raw: &[u8]) -> Result<()> {
        e_raw(unsafe { syscall!(BIND, socket, address_raw.as_ptr(), address_raw.len()) })?;
        Ok(())
    }

    fn connect(socket: c_int, address_raw: &[u8]) -> Result<c_int> {
        Ok(
            e_raw(unsafe { syscall!(CONNECT, socket, address_raw.as_ptr(), address_raw.len()) })?
                as c_int,
        )
    }

    fn getpeername(socket: c_int, address_dst: &mut [u8]) -> Result<socklen_t> {
        let mut len = address_dst.len() as socklen_t;
        e_raw(unsafe { syscall!(GETPEERNAME, socket, address_dst.as_mut_ptr(), &raw mut len) })?;
        Ok(len)
    }

    fn getsockname(socket: c_int, address_dst: &mut [u8]) -> Result<socklen_t> {
        let mut len = address_dst.len() as socklen_t;
        e_raw(unsafe { syscall!(GETSOCKNAME, socket, address_dst.as_mut_ptr(), &raw mut len) })?;
        Ok(len)
    }

    fn getsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: &mut [u8],
    ) -> Result<socklen_t> {
        let mut len = option_value.len() as socklen_t;
        e_raw(unsafe {
            syscall!(
                GETSOCKOPT,
                socket,
                level,
                option_name,
                option_value.as_mut_ptr(),
                &raw mut len
            )
        })?;
        Ok(len)
    }

    fn listen(socket: c_int, backlog: c_int) -> Result<()> {
        e_raw(unsafe { syscall!(LISTEN, socket, backlog) })?;
        Ok(())
    }

    fn recvfrom(
        socket: c_int,
        mut buf: Out<[u8]>,
        flags: c_int,
        address_raw: Option<&mut [u8]>,
    ) -> Result<(usize, socklen_t)> {
        let mut len = address_raw.as_ref().map_or(0, |a| a.len()) as socklen_t;

        let bytes_read = e_raw(unsafe {
            syscall!(
                RECVFROM,
                socket,
                buf.as_mut_ptr().as_mut_ptr(),
                buf.len(),
                flags,
                address_raw.map_or(core::ptr::null_mut(), |a| a.as_mut_ptr()),
                &raw mut len
            )
        })?;

        Ok((bytes_read, len))
    }

    unsafe fn recvmsg(socket: c_int, msg: *mut msghdr, flags: c_int) -> Result<usize> {
        e_raw(syscall!(RECVMSG, socket, msg, flags))
    }

    unsafe fn sendmsg(socket: c_int, msg: *const msghdr, flags: c_int) -> Result<usize> {
        e_raw(syscall!(SENDMSG, socket, msg, flags))
    }

    fn sendto(socket: c_int, buf: &[u8], flags: c_int, dest: Option<&[u8]>) -> Result<usize> {
        e_raw(unsafe {
            syscall!(
                SENDTO,
                socket,
                buf.as_ptr(),
                buf.len(),
                flags,
                dest.map_or(core::ptr::null(), |d| d.as_ptr()),
                dest.map_or(0, |d| d.len())
            )
        })
    }

    fn setsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: &[u8],
    ) -> Result<()> {
        e_raw(unsafe {
            syscall!(
                SETSOCKOPT,
                socket,
                level,
                option_name,
                option_value.as_ptr(),
                option_value.len()
            )
        })?;
        Ok(())
    }

    fn shutdown(socket: c_int, how: c_int) -> Result<()> {
        e_raw(unsafe { syscall!(SHUTDOWN, socket, how) })?;
        Ok(())
    }

    fn socket(domain: c_int, kind: c_int, protocol: c_int) -> Result<c_int> {
        Ok(e_raw(unsafe { syscall!(SOCKET, domain, kind, protocol) })? as c_int)
    }

    fn socketpair(domain: c_int, kind: c_int, protocol: c_int, sv: &mut [c_int; 2]) -> Result<()> {
        e_raw(unsafe { syscall!(SOCKETPAIR, domain, kind, protocol, sv.as_mut_ptr()) })?;
        Ok(())
    }
}
