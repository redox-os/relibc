use alloc::vec::Vec;
use core::{cmp, mem, ptr, slice, str};
use syscall::{self, flag::*, Result};

use super::{
    super::{errno, types::*, Pal, PalSocket},
    e, Sys,
};
use crate::header::{
    arpa_inet::inet_aton,
    netinet_in::{in_addr, in_port_t, sockaddr_in},
    string::strnlen,
    sys_socket::{constants::*, sa_family_t, sockaddr, socklen_t},
    sys_time::timeval,
    sys_un::sockaddr_un,
};

macro_rules! bind_or_connect {
    (bind $path:expr) => {
        concat!("/", $path)
    };
    (connect $path:expr) => {
        $path
    };
    ($mode:ident into, $socket:expr, $address:expr, $address_len:expr) => {{
        let fd = bind_or_connect!($mode copy, $socket, $address, $address_len);

        let result = syscall::dup2(fd, $socket as usize, &[]);
        let _ = syscall::close(fd);
        if (e(result) as c_int) < 0 {
            return -1;
        }
        0
    }};
    ($mode:ident copy, $socket:expr, $address:expr, $address_len:expr) => {{
        if ($address_len as usize) < mem::size_of::<sa_family_t>() {
            errno = syscall::EINVAL;
            return -1;
        }

        let path = match (*$address).sa_family as c_int {
            AF_INET => {
                if ($address_len as usize) != mem::size_of::<sockaddr_in>() {
                    errno = syscall::EINVAL;
                    return -1;
                }
                let data = &*($address as *const sockaddr_in);
                let addr = slice::from_raw_parts(
                    &data.sin_addr.s_addr as *const _ as *const u8,
                    mem::size_of_val(&data.sin_addr.s_addr),
                );
                let port = in_port_t::from_be(data.sin_port);
                let path = format!(
                    bind_or_connect!($mode "{}.{}.{}.{}:{}"),
                    addr[0],
                    addr[1],
                    addr[2],
                    addr[3],
                    port
                );

                path
            },
            AF_UNIX => {
                let data = &*($address as *const sockaddr_un);

                // NOTE: It's UB to access data in given address that exceeds
                // the given address length.

                let maxlen = cmp::min(
                    // Max path length of the full-sized struct
                    data.sun_path.len(),
                    // Length inferred from given addrlen
                    $address_len as usize - data.path_offset()
                );
                let len = cmp::min(
                    // The maximum length of the address
                    maxlen,
                    // The first NUL byte, if any
                    strnlen(&data.sun_path as *const _, maxlen as size_t),
                );

                let addr = slice::from_raw_parts(
                    &data.sun_path as *const _ as *const u8,
                    len,
                );
                let path = format!(
                    "{}",
                    str::from_utf8(addr).unwrap()
                );
                trace!("path: {:?}", path);

                path
            },
            _ => {
                errno = syscall::EAFNOSUPPORT;
                return -1;
            },
        };

        // Duplicate the socket, and then duplicate the copy back to the original fd
        let fd = e(syscall::dup($socket as usize, path.as_bytes()));
        if (fd as c_int) < 0 {
            return -1;
        }
        fd
    }};
}

unsafe fn inner_af_unix(buf: &[u8], address: *mut sockaddr, address_len: *mut socklen_t) {
    let data = &mut *(address as *mut sockaddr_un);

    data.sun_family = AF_UNIX as c_ushort;

    let path =
        slice::from_raw_parts_mut(&mut data.sun_path as *mut _ as *mut u8, data.sun_path.len());

    let len = cmp::min(path.len(), buf.len());
    path[..len].copy_from_slice(&buf[..len]);

    *address_len = len as socklen_t;
}

unsafe fn inner_af_inet(
    local: bool,
    buf: &[u8],
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) {
    let mut parts = buf.split(|c| *c == b'/');
    if local {
        // Skip the remote part
        parts.next();
    }
    let mut unparsed_addr = Vec::from(parts.next().expect("missing address"));

    let sep = memchr::memchr(b':', &unparsed_addr).expect("missing port");
    let (raw_addr, rest) = unparsed_addr.split_at_mut(sep);
    let (colon, raw_port) = rest.split_at_mut(1);
    let port = str::from_utf8(raw_port)
        .expect("non-utf8 port")
        .parse()
        .expect("invalid port");

    // Make address be followed by a NUL-byte
    colon[0] = b'\0';

    trace!("address: {:?}, port: {:?}", str::from_utf8(&raw_addr), port);

    let mut addr = in_addr::default();
    assert_eq!(
        inet_aton(raw_addr.as_ptr() as *mut i8, &mut addr),
        1,
        "inet_aton might be broken, failed to parse netstack address"
    );

    let ret = sockaddr_in {
        sin_family: AF_INET as sa_family_t,
        sin_port: port,
        sin_addr: addr,

        ..sockaddr_in::default()
    };
    let len = cmp::min(*address_len as usize, mem::size_of_val(&ret));

    ptr::copy_nonoverlapping(&ret as *const _ as *const u8, address as *mut u8, len);
    *address_len = len as socklen_t;
}

unsafe fn inner_get_name(
    local: bool,
    socket: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> Result<usize> {
    // Format: [udp|tcp:]remote/local, chan:path
    let mut buf = [0; 256];
    let len = syscall::fpath(socket as usize, &mut buf)?;
    let buf = &buf[..len];

    if buf.starts_with(b"tcp:") || buf.starts_with(b"udp:") {
        inner_af_inet(local, &buf[4..], address, address_len);
    } else if buf.starts_with(b"chan:") {
        inner_af_unix(&buf[5..], address, address_len);
    } else {
        // Socket doesn't belong to any scheme
        panic!(
            "socket {:?} doesn't match either tcp, udp or chan schemes",
            str::from_utf8(buf)
        );
    }

    Ok(0)
}

fn socket_kind(mut kind: c_int) -> (c_int, usize) {
    let mut flags = O_RDWR;
    if kind & SOCK_NONBLOCK == SOCK_NONBLOCK {
        kind &= !SOCK_NONBLOCK;
        flags |= O_NONBLOCK;
    }
    if kind & SOCK_CLOEXEC == SOCK_CLOEXEC {
        kind &= !SOCK_CLOEXEC;
        flags |= O_CLOEXEC;
    }
    (kind, flags)
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
        bind_or_connect!(bind into, socket, address, address_len)
    }

    unsafe fn connect(socket: c_int, address: *const sockaddr, address_len: socklen_t) -> c_int {
        bind_or_connect!(connect into, socket, address, address_len)
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

    fn getsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: *mut c_void,
        option_len: *mut socklen_t,
    ) -> c_int {
        match level {
            SOL_SOCKET => match option_name {
                SO_ERROR => {
                    if option_value.is_null() {
                        return e(Err(syscall::Error::new(syscall::EFAULT))) as c_int;
                    }

                    if (option_len as usize) < mem::size_of::<c_int>() {
                        return e(Err(syscall::Error::new(syscall::EINVAL))) as c_int;
                    }

                    let error = unsafe { &mut *(option_value as *mut c_int) };
                    //TODO: Socket nonblock connection error
                    *error = 0;

                    return 0;
                }
                _ => (),
            },
            _ => (),
        }

        eprintln!(
            "getsockopt({}, {}, {}, {:p}, {:p})",
            socket, level, option_name, option_value, option_len
        );
        e(Err(syscall::Error::new(syscall::ENOSYS))) as c_int
    }

    fn listen(socket: c_int, backlog: c_int) -> c_int {
        // Redox has no need to listen
        0
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
        if address == ptr::null_mut() || address_len == ptr::null_mut() {
            Self::read(socket, slice::from_raw_parts_mut(buf as *mut u8, len))
        } else {
            let fd = e(syscall::dup(socket as usize, b"listen"));
            if fd == !0 {
                return -1;
            }
            if Self::getpeername(fd as c_int, address, address_len) < 0 {
                let _ = syscall::close(fd);
                return -1;
            }

            let ret = Self::read(fd as c_int, slice::from_raw_parts_mut(buf as *mut u8, len));
            let _ = syscall::close(fd);
            ret
        }
    }

    unsafe fn sendto(
        socket: c_int,
        buf: *const c_void,
        len: size_t,
        flags: c_int,
        dest_addr: *const sockaddr,
        dest_len: socklen_t,
    ) -> ssize_t {
        if flags != 0 {
            errno = syscall::EOPNOTSUPP;
            return -1;
        }
        if dest_addr == ptr::null() || dest_len == 0 {
            Self::write(socket, slice::from_raw_parts(buf as *const u8, len))
        } else {
            let fd = bind_or_connect!(connect copy, socket, dest_addr, dest_len);
            let ret = Self::write(fd as c_int, slice::from_raw_parts(buf as *const u8, len));
            let _ = syscall::close(fd);
            ret
        }
    }

    fn setsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: *const c_void,
        option_len: socklen_t,
    ) -> c_int {
        let set_timeout = |timeout_name: &[u8]| -> c_int {
            if option_value.is_null() {
                return e(Err(syscall::Error::new(syscall::EFAULT))) as c_int;
            }

            if (option_len as usize) < mem::size_of::<timeval>() {
                return e(Err(syscall::Error::new(syscall::EINVAL))) as c_int;
            }

            let timeval = unsafe { &*(option_value as *const timeval) };

            let fd = e(syscall::dup(socket as usize, timeout_name));
            if fd == !0 {
                return -1;
            }

            let timespec = syscall::TimeSpec {
                tv_sec: timeval.tv_sec,
                tv_nsec: timeval.tv_usec * 1000,
            };

            let ret = Self::write(fd as c_int, &timespec);

            let _ = syscall::close(fd);

            if ret >= 0 {
                0
            } else {
                -1
            }
        };

        match level {
            SOL_SOCKET => match option_name {
                SO_RCVTIMEO => return set_timeout(b"read_timeout"),
                SO_SNDTIMEO => return set_timeout(b"write_timeout"),
                _ => (),
            },
            _ => (),
        }

        eprintln!(
            "setsockopt({}, {}, {}, {:p}, {}) - unknown option",
            socket, level, option_name, option_value, option_len
        );
        0
    }

    fn shutdown(socket: c_int, how: c_int) -> c_int {
        eprintln!("shutdown({}, {})", socket, how);
        e(Err(syscall::Error::new(syscall::ENOSYS))) as c_int
    }

    unsafe fn socket(domain: c_int, kind: c_int, protocol: c_int) -> c_int {
        if domain != AF_INET && domain != AF_UNIX {
            errno = syscall::EAFNOSUPPORT;
            return -1;
        }
        // if protocol != 0 {
        //     errno = syscall::EPROTONOSUPPORT;
        //     return -1;
        // }

        let (kind, flags) = socket_kind(kind);

        // The tcp: and udp: schemes allow using no path,
        // and later specifying one using `dup`.
        match (domain, kind) {
            (AF_INET, SOCK_STREAM) => e(syscall::open("tcp:", flags)) as c_int,
            (AF_INET, SOCK_DGRAM) => e(syscall::open("udp:", flags)) as c_int,
            (AF_UNIX, SOCK_STREAM) => e(syscall::open("chan:", flags | O_CREAT)) as c_int,
            _ => {
                errno = syscall::EPROTONOSUPPORT;
                -1
            }
        }
    }

    fn socketpair(domain: c_int, kind: c_int, protocol: c_int, sv: &mut [c_int; 2]) -> c_int {
        let (kind, flags) = socket_kind(kind);

        match (domain, kind) {
            (AF_UNIX, SOCK_STREAM) => {
                let listener = e(syscall::open("chan:", flags | O_CREAT));
                if listener == !0 {
                    return -1;
                }

                // For now, chan: lets connects be instant, and instead blocks
                // on any I/O performed. So we don't need to mark this as
                // nonblocking.

                let fd0 = e(syscall::dup(listener, b"connect"));
                if fd0 == !0 {
                    let _ = syscall::close(listener);
                    return -1;
                }

                let fd1 = e(syscall::dup(listener, b"listen"));
                if fd1 == !0 {
                    let _ = syscall::close(fd0);
                    let _ = syscall::close(listener);
                    return -1;
                }

                sv[0] = fd0 as c_int;
                sv[1] = fd1 as c_int;
                0
            }
            _ => unsafe {
                eprintln!(
                    "socketpair({}, {}, {}, {:p})",
                    domain,
                    kind,
                    protocol,
                    sv.as_mut_ptr()
                );
                errno = syscall::EPROTONOSUPPORT;
                -1
            },
        }
    }
}
