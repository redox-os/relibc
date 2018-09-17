use core::{mem, ptr, slice};
use syscall::flag::*;
use syscall::{self, Result};

use super::super::types::*;
use super::super::{errno, Pal, PalSocket};
use super::{e, Sys};
use header::netinet_in::{in_addr_t, in_port_t, sockaddr_in};
use header::sys_socket::constants::*;
use header::sys_socket::{sockaddr, socklen_t};

macro_rules! bind_or_connect {
    (bind $path:expr) => {
        concat!("/", $path)
    };
    (connect $path:expr) => {
        $path
    };
    ($mode:ident $socket:expr, $address:expr, $address_len:expr) => {{
        if (*$address).sa_family as c_int != AF_INET {
            errno = syscall::EAFNOSUPPORT;
            return -1;
        }
        if ($address_len as usize) < mem::size_of::<sockaddr>() {
            errno = syscall::EINVAL;
            return -1;
        }
        let data = &*($address as *const sockaddr_in);
        let addr = data.sin_addr.s_addr;
        let port = in_port_t::from_be(data.sin_port);
        let path = format!(
            bind_or_connect!($mode "{}.{}.{}.{}:{}"),
            addr >> 8 * 3,
            addr >> 8 * 2 & 0xFF,
            addr >> 8 & 0xFF,
            addr & 0xFF,
            port
        );

        // Duplicate the socket, and then duplicate the copy back to the original fd
        let fd = e(syscall::dup($socket as usize, path.as_bytes()));
        if (fd as c_int) < 0 {
            return -1;
        }
        let result = syscall::dup2(fd, $socket as usize, &[]);
        let _ = syscall::close(fd);
        if (e(result) as c_int) < 0 {
            return -1;
        }
        0
    }};
}

unsafe fn inner_get_name(
    local: bool,
    socket: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> Result<usize> {
    // 32 should probably be large enough.
    // Format: tcp:remote/local
    // and since we only yet support IPv4 (I think)...
    let mut buf = [0; 32];
    let len = syscall::fpath(socket as usize, &mut buf)?;
    let buf = &buf[..len];
    assert!(&buf[..4] == b"tcp:" || &buf[..4] == b"udp:");
    let buf = &buf[4..];

    let mut parts = buf.split(|c| *c == b'/');
    if local {
        // Skip the remote part
        parts.next();
    }
    let part = parts.next().expect("Invalid reply from netstack");

    let data = slice::from_raw_parts_mut(
        &mut (*address).data as *mut _ as *mut u8,
        (*address).data.len(),
    );

    let len = data.len().min(part.len());
    data[..len].copy_from_slice(&part[..len]);

    *address_len = len as socklen_t;
    Ok(0)
}

impl PalSocket for Sys {
    unsafe fn accept(socket: c_int, address: *mut sockaddr, address_len: *mut socklen_t) -> c_int {
        let stream = e(syscall::dup(socket as usize, b"listen")) as c_int;
        if stream < 0 {
            return -1;
        }
        if address != ptr::null_mut()
            && address_len != ptr::null_mut()
            && Self::getpeername(stream, address, address_len) < 0
        {
            return -1;
        }
        stream
    }

    unsafe fn bind(socket: c_int, address: *const sockaddr, address_len: socklen_t) -> c_int {
        bind_or_connect!(bind socket, address, address_len)
    }

    unsafe fn connect(socket: c_int, address: *const sockaddr, address_len: socklen_t) -> c_int {
        bind_or_connect!(connect socket, address, address_len)
    }

    unsafe fn getpeername(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> c_int {
        e(inner_get_name(false, socket, address, address_len)) as c_int
    }

    unsafe fn getsockname(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> c_int {
        e(inner_get_name(true, socket, address, address_len)) as c_int
    }

    unsafe fn recvfrom(
        socket: c_int,
        buf: *mut c_void,
        len: size_t,
        flags: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> ssize_t {
        if flags != 0 {
            errno = syscall::EOPNOTSUPP;
            return -1;
        }
        if address != ptr::null_mut()
            && address_len != ptr::null_mut()
            && Self::getpeername(socket, address, address_len) < 0
        {
            return -1;
        }
        Self::read(socket, slice::from_raw_parts_mut(buf as *mut u8, len))
    }

    unsafe fn sendto(
        socket: c_int,
        buf: *const c_void,
        len: size_t,
        flags: c_int,
        dest_addr: *const sockaddr,
        dest_len: socklen_t,
    ) -> ssize_t {
        if dest_addr != ptr::null() || dest_len != 0 {
            errno = syscall::EISCONN;
            return -1;
        }
        if flags != 0 {
            errno = syscall::EOPNOTSUPP;
            return -1;
        }
        Self::write(socket, slice::from_raw_parts(buf as *const u8, len))
    }

    unsafe fn socket(domain: c_int, mut kind: c_int, protocol: c_int) -> c_int {
        if domain != AF_INET {
            errno = syscall::EAFNOSUPPORT;
            return -1;
        }
        // if protocol != 0 {
        //     errno = syscall::EPROTONOSUPPORT;
        //     return -1;
        // }

        let mut flags = O_RDWR;
        if kind & SOCK_NONBLOCK == SOCK_NONBLOCK {
            kind &= !SOCK_NONBLOCK;
            flags |= O_NONBLOCK;
        }
        if kind & SOCK_CLOEXEC == SOCK_CLOEXEC {
            kind &= !SOCK_CLOEXEC;
            flags |= O_CLOEXEC;
        }

        // The tcp: and udp: schemes allow using no path,
        // and later specifying one using `dup`.
        match kind {
            SOCK_STREAM => e(syscall::open("tcp:", flags)) as c_int,
            SOCK_DGRAM => e(syscall::open("udp:", flags)) as c_int,
            _ => {
                errno = syscall::EPROTOTYPE;
                -1
            }
        }
    }
}
