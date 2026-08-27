//! `sys/socket.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_socket.h.html>.

use core::{mem, ptr};

use crate::{
    error::ResultExt,
    header::{bits_safamily_t::sa_family_t, sys_uio::iovec},
    out::Out,
    platform::{
        PalSocket, Sys,
        types::{c_char, c_int, c_long, c_uchar, c_uint, c_void, size_t, ssize_t},
    },
};

pub use crate::header::bits_socklen_t::socklen_t;

pub mod constants;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_socket.h.html>.
#[repr(C)]
#[derive(Default, CheckVsLibcCrate)]
pub struct linger {
    /// Indicates whether linger option is enabled.
    pub l_onoff: c_int,
    /// Linger time, in seconds.
    pub l_linger: c_int,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_socket.h.html>.
#[repr(C)]
#[derive(Debug, CheckVsLibcCrate)]
pub struct msghdr {
    /// Optional address.
    pub msg_name: *mut c_void,
    /// Size of address.
    pub msg_namelen: socklen_t,
    /// Scatter/gather array.
    pub msg_iov: *mut iovec,
    /// Members in `msg_iov`.
    pub msg_iovlen: size_t,
    /// Ancilliary data.
    pub msg_control: *mut c_void,
    /// Ancilliary data buffer length.
    pub msg_controllen: size_t,
    /// Flags on received message.
    pub msg_flags: c_int,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_socket.h.html>.
#[repr(C)]
#[derive(Debug, CheckVsLibcCrate)]
pub struct cmsghdr {
    /// Data byte count, including the `cmsghdr`.
    pub cmsg_len: size_t,
    /// Originating protocol.
    pub cmsg_level: c_int,
    /// Protocol-specific type.
    pub cmsg_type: c_int,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_socket.h.html>.
#[repr(C)]
#[derive(Default, CheckVsLibcCrate)]
pub struct sockaddr {
    /// Address family.
    pub sa_family: sa_family_t,
    /// Socket address.
    pub sa_data: [c_char; 14],
}

// Max size of [`sockaddr_storage`]
/// cbindgen:ignore
const _SS_MAXSIZE: usize = 128;
// Align to pointer width
/// cbindgen:ignore
const _SS_PADDING: usize = _SS_MAXSIZE - mem::size_of::<sa_family_t>() - mem::size_of::<usize>();

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_socket.h.html>.
/// Opaque storage large enough to hold any protocol specific address structure.
///
/// ## Implementation notes
/// * The total size of this struct is 128 bytes which is based off of `musl` and `glibc`
/// * The underscore fields are implementation specific details for padding that may change
/// * [`usize`] is used because it's the width of a pointer for a given platform
/// * The order of the fields is important because the bytes in the padding will be cast to and
///   from protocol structs in C
///
/// cbindgen:ignore
#[repr(C)]
//#[derive(CheckVsLibcCrate)] FIXME: can't ignore private fields yet
pub struct sockaddr_storage {
    /// Address family.
    pub ss_family: sa_family_t,
    __ss_pad2: [u8; _SS_PADDING],
    __ss_align: usize,
}

// These must match C macros in sys_socket/cbindgen.toml {
/// cbindgen:ignore
pub unsafe extern "C" fn __CMSG_LEN(cmsg: *const cmsghdr) -> ssize_t {
    ((unsafe { (*cmsg).cmsg_len as size_t } + mem::size_of::<c_long>() - 1)
        & !(mem::size_of::<c_long>() - 1)) as ssize_t
}

/// cbindgen:ignore
pub unsafe extern "C" fn __CMSG_NEXT(cmsg: *const cmsghdr) -> *mut c_uchar {
    unsafe { (cmsg as *mut c_uchar).offset(__CMSG_LEN(cmsg)) }
}

/// cbindgen:ignore
pub unsafe extern "C" fn __MHDR_END(mhdr: *const msghdr) -> *mut c_uchar {
    unsafe { ((*mhdr).msg_control.cast::<c_uchar>()).add((*mhdr).msg_controllen) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_socket.h.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn CMSG_DATA(cmsg: *const cmsghdr) -> *mut c_uchar {
    unsafe { (cmsg as *mut c_uchar).add(CMSG_ALIGN(mem::size_of::<cmsghdr>())) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_socket.h.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn CMSG_NXTHDR(mhdr: *const msghdr, cmsg: *const cmsghdr) -> *mut cmsghdr {
    if cmsg.is_null() {
        return unsafe { CMSG_FIRSTHDR(mhdr) };
    };

    unsafe {
        let next =
            cmsg as usize + CMSG_ALIGN((*cmsg).cmsg_len) + CMSG_ALIGN(mem::size_of::<cmsghdr>());
        let max = (*mhdr).msg_control as usize + (*mhdr).msg_controllen;
        if next > max {
            ptr::null_mut::<cmsghdr>()
        } else {
            (cmsg as usize + CMSG_ALIGN((*cmsg).cmsg_len)) as *mut cmsghdr
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_socket.h.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn CMSG_FIRSTHDR(mhdr: *const msghdr) -> *mut cmsghdr {
    unsafe {
        if (*mhdr).msg_controllen >= mem::size_of::<cmsghdr>() {
            (*mhdr).msg_control.cast::<cmsghdr>()
        } else {
            ptr::null_mut::<cmsghdr>()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn CMSG_ALIGN(len: size_t) -> size_t {
    (len + mem::size_of::<size_t>() - 1) & !(mem::size_of::<size_t>() - 1)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_socket.h.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn CMSG_SPACE(len: c_uint) -> c_uint {
    (unsafe { CMSG_ALIGN(len as size_t) } + unsafe { CMSG_ALIGN(mem::size_of::<cmsghdr>()) })
        as c_uint
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_socket.h.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn CMSG_LEN(length: c_uint) -> c_uint {
    (unsafe { CMSG_ALIGN(mem::size_of::<cmsghdr>()) } + length as usize) as c_uint
}
// } These must match C macros in sys_socket/cbindgen.toml

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/accept.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept(
    socket: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> c_int {
    let dst = if address.is_null() || address_len.is_null() {
        None
    } else {
        Some(unsafe {
            let len: usize = address_len.read().try_into().unwrap();
            core::slice::from_raw_parts_mut(address.cast::<u8>(), len)
        })
    };
    trace_expr!(
        match Sys::accept(socket, dst) {
            Ok((result, true_len)) => {
                if let Some(len_out) = unsafe { address_len.as_mut() } {
                    *len_out = true_len;
                }

                result
            }
            Err(err) => Err(err).or_minus_one_errno(),
        },
        "accept({}, {:p}, {:p})",
        socket,
        address,
        address_len
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/bind.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bind(
    socket: c_int,
    address: *const sockaddr,
    address_len: socklen_t,
) -> c_int {
    let address_raw =
        unsafe { core::slice::from_raw_parts(address.cast::<u8>(), address_len as usize) };
    trace_expr!(
        Sys::bind(socket, address_raw)
            .map(|()| 0)
            .or_minus_one_errno(),
        "bind({}, {:p}, {})",
        socket,
        address,
        address_len
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/connect.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn connect(
    socket: c_int,
    address: *const sockaddr,
    address_len: socklen_t,
) -> c_int {
    let address_raw =
        unsafe { core::slice::from_raw_parts(address.cast::<u8>(), address_len as usize) };
    trace_expr!(
        Sys::connect(socket, address_raw).or_minus_one_errno(),
        "connect({}, {:p}, {})",
        socket,
        address,
        address_len
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getpeername.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getpeername(
    socket: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> c_int {
    let dst = unsafe {
        let len: usize = address_len.read().try_into().unwrap();
        core::slice::from_raw_parts_mut(address.cast::<u8>(), len)
    };

    trace_expr!(
        match Sys::getpeername(socket, dst) {
            Ok(true_len) => {
                unsafe { address_len.write(true_len) };
                0
            }
            Err(err) => Err(err).or_minus_one_errno(),
        },
        "getpeername({}, {:p}, {:p})",
        socket,
        address,
        address_len
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getsockname.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getsockname(
    socket: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> c_int {
    let dst = unsafe {
        let len: usize = address_len.read().try_into().unwrap();
        core::slice::from_raw_parts_mut(address.cast::<u8>(), len)
    };

    trace_expr!(
        match Sys::getsockname(socket, dst) {
            Ok(true_len) => {
                unsafe { address_len.write(true_len) };
                0
            }
            Err(err) => Err(err).or_minus_one_errno(),
        },
        "getsockname({}, {:p}, {:p})",
        socket,
        address,
        address_len
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getsockopt.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getsockopt(
    socket: c_int,
    level: c_int,
    option_name: c_int,
    option_value: *mut c_void,
    option_len: *mut socklen_t,
) -> c_int {
    let option_value = unsafe {
        core::slice::from_raw_parts_mut(option_value.cast::<u8>(), option_len.read() as usize)
    };

    trace_expr!(
        match Sys::getsockopt(socket, level, option_name, option_value) {
            Ok(true_len) => {
                unsafe {
                    option_len.write(true_len);
                }
                0
            }
            Err(error) => Err(error).or_minus_one_errno(),
        },
        "getsockopt({}, {}, {}, {:p}, {:p})",
        socket,
        level,
        option_name,
        option_value,
        option_len
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/listen.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn listen(socket: c_int, backlog: c_int) -> c_int {
    Sys::listen(socket, backlog)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/recv.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn recv(
    socket: c_int,
    buffer: *mut c_void,
    length: size_t,
    flags: c_int,
) -> ssize_t {
    unsafe {
        recvfrom(
            socket,
            buffer,
            length,
            flags,
            ptr::null_mut(),
            ptr::null_mut(),
        )
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/recvfrom.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn recvfrom(
    socket: c_int,
    buffer: *mut c_void,
    length: size_t,
    flags: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> ssize_t {
    let buffer_out = unsafe { Out::from_raw_parts(buffer.cast::<u8>(), length) };
    let address_slice = if address.is_null() || address_len.is_null() {
        None
    } else {
        Some(unsafe {
            core::slice::from_raw_parts_mut(address.cast::<u8>(), address_len.read() as usize)
        })
    };

    trace_expr!(
        match Sys::recvfrom(socket, buffer_out, flags, address_slice) {
            Ok((bytes_read, addr_len)) => {
                if let Some(len_out) = unsafe { address_len.as_mut() } {
                    *len_out = addr_len;
                }
                bytes_read as ssize_t
            }
            Err(err) => Err(err).or_minus_one_errno(),
        },
        "recvfrom({}, {:p}, {}, {:#x}, {:p}, {:p})",
        socket,
        buffer,
        length,
        flags,
        address,
        address_len
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/recvmsg.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn recvmsg(socket: c_int, msg: *mut msghdr, flags: c_int) -> ssize_t {
    unsafe { Sys::recvmsg(socket, msg, flags) }
        .map(|r| r as ssize_t)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/send.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn send(
    socket: c_int,
    message: *const c_void,
    length: size_t,
    flags: c_int,
) -> ssize_t {
    unsafe { sendto(socket, message, length, flags, ptr::null(), 0) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sendmsg.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sendmsg(socket: c_int, msg: *const msghdr, flags: c_int) -> ssize_t {
    unsafe { Sys::sendmsg(socket, msg, flags) }
        .map(|w| w as ssize_t)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sendto.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sendto(
    socket: c_int,
    message: *const c_void,
    length: size_t,
    flags: c_int,
    dest_addr: *const sockaddr,
    dest_len: socklen_t,
) -> ssize_t {
    let message = unsafe { core::slice::from_raw_parts(message.cast::<u8>(), length) };

    let dest_addr_slice = if dest_addr.is_null() {
        None
    } else {
        Some(unsafe { core::slice::from_raw_parts(dest_addr.cast::<u8>(), dest_len as usize) })
    };

    trace_expr!(
        Sys::sendto(socket, message, flags, dest_addr_slice)
            .map(|w| w as ssize_t)
            .or_minus_one_errno(),
        "sendto({}, {:p}, {}, {:#x}, {:p}, {})",
        socket,
        message,
        length,
        flags,
        dest_addr,
        dest_len
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setsockopt.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setsockopt(
    socket: c_int,
    level: c_int,
    option_name: c_int,
    option_value: *const c_void,
    option_len: socklen_t,
) -> c_int {
    let option_value =
        unsafe { core::slice::from_raw_parts(option_value.cast::<u8>(), option_len as usize) };

    trace_expr!(
        Sys::setsockopt(socket, level, option_name, option_value)
            .map(|()| 0)
            .or_minus_one_errno(),
        "setsockopt({}, {}, {}, {:p}, {})",
        socket,
        level,
        option_name,
        option_value,
        option_len
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/shutdown.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shutdown(socket: c_int, how: c_int) -> c_int {
    Sys::shutdown(socket, how).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/socket.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn socket(domain: c_int, kind: c_int, protocol: c_int) -> c_int {
    trace_expr!(
        Sys::socket(domain, kind, protocol).or_minus_one_errno(),
        "socket({}, {}, {})",
        domain,
        kind,
        protocol,
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/socketpair.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn socketpair(
    domain: c_int,
    kind: c_int,
    protocol: c_int,
    sv: *mut c_int,
) -> c_int {
    trace_expr!(
        Sys::socketpair(domain, kind, protocol, unsafe {
            &mut *sv.cast::<[c_int; 2]>()
        })
        .map(|()| 0)
        .or_minus_one_errno(),
        "socketpair({}, {}, {}, {:p})",
        domain,
        kind,
        protocol,
        sv
    )
}
