use alloc::vec::Vec;
use core::{cmp, mem, ptr, slice, str};
use redox_rt::proc::FdGuard;
use syscall::{self, flag::*};

use super::{
    super::{types::*, Pal, PalSocket, ERRNO},
    Sys,
};
use crate::{
    error::{Errno, Result, ResultExt},
    header::{
        arpa_inet::inet_aton,
        errno::{EAFNOSUPPORT, EDOM, EFAULT, EINVAL, ENOSYS, EOPNOTSUPP, EPROTONOSUPPORT},
        netinet_in::{in_addr, in_port_t, sockaddr_in},
        string::strnlen,
        sys_socket::{
            cmsghdr, constants::*, msghdr, sa_family_t, sockaddr, socklen_t, ucred, CMSG_ALIGN,
            CMSG_DATA, CMSG_FIRSTHDR, CMSG_LEN, CMSG_NXTHDR, CMSG_SPACE,
        },
        sys_time::timeval,
        sys_un::sockaddr_un,
    },
};

macro_rules! bind_or_connect {
    (bind $path:expr) => {
        concat!("/", $path)
    };
    (connect $path:expr) => {
        $path
    };
    ($mode:ident into, $socket:expr, $address:expr, $address_len:expr) => {{
        let fd = bind_or_connect!($mode copy, $socket, $address, $address_len)?;

        let _ = syscall::dup2(fd, $socket as usize, &[])?;
        Result::<c_int, Errno>::Ok(0)
    }};
    ($mode:ident copy, $socket:expr, $address:expr, $address_len:expr) => {{
        if ($address_len as usize) < mem::size_of::<sa_family_t>() {
            return Err(Errno(EINVAL));
        }

        let path = match (*$address).sa_family as c_int {
            AF_INET => {
                if ($address_len as usize) != mem::size_of::<sockaddr_in>() {
                    return Err(Errno(EINVAL));
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
                return Err(Errno(EAFNOSUPPORT));
            },
        };

        // Duplicate the socket, and then duplicate the copy back to the original fd
        syscall::dup($socket as usize, path.as_bytes())
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
        inet_aton(raw_addr.as_ptr() as *mut c_char, &mut addr),
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
) -> Result<()> {
    // Format: [udp|tcp:]remote/local, chan:path
    let mut buf = [0; 256];
    let len = syscall::fpath(socket as usize, &mut buf)?;
    let buf = &buf[..len];

    if buf.starts_with(b"tcp:") || buf.starts_with(b"udp:") {
        inner_af_inet(local, &buf[4..], address, address_len);
    } else if buf.starts_with(b"/scheme/tcp/") || buf.starts_with(b"/scheme/udp/") {
        inner_af_inet(local, &buf[12..], address, address_len);
    } else if buf.starts_with(b"chan:") {
        inner_af_unix(&buf[5..], address, address_len);
    } else if buf.starts_with(b"/scheme/chan/") {
        inner_af_unix(&buf[13..], address, address_len);
    } else if buf.starts_with(b"/scheme/uds_stream/") {
        inner_af_unix(&buf[19..], address, address_len);
    } else if buf.starts_with(b"/scheme/uds_dgram/") {
        inner_af_unix(&buf[18..], address, address_len);
    } else {
        // Socket doesn't belong to any scheme
        panic!(
            "socket {:?} doesn't match either tcp, udp or chan schemes",
            str::from_utf8(buf)
        );
    }

    Ok(())
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
    unsafe fn accept(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> Result<c_int> {
        let stream = syscall::dup(socket as usize, b"listen")? as c_int;
        if address != ptr::null_mut() && address_len != ptr::null_mut() {
            let _ = Self::getpeername(stream, address, address_len)?;
        }
        Ok(stream)
    }

    unsafe fn bind(socket: c_int, address: *const sockaddr, address_len: socklen_t) -> Result<()> {
        bind_or_connect!(bind into, socket, address, address_len)?;
        Ok(())
    }

    unsafe fn connect(
        socket: c_int,
        address: *const sockaddr,
        address_len: socklen_t,
    ) -> Result<c_int> {
        bind_or_connect!(connect into, socket, address, address_len)
    }

    unsafe fn getpeername(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> Result<()> {
        inner_get_name(false, socket, address, address_len)
    }

    unsafe fn getsockname(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> Result<()> {
        inner_get_name(true, socket, address, address_len)
    }

    unsafe fn getsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: *mut c_void,
        option_len: *mut socklen_t,
    ) -> Result<()> {
        match level {
            SOL_SOCKET => match option_name {
                SO_ERROR => {
                    if option_value.is_null() {
                        return Err(Errno(EFAULT));
                    }

                    if (option_len as usize) < mem::size_of::<c_int>() {
                        return Err(Errno(EINVAL));
                    }

                    let error = unsafe { &mut *(option_value as *mut c_int) };
                    //TODO: Socket nonblock connection error
                    *error = 0;

                    return Ok(());
                }
                _ => (),
            },
            _ => (),
        }

        eprintln!(
            "getsockopt({}, {}, {}, {:p}, {:p})",
            socket, level, option_name, option_value, option_len
        );
        Err(Errno(ENOSYS))
    }

    fn listen(socket: c_int, backlog: c_int) -> Result<()> {
        // Redox has no need to listen
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
        if flags != 0 {
            return Err(Errno(EOPNOTSUPP));
        }
        if address == ptr::null_mut() || address_len == ptr::null_mut() {
            Self::read(socket, slice::from_raw_parts_mut(buf as *mut u8, len))
        } else {
            let fd = FdGuard::new(syscall::dup(socket as usize, b"listen")?);
            Self::getpeername(*fd as c_int, address, address_len)?;

            Self::read(*fd as c_int, slice::from_raw_parts_mut(buf as *mut u8, len))
        }
    }

    unsafe fn recvmsg(socket: c_int, msg: *mut msghdr, flags: c_int) -> Result<usize> {
        if msg.is_null() {
            return Err(Errno(EINVAL));
        }
        let mhdr = &mut *msg;

        // 1. Read the size of the ancillary data.
        let mut ancillary_len_bytes = [0u8; mem::size_of::<usize>()];
        Self::read(socket, &mut ancillary_len_bytes)?;
        let ancillary_len = usize::from_le_bytes(ancillary_len_bytes);

        enum AncillaryType {
            Rights(Vec<c_int>),
            Credentials(ucred),
        }
        let mut ancillary_data: Vec<AncillaryType> = Vec::new();

        // 2. Read and parse the serialized ancillary data.
        if ancillary_len > 0 {
            let mut ancillary_data_buffer = vec![0u8; ancillary_len];
            Self::read(socket, &mut ancillary_data_buffer)?;

            let mut cursor = 0;
            while cursor < ancillary_len {
                let level_bytes = &ancillary_data_buffer[cursor..cursor + 4];
                let type_bytes = &ancillary_data_buffer[cursor + 4..cursor + 8];
                let cmsg_level = u32::from_le_bytes(level_bytes.try_into().unwrap()) as c_int;
                let cmsg_type = u32::from_le_bytes(type_bytes.try_into().unwrap()) as c_int;
                cursor += 8;

                match (cmsg_level, cmsg_type) {
                    (SOL_SOCKET, SCM_RIGHTS) => {
                        let mut received_fds: Vec<c_int> = Vec::new();
                        let count_bytes = &ancillary_data_buffer[cursor..cursor + 4];
                        let fd_count = u32::from_le_bytes(count_bytes.try_into().unwrap());
                        cursor += 4;

                        for _ in 0..fd_count {
                            let new_fd = syscall::dup(socket as usize, b"recvfd")?;
                            received_fds.push(new_fd as c_int);
                        }
                        ancillary_data.push(AncillaryType::Rights(received_fds));
                    }
                    (SOL_SOCKET, SCM_CREDENTIALS) => {
                        let cred_id = FdGuard::new(syscall::dup(socket as usize, b"cred")?);

                        // Assuming the read returns pid, uid, gid packed together.
                        let mut cred_buf =
                            vec![0u8; mem::size_of::<usize>() + mem::size_of::<u32>() * 2];
                        Self::read(*cred_id as c_int, &mut cred_buf)?;
                        Self::close(*cred_id as c_int)?;

                        let pid = usize::from_le_bytes(
                            cred_buf[0..mem::size_of::<usize>()].try_into().unwrap(),
                        ) as pid_t;
                        let uid = u32::from_le_bytes(
                            cred_buf[mem::size_of::<usize>()..mem::size_of::<usize>() + 4]
                                .try_into()
                                .unwrap(),
                        ) as uid_t;
                        let gid = u32::from_le_bytes(
                            cred_buf[mem::size_of::<usize>() + 4..].try_into().unwrap(),
                        ) as gid_t;

                        ancillary_data.push(AncillaryType::Credentials(ucred { pid, uid, gid }));
                    }
                    _ => {
                        // Unknown type, skip.
                        return Err(Errno(EINVAL));
                    }
                }
            }
        }

        // 3. Reconstruct the ancillary data in the user-provided buffer.
        mhdr.msg_controllen = 0;
        if !mhdr.msg_control.is_null() && mhdr.msg_controllen > 0 {
            let mut cmsg_space = mhdr.msg_controllen;
            let cmsg_ptr = mhdr.msg_control as *mut cmsghdr;
            let mut current_cmsg = CMSG_FIRSTHDR(mhdr);
            let mut written_len = 0;

            for ancillary_datum in ancillary_data {
                match ancillary_datum {
                    AncillaryType::Rights(received_fds) => {
                        // Reconstruct SCM_RIGHTS
                        let data_len = mem::size_of::<c_int>() * received_fds.len();
                        let space_needed = CMSG_SPACE(data_len as u32) as usize;

                        if !current_cmsg.is_null() && cmsg_space >= space_needed {
                            let cmsg = &mut *current_cmsg;
                            cmsg.cmsg_len = CMSG_LEN(data_len as u32) as usize;
                            cmsg.cmsg_level = SOL_SOCKET;
                            cmsg.cmsg_type = SCM_RIGHTS;

                            let data_ptr = CMSG_DATA(cmsg);
                            ptr::copy_nonoverlapping(
                                received_fds.as_ptr(),
                                data_ptr as *mut c_int,
                                received_fds.len(),
                            );

                            written_len += CMSG_ALIGN(cmsg.cmsg_len);
                            cmsg_space -= CMSG_ALIGN(cmsg.cmsg_len);
                            current_cmsg = CMSG_NXTHDR(mhdr, current_cmsg);
                        }
                    }
                    AncillaryType::Credentials(cred) => {
                        // Reconstruct SCM_CREDENTIALS
                        let data_len = mem::size_of::<ucred>();
                        let space_needed = CMSG_SPACE(data_len as u32) as usize;

                        if !current_cmsg.is_null() && cmsg_space >= space_needed {
                            let cmsg = &mut *current_cmsg;
                            cmsg.cmsg_len = CMSG_LEN(data_len as u32) as usize;
                            cmsg.cmsg_level = SOL_SOCKET;
                            cmsg.cmsg_type = SCM_CREDENTIALS;

                            let data_ptr = CMSG_DATA(cmsg);
                            *(data_ptr as *mut ucred) = cred;

                            written_len += CMSG_ALIGN(cmsg.cmsg_len);
                        }
                    }
                }
            }
            mhdr.msg_controllen = written_len;
        }

        // 4. Read the message body into iov.
        let mut bytes_read_to_iov = 0;
        if mhdr.msg_iovlen > 0 {
            let iovs = slice::from_raw_parts_mut(mhdr.msg_iov, mhdr.msg_iovlen as usize);
            for iov in iovs {
                if !iov.iov_base.is_null() && iov.iov_len > 0 {
                    let data_slice =
                        slice::from_raw_parts_mut(iov.iov_base as *mut u8, iov.iov_len);
                    let read = Self::read(socket, data_slice)?;
                    bytes_read_to_iov += read;
                    if read < iov.iov_len {
                        break;
                    }
                }
            }
        }

        Ok(bytes_read_to_iov)
    }

    unsafe fn sendmsg(socket: c_int, msg: *const msghdr, flags: c_int) -> Result<usize> {
        if msg.is_null() {
            return Err(Errno(EINVAL));
        }
        let mhdr = &*msg;

        let mut fds_to_send: Vec<c_int> = Vec::new();
        let mut ancillary_data_buffer: Vec<u8> = Vec::new();

        // 1. Process Control Messages from msghdr and serialize them.
        if mhdr.msg_controllen > 0 {
            if mhdr.msg_control.is_null() {
                return Err(Errno(EINVAL));
            }

            let mut cmsg: *mut cmsghdr = CMSG_FIRSTHDR(msg);
            while !cmsg.is_null() {
                let current_cmsg = &*cmsg;
                if current_cmsg.cmsg_len < CMSG_ALIGN(mem::size_of::<cmsghdr>()) {
                    return Err(Errno(EINVAL));
                }

                match (current_cmsg.cmsg_level, current_cmsg.cmsg_type) {
                    (SOL_SOCKET, SCM_RIGHTS) => {
                        let data_len =
                            current_cmsg.cmsg_len - CMSG_ALIGN(mem::size_of::<cmsghdr>());
                        if data_len % mem::size_of::<c_int>() != 0 {
                            return Err(Errno(EINVAL));
                        }
                        let fd_count = data_len / mem::size_of::<c_int>();

                        if fd_count > 0 {
                            let fds_ptr = CMSG_DATA(cmsg) as *const c_int;
                            let fds_slice = slice::from_raw_parts(fds_ptr, fd_count);
                            fds_to_send.extend_from_slice(fds_slice);
                        }

                        // Serialize: [cmsg_level, cmsg_type, number of fds]
                        ancillary_data_buffer
                            .extend_from_slice(&current_cmsg.cmsg_level.to_le_bytes());
                        ancillary_data_buffer
                            .extend_from_slice(&current_cmsg.cmsg_type.to_le_bytes());
                        ancillary_data_buffer.extend_from_slice(&(fd_count as u32).to_le_bytes());
                    }
                    (SOL_SOCKET, SCM_CREDENTIALS) => {
                        // Serialize: [cmsg_level, cmsg_type]
                        // The actual credentials will be fetched by the receiver from the kernel
                        ancillary_data_buffer
                            .extend_from_slice(&current_cmsg.cmsg_level.to_le_bytes());
                        ancillary_data_buffer
                            .extend_from_slice(&current_cmsg.cmsg_type.to_le_bytes());
                    }
                    _ => {
                        eprintln!("sendmsg: Unsupported cmsg level or type");
                        return Err(Errno(EOPNOTSUPP));
                    }
                }
                cmsg = CMSG_NXTHDR(msg, cmsg);
            }
        }

        // 2. Send FDs using the special syscall.
        for fd in fds_to_send {
            syscall::sendfd(socket as usize, fd as usize, 0, 0)?;
        }

        // 3. Send the serialized ancillary data.
        // 3.1. Send the size of the ancillary data buffer first.
        let ancillary_len: usize = ancillary_data_buffer.len();
        Self::write(socket, &ancillary_len.to_le_bytes())?;

        // 3.2. Send the ancillary data buffer itself, if any.
        if ancillary_len > 0 {
            Self::write(socket, &ancillary_data_buffer)?;
        }

        // 4. Send the actual message data from iov.
        let mut bytes_sent_from_iov = 0;
        if mhdr.msg_iovlen > 0 {
            let iovs = slice::from_raw_parts(mhdr.msg_iov, mhdr.msg_iovlen as usize);
            for iov in iovs {
                if !iov.iov_base.is_null() && iov.iov_len > 0 {
                    let data_slice = slice::from_raw_parts(iov.iov_base as *const u8, iov.iov_len);
                    let written = Self::write(socket, data_slice)?;
                    bytes_sent_from_iov += written;
                }
            }
        }

        Ok(bytes_sent_from_iov)
    }

    unsafe fn sendto(
        socket: c_int,
        buf: *const c_void,
        len: size_t,
        flags: c_int,
        dest_addr: *const sockaddr,
        dest_len: socklen_t,
    ) -> Result<usize> {
        if flags != 0 {
            return Err(Errno(EOPNOTSUPP));
        }
        if dest_addr == ptr::null() || dest_len == 0 {
            Self::write(socket, slice::from_raw_parts(buf as *const u8, len))
        } else {
            let fd = FdGuard::new(bind_or_connect!(connect copy, socket, dest_addr, dest_len)?);
            Self::write(*fd as c_int, slice::from_raw_parts(buf as *const u8, len))
        }
    }

    unsafe fn setsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: *const c_void,
        option_len: socklen_t,
    ) -> Result<()> {
        let set_timeout = |timeout_name: &[u8]| -> Result<()> {
            if option_value.is_null() {
                return Err(Errno(EFAULT));
            }

            if (option_len as usize) < mem::size_of::<timeval>() {
                return Err(Errno(EINVAL));
            }

            let timeval = unsafe { &*(option_value as *const timeval) };

            let fd = FdGuard::new(syscall::dup(socket as usize, timeout_name)?);

            let Some(tv_nsec) = timeval.tv_usec.checked_mul(1000) else {
                return Err(Errno(EDOM));
            };

            let timespec = syscall::TimeSpec {
                tv_sec: timeval.tv_sec as i64,
                tv_nsec,
            };

            Self::write(*fd as c_int, &timespec)?;
            Ok(())
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
        Ok(())
    }

    fn shutdown(socket: c_int, how: c_int) -> Result<()> {
        eprintln!("shutdown({}, {})", socket, how);
        Err(Errno(ENOSYS))
    }

    unsafe fn socket(domain: c_int, kind: c_int, protocol: c_int) -> Result<c_int> {
        if domain != AF_INET && domain != AF_UNIX {
            return Err(Errno(EAFNOSUPPORT));
        }
        // if protocol != 0 {
        //     ERRNO.set(syscall::EPROTONOSUPPORT);
        //     return -1;
        // }

        let (kind, flags) = socket_kind(kind);

        // The tcp: and udp: schemes allow using no path,
        // and later specifying one using `dup`.
        Ok(match (domain, kind) {
            (AF_INET, SOCK_STREAM) => syscall::open("/scheme/tcp", flags)? as c_int,
            (AF_INET, SOCK_DGRAM) => syscall::open("/scheme/udp", flags)? as c_int,
            (AF_UNIX, SOCK_STREAM) => {
                syscall::open("/scheme/uds_stream", flags | O_CREAT)? as c_int
            }
            (AF_UNIX, SOCK_DGRAM) => syscall::open("/scheme/uds_dgram", flags | O_CREAT)? as c_int,
            _ => return Err(Errno(EPROTONOSUPPORT)),
        })
    }

    fn socketpair(domain: c_int, kind: c_int, protocol: c_int, sv: &mut [c_int; 2]) -> Result<()> {
        let (kind, flags) = socket_kind(kind);

        match (domain, kind) {
            (AF_UNIX, SOCK_STREAM) => {
                let listener = FdGuard::new(syscall::open("/scheme/uds_stream", flags | O_CREAT)?);

                // For now, uds_stream: lets connects be instant, and instead blocks
                // on any I/O performed. So we don't need to mark this as
                // nonblocking.

                let mut fd0 = FdGuard::new(syscall::dup(*listener, b"connect")?);

                let mut fd1 = FdGuard::new(syscall::dup(*listener, b"listen")?);

                sv[0] = fd0.take() as c_int;
                sv[1] = fd1.take() as c_int;
                Ok(())
            }
            (AF_UNIX, SOCK_DGRAM) => {
                let listener = FdGuard::new(syscall::open("/scheme/uds_dgram", flags | O_CREAT)?);

                // For now, uds_dgram: lets connects be instant, and instead blocks
                // on any I/O performed. So we don't need to mark this as
                // nonblocking.

                let mut fd0 = FdGuard::new(syscall::dup(*listener, b"connect")?);

                sv[0] = fd0.take() as c_int;
                sv[1] = listener.take() as c_int;
                Ok(())
            }
            _ => unsafe {
                eprintln!(
                    "socketpair({}, {}, {}, {:p})",
                    domain,
                    kind,
                    protocol,
                    sv.as_mut_ptr()
                );
                Err(Errno(EPROTONOSUPPORT))
            },
        }
    }
}
