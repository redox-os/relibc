use alloc::{string::String, vec::Vec};
use core::{cmp, mem, ptr, slice, str};
use redox_rt::{
    proc::FdGuard,
    protocol::{FsCall, SocketCall},
};
use syscall::{self, flag::*};

use super::{
    super::{ERRNO, Pal, PalSocket, types::*},
    Sys,
    path::dir_path_and_fd_path,
};
use crate::{
    error::{Errno, Result, ResultExt},
    header::{
        arpa_inet::inet_aton,
        errno::{
            EAFNOSUPPORT, EDOM, EFAULT, EINVAL, EISCONN, EMSGSIZE, ENOMEM, ENOSYS, ENOTCONN,
            ENOTSOCK, EOPNOTSUPP, EPROTONOSUPPORT,
        },
        netinet_in::{in_addr, in_port_t, sockaddr_in},
        string::strnlen,
        sys_socket::{
            CMSG_ALIGN, CMSG_DATA, CMSG_FIRSTHDR, CMSG_LEN, CMSG_NXTHDR, CMSG_SPACE, cmsghdr,
            constants::*, msghdr, sa_family_t, sockaddr, socklen_t, ucred,
        },
        sys_time::timeval,
        sys_uio::iovec,
        sys_un::sockaddr_un,
    },
};

unsafe fn bind_or_connect(
    op: SocketCall,
    socket: c_int,
    address: *const sockaddr,
    address_len: socklen_t,
) -> Result<usize, Errno> {
    if (address_len as usize) < mem::size_of::<sa_family_t>() {
        return Err(Errno(EINVAL));
    }

    let path = match unsafe { (*address).sa_family } as c_int {
        AF_INET => {
            if (address_len as usize) != mem::size_of::<sockaddr_in>() {
                return Err(Errno(EINVAL));
            }

            let data = unsafe { &*(address as *const sockaddr_in) };
            let addr = unsafe {
                slice::from_raw_parts(
                    &data.sin_addr.s_addr as *const _ as *const u8,
                    mem::size_of_val(&data.sin_addr.s_addr),
                )
            };
            let port = in_port_t::from_be(data.sin_port);

            match op {
                SocketCall::Bind => {
                    format!("/{}.{}.{}.{}:{}", addr[0], addr[1], addr[2], addr[3], port)
                }
                SocketCall::Connect => {
                    format!("{}.{}.{}.{}:{}", addr[0], addr[1], addr[2], addr[3], port)
                }
                _ => unreachable!(),
            }
        }
        AF_UNIX => {
            eprintln!("bind/connect with AF_UNIX were replaced with SYS_CALL.");
            return Err(Errno(EAFNOSUPPORT));
        }
        _ => return Err(Errno(EAFNOSUPPORT)),
    };
    let fd = syscall::dup(socket as usize, path.as_bytes())?;
    Ok(fd)
}

pub unsafe fn bind_or_connect_into(
    op: SocketCall,
    socket: c_int,
    address: *const sockaddr,
    address_len: socklen_t,
) -> Result<c_int, Errno> {
    // Duplicate the socket, and then duplicate the copy back to the original fd
    let fd = FdGuard::new(unsafe { bind_or_connect(op, socket, address, address_len) }?);
    syscall::dup2(fd.as_raw_fd(), socket as usize, &[])?;
    Ok(0)
}

unsafe fn inner_af_unix(buf: &[u8], address: *mut sockaddr, address_len: *mut socklen_t) {
    let data = unsafe { &mut *(address as *mut sockaddr_un) };

    data.sun_family = AF_UNIX as c_ushort;

    let path = unsafe {
        slice::from_raw_parts_mut(&mut data.sun_path as *mut _ as *mut u8, data.sun_path.len())
    };

    let len = cmp::min(path.len(), buf.len());
    path[..len].copy_from_slice(&buf[..len]);
    if len < path.len() {
        path[len] = 0;
    }

    unsafe { *address_len = len as socklen_t };
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
        unsafe { inet_aton(raw_addr.as_ptr() as *mut c_char, &mut addr) },
        1,
        "inet_aton might be broken, failed to parse netstack address"
    );

    let ret = sockaddr_in {
        sin_family: AF_INET as sa_family_t,
        sin_port: in_port_t::to_be(port),
        sin_addr: addr,

        ..sockaddr_in::default()
    };
    let len = cmp::min(unsafe { *address_len } as usize, mem::size_of_val(&ret));

    unsafe {
        ptr::copy_nonoverlapping(&ret as *const _ as *const u8, address as *mut u8, len);
        *address_len = len as socklen_t;
    }
}

unsafe fn inner_get_name_inner(
    local: bool,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
    buf: &[u8],
) -> Result<()> {
    if buf.starts_with(b"tcp:") || buf.starts_with(b"udp:") {
        unsafe { inner_af_inet(local, &buf[4..], address, address_len) };
    } else if buf.starts_with(b"/scheme/tcp/") || buf.starts_with(b"/scheme/udp/") {
        unsafe { inner_af_inet(local, &buf[12..], address, address_len) };
    } else if buf.starts_with(b"chan:") {
        unsafe { inner_af_unix(&buf[5..], address, address_len) };
    } else if buf.starts_with(b"/scheme/chan/") {
        unsafe { inner_af_unix(&buf[13..], address, address_len) };
    } else if buf.starts_with(b"/scheme/uds_stream/") {
        unsafe { inner_af_unix(&buf[19..], address, address_len) };
    } else if buf.starts_with(b"/scheme/uds_dgram/") {
        unsafe { inner_af_unix(&buf[18..], address, address_len) };
    } else {
        // Socket doesn't belong to any scheme
        trace!(
            "socket {:?} doesn't match either tcp, udp or chan schemes",
            str::from_utf8(buf)
        );
        return Err(Errno(ENOTSOCK));
    }
    Ok(())
}

fn socket_domain_type(socket: c_int) -> Result<(c_int, c_int)> {
    let mut buf = [0; 256];
    let len = syscall::fpath(socket as usize, &mut buf)?;
    Ok(
        if buf.starts_with(b"tcp:") || buf.starts_with(b"/scheme/tcp/") {
            (AF_INET, SOCK_STREAM)
        } else if buf.starts_with(b"udp:") || buf.starts_with(b"/scheme/udp/") {
            (AF_INET, SOCK_DGRAM)
        } else if buf.starts_with(b"/scheme/uds_stream/") {
            (AF_UNIX, SOCK_STREAM)
        } else if buf.starts_with(b"/scheme/uds_dgram/") {
            (AF_UNIX, SOCK_DGRAM)
        } else {
            return Err(Errno(ENOTSOCK));
        },
    )
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

unsafe fn serialize_payload_to_stream(
    msg_stream: &mut Vec<u8>,
    iovs: &[iovec],
    whole_iov_size: usize,
) -> Result<usize> {
    msg_stream.extend_from_slice(&whole_iov_size.to_le_bytes());

    for iov in iovs {
        if iov.iov_len > 0 {
            if iov.iov_base.is_null() {
                return Err(Errno(EFAULT));
            }
            let source_slice: &[u8] =
                unsafe { slice::from_raw_parts(iov.iov_base as *const u8, iov.iov_len) };
            msg_stream.extend_from_slice(source_slice);
        }
    }
    Ok(whole_iov_size)
}

unsafe fn serialize_ancillary_data_to_stream(
    msg: *const msghdr,
    mhdr: &msghdr,
    socket: c_int,
    msg_stream: &mut Vec<u8>,
) -> Result<()> {
    if mhdr.msg_control.is_null() {
        return Err(Errno(EINVAL));
    }

    let mut cmsg: *mut cmsghdr = unsafe { CMSG_FIRSTHDR(msg) };
    let mut cmsg_count = 0;
    while !cmsg.is_null() {
        cmsg_count += 1;
        let current_cmsg = unsafe { &*cmsg };
        let min_cmsg_len = unsafe { CMSG_ALIGN(mem::size_of::<cmsghdr>()) };
        if current_cmsg.cmsg_len < min_cmsg_len {
            return Err(Errno(EINVAL));
        }

        // cmsg entry format: [level(i32)][type(i32)][data_len(usize)][data]
        msg_stream.extend_from_slice(&current_cmsg.cmsg_level.to_le_bytes());
        msg_stream.extend_from_slice(&current_cmsg.cmsg_type.to_le_bytes());

        match (current_cmsg.cmsg_level, current_cmsg.cmsg_type) {
            (SOL_SOCKET, SCM_RIGHTS) => {
                let data_len = current_cmsg.cmsg_len - min_cmsg_len;
                if data_len % mem::size_of::<c_int>() != 0 {
                    return Err(Errno(EINVAL));
                }
                let fd_count = data_len / mem::size_of::<c_int>();

                // Call syscall::sendfd for each fd.
                if fd_count > 0 {
                    let fds_ptr = unsafe { CMSG_DATA(cmsg) } as *const c_int;
                    let fds_slice = unsafe { slice::from_raw_parts(fds_ptr, fd_count) };
                    for &fd in fds_slice.iter() {
                        let fd_to_send = FdGuard::new(syscall::dup(fd as usize, b"")?);
                        syscall::sendfd(socket as usize, fd_to_send.as_raw_fd(), 0, 0)?;
                    }
                }

                // Serialize to ancillary_data_stream.
                // Our intermediate format: data_len is size of fd_count (usize), data is fd_count (usize)
                let data_for_stream_len = mem::size_of::<usize>();
                let data_for_stream_payload = (fd_count as usize).to_le_bytes();

                msg_stream.extend_from_slice(&(data_for_stream_len as usize).to_le_bytes());
                msg_stream.extend_from_slice(&data_for_stream_payload);
            }
            (SOL_SOCKET, SCM_CREDENTIALS) => {
                // Our intermediate format: data_len is 0, no data payload
                let data_for_stream_len = 0usize;
                msg_stream.extend_from_slice(&(data_for_stream_len as usize).to_le_bytes());
            }
            _ => {
                return Err(Errno(EOPNOTSUPP));
            }
        }
        cmsg = unsafe { CMSG_NXTHDR(msg, cmsg) };
    }
    Ok(())
}

unsafe fn deserialize_name_from_stream(
    mhdr: &mut msghdr,
    msg_stream: &[u8],
    cursor: &mut usize,
) -> Result<()> {
    // Read name_len from stream
    let name_len_in_stream = read_num::<usize>(&msg_stream[*cursor..])?;
    let name_len = cmp::min(name_len_in_stream, mhdr.msg_namelen as usize);
    *cursor += mem::size_of::<usize>();

    if name_len > 0 {
        if *cursor + name_len > msg_stream.len() {
            return Err(Errno(EMSGSIZE));
        }
        if !mhdr.msg_name.is_null() && mhdr.msg_namelen > 0 {
            let name_buffer = &msg_stream[*cursor..*cursor + name_len];
            (unsafe {
                inner_get_name_inner(
                    false,
                    mhdr.msg_name as *mut sockaddr,
                    &mut mhdr.msg_namelen,
                    name_buffer,
                )
            })?;
        }
        *cursor += name_len;
    } else {
        // If name_len is 0, set msg_namelen to 0
        mhdr.msg_namelen = 0;
    }
    Ok(())
}

unsafe fn deserialize_payload_from_stream(
    mhdr: &mut msghdr,
    msg_stream: &[u8],
    iovs: &[iovec],
    whole_iov_size: usize,
    cursor: &mut usize,
    test: u8,
) -> Result<usize> {
    let full_payload_len_from_scheme = read_num::<usize>(&msg_stream[*cursor..])?;
    *cursor += mem::size_of::<usize>();
    // Determine actual payload data available in the stream
    let payload_len_to_read = cmp::min(full_payload_len_from_scheme, whole_iov_size);
    let payload_data_from_stream = &msg_stream[*cursor..*cursor + payload_len_to_read];
    *cursor += payload_len_to_read;

    let mut total_bytes_written: usize = 0;
    if !iovs.is_empty() && payload_len_to_read > 0 {
        let mut source_bytes_consumed: usize = 0;
        for iov in iovs {
            if iov.iov_len == 0 {
                continue;
            }
            if iov.iov_base.is_null() {
                return Err(Errno(EFAULT));
            }

            let source_bytes_remaining = payload_data_from_stream
                .len()
                .saturating_sub(source_bytes_consumed);
            if source_bytes_remaining == 0 {
                break;
            }

            let bytes_to_write = cmp::min(iov.iov_len, source_bytes_remaining);
            if bytes_to_write > 0 {
                let dest_slice: &mut [u8] =
                    unsafe { slice::from_raw_parts_mut(iov.iov_base as *mut u8, iov.iov_len) };

                let source_sub_slice = &payload_data_from_stream
                    [source_bytes_consumed..source_bytes_consumed + bytes_to_write];
                dest_slice[..bytes_to_write].copy_from_slice(source_sub_slice);
                total_bytes_written += bytes_to_write;
                source_bytes_consumed += bytes_to_write;
            }
        }
    }

    if full_payload_len_from_scheme > whole_iov_size {
        mhdr.msg_flags |= MSG_TRUNC;
    }

    Ok(total_bytes_written)
}

unsafe fn deserialize_ancillary_data_from_stream(
    mhdr: &mut msghdr,
    socket: c_int,
    msg_stream: &[u8],
    cursor: &mut usize,
    cmsg_space_provided: usize,
) -> Result<()> {
    let mut current_cmsg_ptr_in_user_buf = if !mhdr.msg_control.is_null() && cmsg_space_provided > 0
    {
        unsafe { CMSG_FIRSTHDR(mhdr) }
    } else {
        ptr::null_mut()
    };
    let mut remaining_user_cmsg_buf_len = cmsg_space_provided;
    let mut total_csmg_bytes_written_to_user_buf: usize = 0;

    while *cursor < msg_stream.len() {
        const CMSG_HEADER_LEN_IN_STREAM: usize =
            mem::size_of::<c_int>() * 2 + mem::size_of::<usize>();
        if *cursor + CMSG_HEADER_LEN_IN_STREAM > msg_stream.len() {
            if msg_stream[*cursor..].iter().any(|&b| b != 0) {
                mhdr.msg_flags |= MSG_CTRUNC;
            }
            break;
        }

        // cmsg entry format: [level(i32)][type(i32)][data_len(usize)][data]
        let cmsg_level = read_num::<c_int>(&msg_stream[*cursor..])?;
        *cursor += mem::size_of::<c_int>();
        let cmsg_type = read_num::<c_int>(&msg_stream[*cursor..])?;
        *cursor += mem::size_of::<c_int>();
        let cmsg_data_len_in_stream = read_num::<usize>(&msg_stream[*cursor..])?;
        *cursor += mem::size_of::<usize>();

        if *cursor + cmsg_data_len_in_stream > msg_stream.len() {
            mhdr.msg_flags |= MSG_CTRUNC;
            break;
        }

        let cmsg_data_from_stream = &msg_stream[*cursor..*cursor + cmsg_data_len_in_stream];
        *cursor += cmsg_data_len_in_stream;

        let mut actual_posix_cmsg_data_len: usize = 0;
        let mut temp_posix_cmsg_data_buf: Vec<u8> = Vec::new();

        match (cmsg_level, cmsg_type) {
            (SOL_SOCKET, SCM_RIGHTS) => {
                if cmsg_data_len_in_stream != mem::size_of::<usize>() {
                    return Err(Errno(EINVAL));
                }
                let fd_count = read_num::<usize>(&cmsg_data_from_stream)?;

                for _ in 0..fd_count {
                    // Call syscall::dup to duplicate the fd
                    let new_fd = syscall::dup(socket as usize, b"recvfd")?;
                    temp_posix_cmsg_data_buf.extend_from_slice(&(new_fd as c_int).to_le_bytes());
                }
                actual_posix_cmsg_data_len = temp_posix_cmsg_data_buf.len();
            }
            (SOL_SOCKET, SCM_CREDENTIALS) => {
                if cmsg_data_len_in_stream
                    != mem::size_of::<pid_t>() + mem::size_of::<uid_t>() + mem::size_of::<gid_t>()
                {
                    return Err(Errno(EINVAL));
                }

                let pid = read_num::<pid_t>(&cmsg_data_from_stream)?;
                let uid_offset = mem::size_of::<pid_t>();
                let uid = read_num::<uid_t>(&cmsg_data_from_stream[uid_offset..])?;
                let gid_offset = uid_offset + mem::size_of::<uid_t>();
                let gid = read_num::<gid_t>(&cmsg_data_from_stream[gid_offset..])?;
                let cred = ucred { pid, uid, gid };

                temp_posix_cmsg_data_buf.extend_from_slice(unsafe {
                    slice::from_raw_parts(
                        &cred as *const ucred as *const u8,
                        mem::size_of::<ucred>(),
                    )
                });
                actual_posix_cmsg_data_len = temp_posix_cmsg_data_buf.len();
            }
            _ => {
                return Err(Errno(EINVAL));
            }
        }

        let space_needed_for_posix_cmsg =
            unsafe { CMSG_SPACE(actual_posix_cmsg_data_len as u32) } as usize;

        if !current_cmsg_ptr_in_user_buf.is_null()
            && remaining_user_cmsg_buf_len >= space_needed_for_posix_cmsg
        {
            let cmsg_ref = unsafe { &mut *current_cmsg_ptr_in_user_buf };
            cmsg_ref.cmsg_len = unsafe { CMSG_LEN(actual_posix_cmsg_data_len as u32) } as usize;
            cmsg_ref.cmsg_level = cmsg_level;
            cmsg_ref.cmsg_type = cmsg_type;

            let data_ptr_in_user_cmsg = unsafe { CMSG_DATA(cmsg_ref) };
            unsafe {
                ptr::copy_nonoverlapping(
                    temp_posix_cmsg_data_buf.as_ptr(),
                    data_ptr_in_user_cmsg as *mut u8,
                    actual_posix_cmsg_data_len,
                )
            };

            let aligned_len_written = unsafe { CMSG_ALIGN(cmsg_ref.cmsg_len) };
            total_csmg_bytes_written_to_user_buf += aligned_len_written;
            remaining_user_cmsg_buf_len -= aligned_len_written;
            current_cmsg_ptr_in_user_buf =
                unsafe { CMSG_NXTHDR(mhdr, current_cmsg_ptr_in_user_buf) };
        } else {
            mhdr.msg_flags |= MSG_CTRUNC;
            break;
        }
    }
    mhdr.msg_controllen = total_csmg_bytes_written_to_user_buf;
    Ok(())
}

impl PalSocket for Sys {
    unsafe fn accept(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> Result<c_int> {
        let stream = syscall::dup(socket as usize, b"listen")?;
        if address != ptr::null_mut() && address_len != ptr::null_mut() {
            if let Err(err) = unsafe { Self::getpeername(stream as c_int, address, address_len) } {
                let _ = syscall::close(stream);
                return Err(err);
            }
        }
        Ok(stream as c_int)
    }

    unsafe fn bind(socket: c_int, address: *const sockaddr, address_len: socklen_t) -> Result<()> {
        match unsafe { (*address).sa_family } as c_int {
            AF_INET => {
                (unsafe { bind_or_connect_into(SocketCall::Bind, socket, address, address_len) })?;
            }
            AF_UNIX => {
                let data = unsafe { &*(address as *const sockaddr_un) };

                // NOTE: It's UB to access data in given address that exceeds
                // the given address length.

                let maxlen = cmp::min(
                    // Max path length of the full-sized struct
                    data.sun_path.len(),
                    // Length inferred from given addrlen
                    address_len as usize - data.path_offset(),
                );
                let len = cmp::min(
                    // The maximum length of the address
                    maxlen,
                    // The first NUL byte, if any
                    unsafe { strnlen(&data.sun_path as *const _, maxlen as size_t) },
                );

                let addr =
                    unsafe { slice::from_raw_parts(&data.sun_path as *const _ as *const u8, len) };
                let path = format!("{}", str::from_utf8(addr).unwrap());
                trace!("path: {:?}", path);

                let (dir_path, mut fd_path) = dir_path_and_fd_path(&path)?;

                redox_rt::sys::sys_call(
                    socket as usize,
                    unsafe { fd_path.as_bytes_mut() },
                    CallFlags::empty(),
                    &[SocketCall::Bind as u64],
                )?;

                let fs_bind_result = (|| -> Result<()> {
                    let dirfd = FdGuard::open(
                        &dir_path,
                        syscall::O_RDONLY | syscall::O_DIRECTORY | syscall::O_CLOEXEC,
                    )?;
                    let fd_to_send = FdGuard::new(syscall::dup(socket as usize, &[])?);
                    syscall::sendfd(dirfd.as_raw_fd(), fd_to_send.as_raw_fd(), 0, 0)?;
                    Ok(())
                })();

                if let Err(original_error) = fs_bind_result {
                    if let Err(unbind_error) = redox_rt::sys::sys_call(
                        socket as usize,
                        &mut [],
                        CallFlags::empty(),
                        &[SocketCall::Unbind as u64],
                    ) {
                        eprintln!(
                            "bind: CRITICAL: failed to unbind socket after a failed transaction: {:?}",
                            unbind_error
                        );
                    }

                    return Err(original_error);
                }
            }
            _ => {
                return Err(Errno(EAFNOSUPPORT));
            }
        };

        Ok(())
    }

    unsafe fn connect(
        socket: c_int,
        address: *const sockaddr,
        address_len: socklen_t,
    ) -> Result<c_int> {
        match unsafe { (*address).sa_family } as c_int {
            AF_INET => unsafe {
                bind_or_connect_into(SocketCall::Connect, socket, address, address_len)
            },
            AF_UNIX => {
                let data = unsafe { &*(address as *const sockaddr_un) };

                // NOTE: It's UB to access data in given address that exceeds
                // the given address length.

                let maxlen = cmp::min(
                    // Max path length of the full-sized struct
                    data.sun_path.len(),
                    // Length inferred from given addrlen
                    address_len as usize - data.path_offset(),
                );
                let len = cmp::min(
                    // The maximum length of the address
                    maxlen,
                    // The first NUL byte, if any
                    unsafe { strnlen(&data.sun_path as *const _, maxlen as size_t) },
                );

                let addr =
                    unsafe { slice::from_raw_parts(&data.sun_path as *const _ as *const u8, len) };
                let mut path = format!("{}", str::from_utf8(addr).unwrap());
                trace!("path: {:?}", path);

                let (_, fd_path) = dir_path_and_fd_path(&path)?;

                let target_path = format!("/{fd_path}");
                let socket_file_fd = FdGuard::open(&target_path, syscall::O_RDWR)?;

                const TOKEN_BUF_SIZE: usize = 16;

                let mut token_buf = [0u8; TOKEN_BUF_SIZE];

                redox_rt::sys::sys_call(
                    socket_file_fd.as_raw_fd(),
                    &mut token_buf,
                    CallFlags::empty(),
                    &[FsCall::Connect as u64],
                )?;

                redox_rt::sys::sys_call(
                    socket as usize,
                    &mut token_buf,
                    CallFlags::empty(),
                    &[SocketCall::Connect as u64],
                )?;
                Result::<c_int, Errno>::Ok(0)
            }
            _ => Err(Errno(EAFNOSUPPORT)),
        }
    }

    unsafe fn getpeername(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> Result<()> {
        let mut buf = [0; 256];
        let len = redox_rt::sys::sys_call(
            socket as usize,
            &mut buf,
            CallFlags::empty(),
            &[SocketCall::GetPeerName as u64],
        )?;

        unsafe { inner_get_name_inner(false, address, address_len, &buf[..len]) }
    }

    unsafe fn getsockname(
        socket: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> Result<()> {
        let mut buf = [0; 256];
        let len = syscall::fpath(socket as usize, &mut buf)?;

        unsafe { inner_get_name_inner(true, address, address_len, &buf[..len]) }
    }

    unsafe fn getsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: *mut c_void,
        option_len_ptr: *mut socklen_t,
    ) -> Result<()> {
        if option_len_ptr.is_null() {
            return Err(Errno(EFAULT));
        }
        let option_len = (unsafe { *option_len_ptr }) as usize;

        let option_c_int = || -> Result<&mut c_int> {
            if option_value.is_null() {
                return Err(Errno(EFAULT));
            }

            if option_len < mem::size_of::<c_int>() {
                return Err(Errno(EINVAL));
            }

            Ok(unsafe { &mut *(option_value as *mut c_int) })
        };

        match level {
            SOL_SOCKET => match option_name {
                SO_DOMAIN => {
                    let option = option_c_int()?;
                    *option = socket_domain_type(socket)?.0;
                    unsafe { *option_len_ptr = mem::size_of::<c_int>() as socklen_t };
                    return Ok(());
                }
                SO_ERROR => {
                    let option = option_c_int()?;
                    //TODO: Socket nonblock connection error
                    *option = 0;
                    unsafe { *option_len_ptr = mem::size_of::<c_int>() as socklen_t };
                    return Ok(());
                }
                SO_TYPE => {
                    let option = option_c_int()?;
                    *option = socket_domain_type(socket)?.1;
                    unsafe { *option_len_ptr = mem::size_of::<c_int>() as socklen_t };
                    return Ok(());
                }
                _ => {
                    let metadata = [SocketCall::GetSockOpt as u64, option_name as u64];
                    let payload =
                        unsafe { slice::from_raw_parts_mut(option_value as *mut u8, option_len) };
                    let call_flags = CallFlags::empty();
                    unsafe {
                        *option_len_ptr = redox_rt::sys::sys_call(
                            socket as usize,
                            payload,
                            CallFlags::empty(),
                            &metadata,
                        )? as socklen_t;
                    }
                    return Ok(());
                }
            },
            _ => (),
        }

        eprintln!(
            "getsockopt({}, {}, {}, {:p}, {:p})",
            socket, level, option_name, option_value, option_len_ptr
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
            // Convert to recvmsg
            let mut iov = iovec {
                iov_base: buf,
                iov_len: len,
            };
            let mut msg = msghdr {
                msg_name: address as *mut c_void,
                msg_namelen: if !address_len.is_null() {
                    unsafe { *address_len }
                } else {
                    0
                },
                msg_iov: &mut iov,
                msg_iovlen: 1,
                msg_control: ptr::null_mut(),
                msg_controllen: 0,
                msg_flags: 0,
            };
            let count = unsafe { Self::recvmsg(socket, &mut msg, flags) }?;
            if !address_len.is_null() {
                unsafe { *address_len = msg.msg_namelen };
            }
            return Ok(count);
        }
        if address.is_null() || address_len.is_null() {
            Self::read(socket, unsafe {
                slice::from_raw_parts_mut(buf as *mut u8, len)
            })
        } else {
            // TODO: in UDS dgram getpeername on listener always return ENOTCONN,
            // it probably the expected error on usual getpeername call, but not here.
            if let Err(e) = unsafe { Self::getpeername(socket, address, address_len) } {
                if e.0 != ENOTCONN {
                    return Err(e);
                }
                let data = unsafe { &mut *(address as *mut sockaddr) };
                data.sa_family = AF_UNSPEC as u16;
                unsafe { *address_len = 0 };
            }
            Self::read(socket, unsafe {
                slice::from_raw_parts_mut(buf as *mut u8, len)
            })
        }
    }

    unsafe fn recvmsg(socket: c_int, msg: *mut msghdr, flags: c_int) -> Result<usize> {
        if msg.is_null() {
            return Err(Errno(EINVAL));
        }
        let mut mhdr = unsafe { &mut *msg };
        let iovs_slice: &[iovec] = if mhdr.msg_iov.is_null() || mhdr.msg_iovlen == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(mhdr.msg_iov, mhdr.msg_iovlen as usize) }
        };
        let whole_iov_size: usize = iovs_slice.iter().map(|iov| iov.iov_len).sum();

        let mut msg_stream: Vec<u8> = Vec::new();

        // Prepare space for the message stream.
        // [name_len(usize)][name_buffer]
        // [payload_len(usize)][payload_data_buffer]
        // [ancillary_stream_buffer]
        let expected_stream_size = {
            mem::size_of::<usize>()         // name_len
            + mhdr.msg_namelen as usize     // name_buffer
            + mem::size_of::<usize>()       // payload_len
            + whole_iov_size                // payload_data_buffer
            + mem::size_of::<usize>()       // control_len
            + mhdr.msg_controllen as usize // ancillary_stream_buffer
        };
        msg_stream
            .try_reserve_exact(expected_stream_size)
            .map_err(|_| Errno(ENOMEM))?;
        msg_stream.resize(expected_stream_size, 0);

        // Write the information about the msghdr
        let mut cursor: usize = 0;
        msg_stream[cursor..cursor + mem::size_of::<usize>()]
            .copy_from_slice(&(mhdr.msg_namelen as usize).to_le_bytes());
        cursor += mem::size_of::<usize>();
        msg_stream[cursor..cursor + mem::size_of::<usize>()]
            .copy_from_slice(&(whole_iov_size).to_le_bytes());
        cursor += mem::size_of::<usize>();
        msg_stream[cursor..cursor + mem::size_of::<usize>()]
            .copy_from_slice(&(mhdr.msg_controllen as usize).to_le_bytes());

        // Read the message stream.
        let metadata = [SocketCall::RecvMsg as u64, flags as u64];
        let call_flags = CallFlags::empty();
        let actual_read_len =
            redox_rt::sys::sys_call(socket as usize, &mut msg_stream, call_flags, &metadata)?;
        msg_stream.truncate(actual_read_len);

        cursor = 0;
        let cmsg_space_provided_by_user = mhdr.msg_controllen;
        mhdr.msg_flags = 0;

        // Read sender name.
        (unsafe { deserialize_name_from_stream(&mut mhdr, &msg_stream, &mut cursor) })?;

        // Read payload data.
        let actual_payload_bytes_written_to_iov = unsafe {
            deserialize_payload_from_stream(
                &mut mhdr,
                &msg_stream,
                iovs_slice,
                whole_iov_size,
                &mut cursor,
                0u8,
            )
        }?;

        // Reconstruct the ancillary data in the user-provided buffer.
        let has_cmsg_buffer = !mhdr.msg_control.is_null() && cmsg_space_provided_by_user > 0;
        let has_ancillary_data = cursor < msg_stream.len();
        if has_cmsg_buffer && has_ancillary_data {
            (unsafe {
                deserialize_ancillary_data_from_stream(
                    mhdr,
                    socket,
                    &msg_stream,
                    &mut cursor,
                    cmsg_space_provided_by_user as usize,
                )
            })?;
        } else {
            mhdr.msg_controllen = 0; // No ancillary data
        }
        Ok(actual_payload_bytes_written_to_iov)
    }

    unsafe fn sendmsg(socket: c_int, msg: *const msghdr, flags: c_int) -> Result<usize> {
        if msg.is_null() {
            return Err(Errno(EINVAL));
        }
        let mhdr = unsafe { &*msg };

        // Reserve space for the message stream.
        // [payload_len(usize)][payload_data_buffer]
        // [ancillary_stream_buffer]
        let iovs_slice: &[iovec] = if mhdr.msg_iov.is_null() || mhdr.msg_iovlen == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(mhdr.msg_iov, mhdr.msg_iovlen as usize) }
        };

        let mut msg_stream: Vec<u8> = Vec::new();
        let whole_iov_size: usize = iovs_slice.iter().map(|iov| iov.iov_len).sum();
        msg_stream
            .try_reserve_exact(
                mem::size_of::<usize>()     // payload_len
            + whole_iov_size                // payload_data_buffer
            + mhdr.msg_controllen as usize, // ancillary_stream_buffer
            )
            .map_err(|_| Errno(ENOMEM))?;

        // Write the message to the msg_stream.
        let mut actual_payload_bytes_serialized = 0;
        if !mhdr.msg_iov.is_null() && mhdr.msg_iovlen > 0 {
            actual_payload_bytes_serialized = unsafe {
                serialize_payload_to_stream(&mut msg_stream, &iovs_slice, whole_iov_size)
            }?;
        }
        // Process Control Messages from msghdr and serialize them.
        if mhdr.msg_controllen > 0 {
            (unsafe { serialize_ancillary_data_to_stream(msg, mhdr, socket, &mut msg_stream) })?;
        }

        // Send the message stream.
        let metadata = [SocketCall::SendMsg as u64, flags as u64];
        let call_flags = CallFlags::empty();
        let written = redox_rt::sys::sys_call(
            socket as usize,
            msg_stream.as_mut_slice(),
            call_flags,
            &metadata,
        )?;

        Ok(actual_payload_bytes_serialized)
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
            // Convert to sendmsg
            let mut iov = iovec {
                iov_base: buf as *mut c_void,
                iov_len: len,
            };
            let msg = msghdr {
                msg_name: dest_addr as *mut c_void,
                msg_namelen: dest_len,
                msg_iov: &mut iov,
                msg_iovlen: 1,
                msg_control: ptr::null_mut(),
                msg_controllen: 0,
                msg_flags: 0,
            };
            return unsafe { Self::sendmsg(socket, &msg, flags) };
        }
        if dest_addr == ptr::null() || dest_len == 0 {
            Self::write(socket, unsafe {
                slice::from_raw_parts(buf as *const u8, len)
            })
        } else {
            let fd = FdGuard::new(unsafe {
                bind_or_connect(SocketCall::Connect, socket, dest_addr, dest_len)
            }?);
            Self::write(fd.as_c_fd().unwrap(), unsafe {
                slice::from_raw_parts(buf as *const u8, len)
            })
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

            Self::write(fd.as_c_fd().unwrap(), &timespec)?;
            Ok(())
        };

        match level {
            SOL_SOCKET => match option_name {
                SO_RCVTIMEO => return set_timeout(b"read_timeout"),
                SO_SNDTIMEO => return set_timeout(b"write_timeout"),
                _ => {
                    let metadata = [SocketCall::SetSockOpt as u64, option_name as u64];
                    let payload = unsafe {
                        slice::from_raw_parts_mut(option_value as *mut u8, option_len as usize)
                    };
                    let call_flags = CallFlags::empty();
                    redox_rt::sys::sys_call(
                        socket as usize,
                        payload,
                        CallFlags::empty(),
                        &metadata,
                    )?;
                    return Ok(());
                }
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
        let metadata = [SocketCall::Shutdown as u64, how as u64];
        redox_rt::sys::sys_call(socket as usize, &mut [], CallFlags::empty(), &metadata)?;
        Ok(())
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
                let listener = FdGuard::open("/scheme/uds_stream", flags | O_CREAT)?;

                // For now, uds_stream: lets connects be instant, and instead blocks
                // on any I/O performed. So we don't need to mark this as
                // nonblocking.

                let mut fd0 = listener.dup(b"connect")?;
                let mut fd1 = listener.dup(b"listen")?;

                sv[0] = fd0.take() as c_int;
                sv[1] = fd1.take() as c_int;
                Ok(())
            }
            (AF_UNIX, SOCK_DGRAM) => {
                let listener = FdGuard::open("/scheme/uds_dgram", flags | O_CREAT)?;

                // For now, uds_dgram: lets connects be instant, and instead blocks
                // on any I/O performed. So we don't need to mark this as
                // nonblocking.

                let mut fd0 = listener.dup(b"connect")?;

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

fn read_num<T>(buffer: &[u8]) -> Result<T>
where
    T: NumFromBytes,
{
    T::from_le_bytes_slice(buffer)
}
trait NumFromBytes: Sized {
    fn from_le_bytes_slice(buffer: &[u8]) -> Result<Self>;
}
impl NumFromBytes for i32 {
    fn from_le_bytes_slice(buffer: &[u8]) -> Result<Self> {
        Ok(i32::from_le_bytes(
            buffer
                .get(..mem::size_of::<i32>())
                .and_then(|slice| slice.try_into().ok())
                .ok_or_else(|| Errno(EFAULT))?,
        ))
    }
}
impl NumFromBytes for usize {
    fn from_le_bytes_slice(buffer: &[u8]) -> Result<Self> {
        Ok(usize::from_le_bytes(
            buffer
                .get(..mem::size_of::<usize>())
                .and_then(|slice| slice.try_into().ok())
                .ok_or_else(|| Errno(EFAULT))?,
        ))
    }
}
