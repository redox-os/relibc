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
        errno::{
            EAFNOSUPPORT, EDOM, EFAULT, EINVAL, EISCONN, EMSGSIZE, ENOMEM, ENOSYS, EOPNOTSUPP,
            EPROTONOSUPPORT,
        },
        netinet_in::{in_addr, in_port_t, sockaddr_in},
        string::strnlen,
        sys_socket::{
            cmsghdr, constants::*, msghdr, sa_family_t, sockaddr, socklen_t, ucred, CMSG_ALIGN,
            CMSG_DATA, CMSG_FIRSTHDR, CMSG_LEN, CMSG_NXTHDR, CMSG_SPACE,
        },
        sys_time::timeval,
        sys_uio::iovec,
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

    inner_get_name_inner(local, address, address_len, &buf[..len]);

    Ok(())
}

unsafe fn inner_get_name_inner(
    local: bool,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
    buf: &[u8],
) {
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

unsafe fn gather_data_from_iovs(iovs: &[iovec], target_buffer: &mut Vec<u8>) -> Result<usize> {
    let initial_len = target_buffer.len();
    eprintln!(
        "[DEBUG] gather_data_from_iovs: initial target_buffer.len() = {}, num_iovs = {}",
        initial_len,
        iovs.len()
    );

    for (i, iov) in iovs.iter().enumerate() {
        if iov.iov_len == 0 {
            eprintln!(
                "[DEBUG] gather_data_from_iovs: iov #{} has len 0, skipping.",
                i
            );
            continue;
        }
        if iov.iov_base.is_null() {
            eprintln!(
                "[ERROR] gather_data_from_iovs: iov #{} has null base with len {}",
                i, iov.iov_len
            );
            return Err(Errno(EFAULT));
        }
        eprintln!(
            "[DEBUG] gather_data_from_iovs: iov #{}: base={:p}, len={}",
            i, iov.iov_base, iov.iov_len
        );
        let source_slice: &[u8] =
            unsafe { slice::from_raw_parts(iov.iov_base as *const u8, iov.iov_len) };
        target_buffer.extend_from_slice(source_slice);
    }
    let bytes_gathered = target_buffer.len() - initial_len;
    eprintln!(
        "[DEBUG] gather_data_from_iovs: gathered {} bytes. final target_buffer.len() = {}",
        bytes_gathered,
        target_buffer.len()
    );
    Ok(bytes_gathered)
}

unsafe fn scatter_data_into_iovs(iovs: &[iovec], source_data: &[u8]) -> Result<usize> {
    eprintln!(
        "[DEBUG] scatter_data_into_iovs: num_iovs = {}, source_data.len() = {}",
        iovs.len(),
        source_data.len()
    );
    if source_data.is_empty() {
        eprintln!("[DEBUG] scatter_data_into_iovs: source_data is empty, returning 0.");
        return Ok(0);
    }

    let mut total_bytes_written: usize = 0;
    let mut source_bytes_consumed: usize = 0;

    for (i, iov) in iovs.iter().enumerate() {
        if iov.iov_len == 0 {
            eprintln!(
                "[DEBUG] scatter_data_into_iovs: iov #{} has len 0, skipping.",
                i
            );
            continue;
        }
        if iov.iov_base.is_null() {
            eprintln!(
                "[ERROR] scatter_data_into_iovs: iov #{} has null base with len {}",
                i, iov.iov_len
            );
            return Err(Errno(EFAULT));
        }

        let source_bytes_remaining = source_data.len().saturating_sub(source_bytes_consumed);
        if source_bytes_remaining == 0 {
            eprintln!("[DEBUG] scatter_data_into_iovs: source_data exhausted.");
            break;
        }

        let bytes_to_write = cmp::min(iov.iov_len, source_bytes_remaining);
        eprintln!(
            "[DEBUG] scatter_data_into_iovs: iov #{}: base={:p}, len={}, will_write={}",
            i, iov.iov_base, iov.iov_len, bytes_to_write
        );

        if bytes_to_write > 0 {
            let dest_slice: &mut [u8] =
                unsafe { slice::from_raw_parts_mut(iov.iov_base as *mut u8, iov.iov_len) };

            let source_sub_slice =
                &source_data[source_bytes_consumed..source_bytes_consumed + bytes_to_write];

            dest_slice[..bytes_to_write].copy_from_slice(source_sub_slice);

            total_bytes_written += bytes_to_write;
            source_bytes_consumed += bytes_to_write;
        }
    }
    eprintln!(
        "[DEBUG] scatter_data_into_iovs: total_bytes_written = {}. source_bytes_consumed = {}",
        total_bytes_written, source_bytes_consumed
    );
    Ok(total_bytes_written)
}

unsafe fn serialize_payload_to_stream(
    msg_stream: &mut Vec<u8>,
    iovs_slice: &[iovec],
    whole_iov_size: usize,
) -> Result<usize> {
    eprintln!(
        "[DEBUG] serialize_payload_to_stream: target_len = {}, num_iovs = {}",
        whole_iov_size,
        iovs_slice.len()
    );
    let bytes_written = gather_data_from_iovs(iovs_slice, msg_stream)?;
    eprintln!(
        "[DEBUG] serialize_payload_to_stream: gathered {} payload bytes",
        bytes_written
    );

    if bytes_written != whole_iov_size {
        return Err(Errno(EFAULT));
        eprintln!(
            "[ERROR] serialize_payload_to_stream: gathered_bytes ({}) != whole_iov_size ({})",
            bytes_written, whole_iov_size
        );
    }

    assert!(
        msg_stream.len() >= mem::size_of::<usize>(),
        "msg_stream should have enough space for usize"
    );

    msg_stream[0..mem::size_of::<usize>()].copy_from_slice(&(bytes_written).to_le_bytes());

    Ok(bytes_written)
}

unsafe fn serialize_ancillary_data_to_stream(
    msg: *const msghdr,
    mhdr: &msghdr,
    socket_to_use: &FdGuard,
    msg_stream: &mut Vec<u8>,
) -> Result<()> {
    eprintln!(
        "[DEBUG] serialize_ancillary_data_to_stream: msg_controllen = {}",
        mhdr.msg_controllen
    );
    if mhdr.msg_control.is_null() {
        eprintln!("[DEBUG] serialize_ancillary_data_to_stream: No control message.");
        return Err(Errno(EINVAL));
    }

    let mut cmsg: *mut cmsghdr = CMSG_FIRSTHDR(msg);
    let mut cmsg_count = 0;
    while !cmsg.is_null() {
        cmsg_count += 1;
        let current_cmsg = &*cmsg;
        let min_cmsg_len = CMSG_ALIGN(mem::size_of::<cmsghdr>());
        eprintln!("[DEBUG] serialize_ancillary_data_to_stream: Processing cmsg #{}: ptr={:p}, cmsg_len={}, level={}, type={}",
            cmsg_count, cmsg, current_cmsg.cmsg_len, current_cmsg.cmsg_level, current_cmsg.cmsg_type);
        if current_cmsg.cmsg_len < min_cmsg_len {
            eprintln!("[ERROR] serialize_ancillary_data_to_stream: cmsg_len is too small.");
            return Err(Errno(EINVAL));
        }

        msg_stream.extend_from_slice(&current_cmsg.cmsg_level.to_le_bytes());
        msg_stream.extend_from_slice(&current_cmsg.cmsg_type.to_le_bytes());

        // cmsg entry format: [level(i32)][type(i32)][data_len(usize)][data]
        match (current_cmsg.cmsg_level, current_cmsg.cmsg_type) {
            (SOL_SOCKET, SCM_RIGHTS) => {
                let data_len = current_cmsg.cmsg_len - min_cmsg_len;
                if data_len % mem::size_of::<c_int>() != 0 {
                    eprintln!("[ERROR] serialize_ancillary_data_to_stream: SCM_RIGHTS data_len not multiple of c_int size.");
                    return Err(Errno(EINVAL));
                }
                let fd_count = data_len / mem::size_of::<c_int>();
                eprintln!(
                    "[DEBUG] serialize_ancillary_data_to_stream: SCM_RIGHTS, fd_count = {}",
                    fd_count
                );

                // 3.1. Call syscall::sendfd for each fd.
                if fd_count > 0 {
                    let fds_ptr = CMSG_DATA(cmsg) as *const c_int;
                    let fds_slice = slice::from_raw_parts(fds_ptr, fd_count);
                    for (i, &fd) in fds_slice.iter().enumerate() {
                        eprintln!("[DEBUG] serialize_ancillary_data_to_stream: Sending fd #{} (value: {}) via sendfd on socket {}", i, fd, **socket_to_use);
                        syscall::sendfd(**socket_to_use as usize, fd as usize, 0, 0)?;
                    }
                }

                // 3.2. Serialize to ancillary_data_stream.
                // Our intermediate format: data_len is size of fd_count (usize), data is fd_count (usize)
                let data_for_stream_len = mem::size_of::<usize>();
                let data_for_stream_payload = (fd_count as usize).to_le_bytes();

                msg_stream.extend_from_slice(&(data_for_stream_len as usize).to_le_bytes()); // data_len field
                msg_stream.extend_from_slice(&data_for_stream_payload); // data field (fd_count)
                eprintln!("[DEBUG] serialize_ancillary_data_to_stream: Wrote SCM_RIGHTS: level={}, type={}, data_len={}, fd_count={}",
                    current_cmsg.cmsg_level, current_cmsg.cmsg_type, data_for_stream_len, fd_count);
            }
            (SOL_SOCKET, SCM_CREDENTIALS) => {
                // Our intermediate format: data_len is 0, no data payload
                let data_for_stream_len = 0usize;
                msg_stream.extend_from_slice(&(data_for_stream_len as usize).to_le_bytes()); // data_len field (0)
                eprintln!("[DEBUG] serialize_ancillary_data_to_stream: Wrote SCM_CREDENTIALS: level={}, type={}, data_len=0",
                    current_cmsg.cmsg_level, current_cmsg.cmsg_type);
            }
            _ => {
                eprintln!(
                    "[ERROR] sendmsg: Unsupported cmsg level {} or type {}",
                    current_cmsg.cmsg_level, current_cmsg.cmsg_type
                );
                return Err(Errno(EOPNOTSUPP));
            }
        }
        cmsg = CMSG_NXTHDR(msg, cmsg);
    }
    Ok(())
}

unsafe fn deserialize_stream_to_name(
    mhdr: &mut msghdr,
    msg_stream: &[u8],
    cursor: &mut usize,
) -> Result<()> {
    eprintln!(
        "[DEBUG] deserialize_stream_to_name: cursor_start = {}",
        *cursor
    );
    // Read name_len from stream
    if *cursor + mem::size_of::<usize>() > msg_stream.len() {
        eprintln!("[ERROR] deserialize_stream_to_name: Not enough data for name_len.");
        return Err(Errno(EMSGSIZE));
    }
    let name_len = usize::from_le_bytes(
        msg_stream[*cursor..*cursor + mem::size_of::<usize>()]
            .try_into()
            .map_err(|_| {
                eprintln!("[ERROR] deserialize_stream_to_name: Failed to read name_len.");
                Errno(EINVAL)
            })?,
    );
    *cursor += mem::size_of::<usize>();
    eprintln!(
        "[DEBUG] deserialize_stream_to_name: name_len = {}",
        name_len
    );

    if name_len > 0 {
        if *cursor + name_len > msg_stream.len() {
            eprintln!("[ERROR] deserialize_stream_to_name: Not enough data for name_buffer (expected {}).", name_len);
            return Err(Errno(EMSGSIZE));
        }
        if !mhdr.msg_name.is_null() && mhdr.msg_namelen > 0 {
            let name_buffer = &msg_stream[*cursor..*cursor + name_len];
            eprintln!("[DEBUG] deserialize_stream_to_name: User provided msg_name buffer (cap: {}), trying to fill with '{}'", mhdr.msg_namelen, str::from_utf8(name_buffer).unwrap_or("invalid_utf8"));
            inner_get_name_inner(
                false,
                mhdr.msg_name as *mut sockaddr,
                &mut mhdr.msg_namelen,
                name_buffer,
            );
        }
        *cursor += name_len;
    }
    eprintln!(
        "[DEBUG] deserialize_stream_to_name: cursor_end = {}",
        *cursor
    );
    Ok(())
}

unsafe fn deserialize_stream_to_payload(
    mhdr: &mut msghdr,
    msg_stream: &[u8],
    iovs_slice: &[iovec],
    cursor: &mut usize,
) -> Result<usize> {
    eprintln!(
        "[DEBUG] deserialize_stream_to_payload: cursor_start = {}",
        *cursor
    );
    // Read payload_len from stream
    if *cursor + mem::size_of::<usize>() > msg_stream.len() {
        eprintln!("[ERROR] deserialize_stream_to_payload: Not enough data for payload_len.");
        return Err(Errno(EMSGSIZE));
    }
    let payload_len = usize::from_le_bytes(
        msg_stream[*cursor..*cursor + mem::size_of::<usize>()]
            .try_into()
            .map_err(|_| {
                eprintln!("[ERROR] deserialize_stream_to_payload: Failed to read payload_len.");
                Errno(EINVAL)
            })?,
    );
    *cursor += mem::size_of::<usize>();
    eprintln!(
        "[DEBUG] deserialize_stream_to_payload: payload_len = {}",
        payload_len
    );

    // Determine actual payload data available in the stream
    let actual_payload_in_stream_len =
        cmp::min(payload_len, msg_stream.len().saturating_sub(*cursor));
    let payload_data_from_stream = &msg_stream[*cursor..*cursor + actual_payload_in_stream_len];

    // Advance cursor by the length *declared* in the stream, even if truncated
    *cursor += actual_payload_in_stream_len;
    // Ensure cursor does not go beyond msg_stream.len() after this conceptual advance
    *cursor = cmp::min(*cursor, msg_stream.len());

    let mut bytes_scattered: usize = 0;
    if !mhdr.msg_iov.is_null() && mhdr.msg_iovlen > 0 && actual_payload_in_stream_len > 0 {
        // Pass iovs_slice as &[iovec] if scatter_data_into_iovs expects that
        bytes_scattered = scatter_data_into_iovs(iovs_slice, payload_data_from_stream)?;
        eprintln!(
            "[DEBUG] deserialize_stream_to_payload: scattered {} bytes into iovecs.",
            bytes_scattered
        );
    }

    if actual_payload_in_stream_len > bytes_scattered {
        eprintln!("[DEBUG] deserialize_stream_to_payload: MSG_TRUNC set. actual_payload_in_stream_len={}, bytes_scattered={}", actual_payload_in_stream_len, bytes_scattered);
        mhdr.msg_flags |= MSG_TRUNC;
    }
    eprintln!(
        "[DEBUG] deserialize_stream_to_payload: cursor_end = {}",
        *cursor
    );
    Ok(bytes_scattered)
}

unsafe fn deserialize_stream_to_ancillary_data(
    mhdr: &mut msghdr,
    socket_to_use: &FdGuard,
    msg_stream: &[u8],
    cursor: &mut usize,
    cmsg_space_provided: usize,
) -> Result<()> {
    eprintln!(
        "[DEBUG] deserialize_stream_to_ancillary_data: cursor_start={}, cmsg_space_provided={}",
        *cursor, cmsg_space_provided
    );
    let mut current_cmsg_ptr_in_user_buf = if !mhdr.msg_control.is_null() && cmsg_space_provided > 0
    {
        CMSG_FIRSTHDR(mhdr)
    } else {
        ptr::null_mut()
    };
    let mut remaining_user_cmsg_buf_len = cmsg_space_provided;
    let mut total_csmg_bytes_written_to_user_buf: usize = 0;
    let mut cmsg_truncated_flag_set = false;

    while *cursor < msg_stream.len() {
        eprintln!(
            "[DEBUG] deserialize_stream_to_ancillary_data: Loop start, cursor = {}",
            *cursor
        );
        const CMSG_HEADER_LEN_IN_STREAM: usize =
            mem::size_of::<c_int>() * 2 + mem::size_of::<usize>();
        if *cursor + CMSG_HEADER_LEN_IN_STREAM > msg_stream.len() {
            eprintln!("[DEBUG] deserialize_stream_to_ancillary_data: Not enough data for cmsg header, breaking.");
            break;
        }

        // cmsg entry format: [level(i32)][type(i32)][data_len(usize)][data]
        let cmsg_level = i32::from_le_bytes(
            msg_stream[*cursor..*cursor + mem::size_of::<c_int>()]
                .try_into()
                .map_err(|_| Errno(EINVAL))?,
        ) as c_int;
        *cursor += mem::size_of::<c_int>();

        let cmsg_type = i32::from_le_bytes(
            msg_stream[*cursor..*cursor + mem::size_of::<c_int>()]
                .try_into()
                .map_err(|_| Errno(EINVAL))?,
        ) as c_int;
        *cursor += mem::size_of::<c_int>();

        let cmsg_data_len_in_stream = usize::from_le_bytes(
            msg_stream[*cursor..*cursor + mem::size_of::<usize>()]
                .try_into()
                .map_err(|_| Errno(EINVAL))?,
        );
        *cursor += mem::size_of::<usize>();
        eprintln!("[DEBUG] deserialize_stream_to_ancillary_data: Parsed CMSG from stream: level={}, type={}, data_len_in_stream={}",
            cmsg_level, cmsg_type, cmsg_data_len_in_stream);

        if *cursor + cmsg_data_len_in_stream > msg_stream.len() {
            eprintln!("[ERROR] deserialize_stream_to_ancillary_data: Stream ended prematurely for cmsg data (expected {} bytes).", cmsg_data_len_in_stream);
            mhdr.msg_flags |= MSG_CTRUNC;
            cmsg_truncated_flag_set = true;
            break;
        }
        let cmsg_data_from_stream = &msg_stream[*cursor..*cursor + cmsg_data_len_in_stream];
        *cursor += cmsg_data_len_in_stream;

        let mut actual_posix_cmsg_data_len: usize = 0;
        let mut temp_posix_cmsg_data_buf: Vec<u8> = Vec::new();

        match (cmsg_level, cmsg_type) {
            (SOL_SOCKET, SCM_RIGHTS) => {
                if cmsg_data_len_in_stream != mem::size_of::<usize>() {
                    eprintln!("[ERROR] deserialize_stream_to_ancillary_data: SCM_RIGHTS data_len mismatch.");
                    return Err(Errno(EINVAL));
                }
                let fd_count = usize::from_le_bytes(
                    cmsg_data_from_stream
                        .try_into()
                        .map_err(|_| Errno(EINVAL))?,
                );
                eprintln!("[DEBUG] deserialize_stream_to_ancillary_data: SCM_RIGHTS, fd_count from stream = {}", fd_count);

                for _ in 0..fd_count {
                    let new_fd = syscall::dup(**socket_to_use as usize, b"recvfd")?;
                    temp_posix_cmsg_data_buf.extend_from_slice(&(new_fd as c_int).to_le_bytes());
                }
                actual_posix_cmsg_data_len = temp_posix_cmsg_data_buf.len();
            }
            (SOL_SOCKET, SCM_CREDENTIALS) => {
                if cmsg_data_len_in_stream
                    != mem::size_of::<pid_t>() + mem::size_of::<uid_t>() + mem::size_of::<gid_t>()
                {
                    eprintln!("[ERROR] deserialize_stream_to_ancillary_data: SCM_CREDENTIALS data_len mismatch.");
                    return Err(Errno(EINVAL));
                }
                let pid = pid_t::from_le_bytes(
                    cmsg_data_from_stream[0..mem::size_of::<pid_t>()]
                        .try_into()
                        .map_err(|_| Errno(EINVAL))?,
                );
                let uid_offset = mem::size_of::<pid_t>();
                let uid = uid_t::from_le_bytes(
                    cmsg_data_from_stream[uid_offset..uid_offset + mem::size_of::<uid_t>()]
                        .try_into()
                        .map_err(|_| Errno(EINVAL))?,
                );
                let gid_offset = uid_offset + mem::size_of::<uid_t>();
                let gid = gid_t::from_le_bytes(
                    cmsg_data_from_stream[gid_offset..gid_offset + mem::size_of::<gid_t>()]
                        .try_into()
                        .map_err(|_| Errno(EINVAL))?,
                );
                let cred = ucred { pid, uid, gid };
                eprintln!("[DEBUG] deserialize_stream_to_ancillary_data: SCM_CREDENTIALS, pid={}, uid={}, gid={}", pid, uid, gid);

                temp_posix_cmsg_data_buf.extend_from_slice(unsafe {
                    slice::from_raw_parts(
                        &cred as *const ucred as *const u8,
                        mem::size_of::<ucred>(),
                    )
                });
                actual_posix_cmsg_data_len = temp_posix_cmsg_data_buf.len();
            }
            _ => {
                eprintln!("[ERROR] deserialize_stream_to_ancillary_data: Unsupported cmsg: level={}, type={}", cmsg_level, cmsg_type);
                return Err(Errno(EINVAL));
            }
        }

        let space_needed_for_posix_cmsg = CMSG_SPACE(actual_posix_cmsg_data_len as u32) as usize;
        eprintln!("[DEBUG] deserialize_stream_to_ancillary_data: POSIX cmsg will need {} bytes. User buffer has {} remaining.", space_needed_for_posix_cmsg, remaining_user_cmsg_buf_len);

        if !current_cmsg_ptr_in_user_buf.is_null()
            && remaining_user_cmsg_buf_len >= space_needed_for_posix_cmsg
        {
            let cmsg_ref = &mut *current_cmsg_ptr_in_user_buf;
            cmsg_ref.cmsg_len = CMSG_LEN(actual_posix_cmsg_data_len as u32) as usize;
            cmsg_ref.cmsg_level = cmsg_level;
            cmsg_ref.cmsg_type = cmsg_type;

            let data_ptr_in_user_cmsg = CMSG_DATA(cmsg_ref);
            ptr::copy_nonoverlapping(
                temp_posix_cmsg_data_buf.as_ptr(),
                data_ptr_in_user_cmsg as *mut u8,
                actual_posix_cmsg_data_len,
            );

            let aligned_len_written = CMSG_ALIGN(cmsg_ref.cmsg_len);
            total_csmg_bytes_written_to_user_buf += aligned_len_written;
            remaining_user_cmsg_buf_len -= aligned_len_written;
            current_cmsg_ptr_in_user_buf = CMSG_NXTHDR(mhdr, current_cmsg_ptr_in_user_buf);
            eprintln!("[DEBUG] deserialize_stream_to_ancillary_data: Wrote POSIX cmsg. total_written={}, remaining_space={}", total_csmg_bytes_written_to_user_buf, remaining_user_cmsg_buf_len);
        } else {
            eprintln!("[DEBUG] deserialize_stream_to_ancillary_data: Not enough space in user cmsg_control, or cmsg_control is null. Setting MSG_CTRUNC.");
            mhdr.msg_flags |= MSG_CTRUNC;
            cmsg_truncated_flag_set = true; // Mark that truncation occurred
            break; // Stop processing further cmsgs
        }
    }
    mhdr.msg_controllen = total_csmg_bytes_written_to_user_buf;
    eprintln!(
        "[DEBUG] deserialize_stream_to_ancillary_data: Final mhdr.msg_controllen = {}",
        mhdr.msg_controllen
    );
    if cmsg_truncated_flag_set {
        eprintln!("[DEBUG] deserialize_stream_to_ancillary_data: MSG_CTRUNC was set.");
    }
    Ok(())
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
        // 1. Prepare iovec
        let mut iov = iovec {
            iov_base: buf,
            iov_len: len,
        };

        // 2. Prepare msghdr
        let mut msg: msghdr = mem::zeroed();

        // 3. Setting destination address
        if !address.is_null() && !address_len.is_null() {
            msg.msg_name = address as *mut c_void;
            msg.msg_namelen = *address_len;
        }
        // 4. Setting data for receiving
        msg.msg_iov = &mut iov;
        msg.msg_iovlen = 1;

        let read = Self::recvmsg(socket, &mut msg, flags)?;

        if !address.is_null() && !address_len.is_null() {
            // Update the address length
            *address_len = msg.msg_namelen;
        }

        Ok(read)
    }

    unsafe fn recvmsg(socket: c_int, msg: *mut msghdr, flags: c_int) -> Result<usize> {
        eprintln!("[DEBUG] recvmsg: socket={}, flags={}", socket, flags);
        if msg.is_null() {
            return Err(Errno(EINVAL));
        }
        let mut mhdr = &mut *msg;
        // 1. accept the socket
        let socket_to_use: FdGuard;
        if !mhdr.msg_name.is_null() || mhdr.msg_namelen != 0 {
            socket_to_use = FdGuard::new(syscall::dup(socket as usize, b"listen")?);
        } else {
            socket_to_use = FdGuard::new(socket.try_into().map_err(|_| Errno(EINVAL))?);
        }

        let mut msg_stream: Vec<u8> = Vec::new();
        let (iovs_slice, whole_iov_size): (&[iovec], usize) =
            if mhdr.msg_iov.is_null() || mhdr.msg_iovlen == 0 {
                (&[], 0)
            } else {
                let iovs_slice =
                    unsafe { slice::from_raw_parts(mhdr.msg_iov, mhdr.msg_iovlen as usize) };
                let whole_iov_size = iovs_slice.iter().map(|iov| iov.iov_len).sum();
                (iovs_slice, whole_iov_size)
            };
        // 2. Reserve space for the message stream.
        // [name_len(usize)][name_buffer]
        // [payload_len(usize)][payload_data_buffer]
        // [ancillary_stream_buffer]
        msg_stream
            .try_reserve_exact(
                mem::size_of::<usize>()        // name_len
                + mhdr.msg_namelen as usize    // name_buffer
                + mem::size_of::<usize>()      // payload_len
                + whole_iov_size               // payload_data_buffer
                + mhdr.msg_controllen as usize, // ancillary_stream_buffer
            )
            .map_err(|_| Errno(ENOMEM))?;

        // 3. Read the message stream.
        let mut command_bytes = [0u8; 8];
        let command = b"recvmsg";
        command_bytes[..command.len()].copy_from_slice(command);
        let metadata = [u64::from_le_bytes(command_bytes.try_into().unwrap())];
        let call_flags = CallFlags::empty();
        eprintln!(
            "[DEBUG] recvmsg: Calling sys_call for socket {}",
            *socket_to_use
        );
        let actual_read_len = redox_rt::sys::sys_call(
            *socket_to_use as usize,
            &mut msg_stream,
            call_flags,
            &metadata,
        )?;
        msg_stream.truncate(actual_read_len);
        eprintln!(
            "[DEBUG] recvmsg: sys_call read {} bytes into msg_stream",
            actual_read_len
        );

        let mut cursor: usize = 0;
        let cmsg_space_provided_by_user = mhdr.msg_controllen;
        mhdr.msg_flags = 0;

        // 4. Get remote name.
        deserialize_stream_to_name(&mut mhdr, &msg_stream, &mut cursor)?;
        eprintln!(
            "[DEBUG] recvmsg: After deserialize_stream_to_name, cursor = {}, mhdr.msg_namelen = {}",
            cursor, mhdr.msg_namelen
        );

        // 5. Get payload data.
        let actual_payload_bytes_written_to_iov =
            deserialize_stream_to_payload(&mut mhdr, &msg_stream, iovs_slice, &mut cursor)?;
        eprintln!("[DEBUG] recvmsg: After deserialize_stream_to_payload, cursor = {}, payload_bytes_written_to_iov = {}", cursor, actual_payload_bytes_written_to_iov);

        // 6. Reconstruct the ancillary data in the user-provided buffer.
        // cmsg entry format: [level(i32)][type(i32)][data_len(usize)][data]
        if !mhdr.msg_control.is_null() && cmsg_space_provided_by_user > 0 {
            if cursor < msg_stream.len() {
                deserialize_stream_to_ancillary_data(
                    mhdr,
                    &socket_to_use,
                    &msg_stream,
                    &mut cursor,
                    cmsg_space_provided_by_user as usize,
                )?;
                eprintln!("[DEBUG] recvmsg: After deserialize_stream_to_ancillary_data, cursor = {}, mhdr.msg_controllen = {}", cursor, mhdr.msg_controllen);
            } else {
                eprintln!("[DEBUG] recvmsg: No ancillary data found in stream after payload (cursor={}, stream_len={})", cursor, msg_stream.len());
                mhdr.msg_controllen = 0;
            }
        } else {
            eprintln!(
                "[DEBUG] recvmsg: User did not provide msg_control buffer or controllen is 0."
            );
            mhdr.msg_controllen = 0;
        }
        eprintln!(
            "[DEBUG] recvmsg: Returning Ok({})",
            actual_payload_bytes_written_to_iov
        );
        Ok(actual_payload_bytes_written_to_iov)
    }

    unsafe fn sendmsg(socket: c_int, msg: *const msghdr, flags: c_int) -> Result<usize> {
        eprintln!("[DEBUG] sendmsg: socket={}, flags={}", socket, flags);
        if msg.is_null() {
            return Err(Errno(EINVAL));
        }
        let mhdr = &*msg;

        // 1. Determine if the socket is connected or needs to be bound.
        let socket_to_use: FdGuard;
        if !mhdr.msg_name.is_null() || mhdr.msg_namelen != 0 {
            eprintln!(
                "[DEBUG] sendmsg: msg_name is set, attempting connect copy for socket {}",
                socket
            );
            match bind_or_connect!(connect copy, socket, mhdr.msg_name as *const sockaddr, mhdr.msg_namelen)
            {
                Ok(new_fd) => {
                    eprintln!(
                        "[DEBUG] sendmsg: connect copy successful, using new_fd = {}",
                        new_fd
                    );
                    socket_to_use = FdGuard::new(new_fd);
                }
                Err(err) if err.errno == EISCONN => {
                    eprintln!(
                        "[DEBUG] sendmsg: connect copy returned EISCONN, using original socket {}",
                        socket
                    );
                    socket_to_use = FdGuard::new(socket.try_into().map_err(|_| Errno(EINVAL))?);
                }
                Err(err) => {
                    eprintln!("[ERROR] sendmsg: connect copy failed with error: {:?}", err);
                    return Err(err.into());
                }
            };
        } else {
            eprintln!(
                "[DEBUG] sendmsg: msg_name not set, using original socket {}",
                socket
            );
            socket_to_use = FdGuard::new(socket.try_into().map_err(|_| Errno(EINVAL))?);
        };

        // 2. Reserve space for the message stream.
        // [payload_len(usize)][payload_data_buffer]
        // [ancillary_stream_buffer]
        let mut msg_stream: Vec<u8> = Vec::new();
        let (iovs_slice, whole_iov_size): (&[iovec], usize) =
            if mhdr.msg_iov.is_null() || mhdr.msg_iovlen == 0 {
                eprintln!("[DEBUG] sendmsg: No payload iovecs.");
                (&[], 0)
            } else {
                let iovs_slice =
                    unsafe { slice::from_raw_parts(mhdr.msg_iov, mhdr.msg_iovlen as usize) };
                let whole_iov_size = iovs_slice.iter().map(|iov| iov.iov_len).sum();
                eprintln!(
                    "[DEBUG] sendmsg: Found {} iovecs, total payload size calculated = {}",
                    iovs_slice.len(),
                    whole_iov_size
                );
                (iovs_slice, whole_iov_size)
            };
        msg_stream
            .try_reserve_exact(
                mem::size_of::<usize>()     // payload_len
            + whole_iov_size                // payload_data_buffer
            + mhdr.msg_controllen as usize, // ancillary_stream_buffer
            )
            .map_err(|_| Errno(ENOMEM))?;
        // write a placeholder for payload_len
        msg_stream.extend_from_slice(&[0u8; mem::size_of::<usize>()]);

        // 3. Write the message to the msg_stream.
        let mut actual_payload_bytes_serialized = 0;
        if !mhdr.msg_iov.is_null() && mhdr.msg_iovlen > 0 {
            actual_payload_bytes_serialized =
                serialize_payload_to_stream(&mut msg_stream, &iovs_slice, whole_iov_size)?;
        } else {
            eprintln!(
                "[DEBUG] sendmsg: No payload, wrote 0 length prefix. msg_stream.len() = {}",
                msg_stream.len()
            );
        }

        // 4. Process Control Messages from msghdr and serialize them.
        if mhdr.msg_controllen > 0 {
            serialize_ancillary_data_to_stream(msg, mhdr, &socket_to_use, &mut msg_stream)?;
            eprintln!(
                "[DEBUG] sendmsg: Ancillary data serialized. msg_stream.len() = {}",
                msg_stream.len()
            );
        } else {
            eprintln!("[DEBUG] sendmsg: No ancillary data to serialize.");
        }

        // 5. Prepare command.
        let mut command_bytes = [0u8; 8];
        let command = b"sendmsg";
        command_bytes[..command.len()].copy_from_slice(command);
        let metadata = [u64::from_le_bytes(command_bytes.try_into().unwrap())];
        let call_flags = CallFlags::empty();

        // 6. Send the message stream.
        eprintln!(
            "[DEBUG] sendmsg: Calling sys_call for socket {}, msg_stream.len() = {}",
            *socket_to_use,
            msg_stream.len()
        );
        let written = redox_rt::sys::sys_call(
            *socket_to_use as usize,
            msg_stream.as_mut_slice(),
            call_flags,
            &metadata,
        )?;

        eprintln!(
            "[DEBUG] sendmsg: Returning Ok({})",
            actual_payload_bytes_serialized
        );

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
            return Err(Errno(EOPNOTSUPP));
        }
        // 1. Prepare iovec
        let iov = iovec {
            iov_base: buf as *mut c_void,
            iov_len: len,
        };

        // 2.prepare msghdr
        let mut msg: msghdr = mem::zeroed();

        // 3. setting destination address
        if dest_addr != ptr::null() || dest_len != 0 {
            msg.msg_name = dest_addr as *mut c_void;
            msg.msg_namelen = dest_len;
        }

        // 4. setting data for sending
        msg.msg_iov = &iov as *const iovec as *mut iovec;
        msg.msg_iovlen = 1;

        // 5. sendto does not support control messages
        msg.msg_control = ptr::null_mut();
        msg.msg_controllen = 0;

        Self::sendmsg(socket, &msg, flags)
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
                SO_PASSCRED => {
                    let mut command_bytes = [0u8; 16];
                    let command = b"setsockopt";
                    command_bytes[..command.len()].copy_from_slice(command);
                    let metadata = [
                        u64::from_le_bytes(command_bytes[0..8].try_into().unwrap()),
                        u64::from_le_bytes(command_bytes[8..16].try_into().unwrap()),
                    ];
                    let mut payload = SO_PASSCRED.to_ne_bytes().to_vec();
                    let call_flags = CallFlags::empty();
                    redox_rt::sys::sys_call(
                        socket_fd as usize,
                        payload.as_mut_slice(),
                        CallFlags::empty(),
                        &metadata,
                    )?;
                    return Ok(());
                }
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
