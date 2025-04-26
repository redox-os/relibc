//! socket implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xns/syssocket.h.html

use core::{mem, ptr};

use crate::{
    error::ResultExt,
    header::sys_uio::iovec,
    platform::{types::*, PalSocket, Sys},
};

pub mod constants;

pub type sa_family_t = u16;
pub type socklen_t = size_t;

#[repr(C)]
#[derive(Default)]
pub struct linger {
    pub l_onoff: c_int,
    pub l_linger: c_int,
}

#[no_mangle]
pub extern "C" fn _cbindgen_export_linger(linger: linger) {}

#[repr(C)]
pub struct msghdr {
    pub msg_name: *mut c_void,
    pub msg_namelen: socklen_t,
    pub msg_iov: *mut iovec,
    pub msg_iovlen: size_t,
    pub msg_control: *mut c_void,
    pub msg_controllen: size_t,
    pub msg_flags: c_int,
}

#[repr(C)]
pub struct cmsghdr {
    pub cmsg_len: size_t,
    pub cmsg_level: c_int,
    pub cmsg_type: c_int,
}

#[repr(C)]
#[derive(Default)]
pub struct sockaddr {
    pub sa_family: sa_family_t,
    pub sa_data: [c_char; 14],
}

// Max size of [`sockaddr_storage`]
const _SS_MAXSIZE: usize = 128;
// Align to pointer width
const _SS_PADDING: usize = _SS_MAXSIZE - mem::size_of::<sa_family_t>() - mem::size_of::<usize>();

/// Opaque storage large enough to hold any protocol specific address structure.
///
/// ## Implementation notes
/// * The total size of this struct is 128 bytes which is based off of `musl` and `glibc`
/// * The underscore fields are implementation specific details for padding that may change
/// * [`usize`] is used because it's the width of a pointer for a given platform
/// * The order of the fields is important because the bytes in the padding will be cast to and
/// from protocol structs in C
#[repr(C)]
pub struct sockaddr_storage {
    pub ss_family: sa_family_t,
    __ss_pad2: [u8; _SS_PADDING],
    __ss_align: usize,
}

#[no_mangle]
pub unsafe extern "C" fn CMSG_ALIGN(len: size_t) -> size_t {
    (len + mem::size_of::<size_t>() - 1) & !(mem::size_of::<size_t>() - 1)
}

#[no_mangle]
pub unsafe extern "C" fn CMSG_LEN(length: c_uint) -> c_uint {
    (CMSG_ALIGN(mem::size_of::<cmsghdr>()) + length as usize) as c_uint
}

#[no_mangle]
pub unsafe extern "C" fn CMSG_SPACE(len: c_uint) -> c_uint {
    (CMSG_ALIGN(len as size_t) + CMSG_ALIGN(mem::size_of::<cmsghdr>())) as c_uint
}

#[no_mangle]
pub unsafe extern "C" fn CMSG_FIRSTHDR(mhdr: *const msghdr) -> *mut cmsghdr {
    unsafe {
        if (*mhdr).msg_controllen as usize >= mem::size_of::<cmsghdr>() {
            (*mhdr).msg_control as *mut cmsghdr
        } else {
            0 as *mut cmsghdr
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn CMSG_NXTHDR(mhdr: *const msghdr, cmsg: *const cmsghdr) -> *mut cmsghdr {
    if cmsg.is_null() {
        return CMSG_FIRSTHDR(mhdr);
    };

    unsafe {
        let next = cmsg as usize
            + CMSG_ALIGN((*cmsg).cmsg_len as usize)
            + CMSG_ALIGN(mem::size_of::<cmsghdr>());
        let max = (*mhdr).msg_control as usize + (*mhdr).msg_controllen as usize;
        if next > max {
            0 as *mut cmsghdr
        } else {
            (cmsg as usize + CMSG_ALIGN((*cmsg).cmsg_len as usize)) as *mut cmsghdr
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn CMSG_DATA(cmsg: *const cmsghdr) -> *mut c_uchar {
    unsafe { (cmsg as *mut c_uchar).offset(CMSG_ALIGN(mem::size_of::<cmsghdr>()) as isize) }
}

#[no_mangle]
pub fn __MHDR_END(mhdr: *const msghdr) -> *mut c_uchar {
    unsafe { (*mhdr).msg_control.offset((*mhdr).msg_controllen as isize) }.cast()
}

#[no_mangle]
pub fn __CMSG_LEN(cmsg: *const cmsghdr) -> ssize_t {
    ((unsafe { (*cmsg).cmsg_len as size_t } + mem::size_of::<c_long>() - 1)
        & !(mem::size_of::<c_long>() - 1)) as ssize_t
}

#[no_mangle]
pub fn __CMSG_NEXT(cmsg: *const cmsghdr) -> *mut c_uchar {
    (unsafe { cmsg.offset(__CMSG_LEN(cmsg)) }) as *mut c_uchar
}

#[no_mangle]
pub unsafe extern "C" fn accept(
    socket: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> c_int {
    trace_expr!(
        Sys::accept(socket, address, address_len).or_minus_one_errno(),
        "accept({}, {:p}, {:p})",
        socket,
        address,
        address_len
    )
}

#[no_mangle]
pub unsafe extern "C" fn bind(
    socket: c_int,
    address: *const sockaddr,
    address_len: socklen_t,
) -> c_int {
    trace_expr!(
        Sys::bind(socket, address, address_len)
            .map(|()| 0)
            .or_minus_one_errno(),
        "bind({}, {:p}, {})",
        socket,
        address,
        address_len
    )
}

#[no_mangle]
pub unsafe extern "C" fn connect(
    socket: c_int,
    address: *const sockaddr,
    address_len: socklen_t,
) -> c_int {
    trace_expr!(
        Sys::connect(socket, address, address_len).or_minus_one_errno(),
        "connect({}, {:p}, {})",
        socket,
        address,
        address_len
    )
}

#[no_mangle]
pub unsafe extern "C" fn getpeername(
    socket: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> c_int {
    trace_expr!(
        Sys::getpeername(socket, address, address_len)
            .map(|()| 0)
            .or_minus_one_errno(),
        "getpeername({}, {:p}, {:p})",
        socket,
        address,
        address_len
    )
}

#[no_mangle]
pub unsafe extern "C" fn getsockname(
    socket: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> c_int {
    trace_expr!(
        Sys::getsockname(socket, address, address_len)
            .map(|()| 0)
            .or_minus_one_errno(),
        "getsockname({}, {:p}, {:p})",
        socket,
        address,
        address_len
    )
}

#[no_mangle]
pub unsafe extern "C" fn getsockopt(
    socket: c_int,
    level: c_int,
    option_name: c_int,
    option_value: *mut c_void,
    option_len: *mut socklen_t,
) -> c_int {
    trace_expr!(
        Sys::getsockopt(socket, level, option_name, option_value, option_len)
            .map(|()| 0)
            .or_minus_one_errno(),
        "getsockopt({}, {}, {}, {:p}, {:p})",
        socket,
        level,
        option_name,
        option_value,
        option_len
    )
}

#[no_mangle]
pub unsafe extern "C" fn listen(socket: c_int, backlog: c_int) -> c_int {
    Sys::listen(socket, backlog)
        .map(|()| 0)
        .or_minus_one_errno()
}

#[no_mangle]
pub unsafe extern "C" fn recv(
    socket: c_int,
    buffer: *mut c_void,
    length: size_t,
    flags: c_int,
) -> ssize_t {
    recvfrom(
        socket,
        buffer,
        length,
        flags,
        ptr::null_mut(),
        ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "C" fn recvfrom(
    socket: c_int,
    buffer: *mut c_void,
    length: size_t,
    flags: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> ssize_t {
    trace_expr!(
        Sys::recvfrom(socket, buffer, length, flags, address, address_len)
            .map(|r| r as ssize_t)
            .or_minus_one_errno(),
        "recvfrom({}, {:p}, {}, {:#x}, {:p}, {:p})",
        socket,
        buffer,
        length,
        flags,
        address,
        address_len
    )
}

#[no_mangle]
pub unsafe extern "C" fn recvmsg(socket: c_int, msg: *mut msghdr, flags: c_int) -> ssize_t {
    Sys::recvmsg(socket, msg, flags)
        .map(|r| r as ssize_t)
        .or_minus_one_errno()
}

#[no_mangle]
pub unsafe extern "C" fn send(
    socket: c_int,
    message: *const c_void,
    length: size_t,
    flags: c_int,
) -> ssize_t {
    sendto(socket, message, length, flags, ptr::null(), 0)
}

#[no_mangle]
pub unsafe extern "C" fn sendmsg(socket: c_int, msg: *const msghdr, flags: c_int) -> ssize_t {
    Sys::sendmsg(socket, msg, flags)
        .map(|w| w as ssize_t)
        .or_minus_one_errno()
}

#[no_mangle]
pub unsafe extern "C" fn sendto(
    socket: c_int,
    message: *const c_void,
    length: size_t,
    flags: c_int,
    dest_addr: *const sockaddr,
    dest_len: socklen_t,
) -> ssize_t {
    trace_expr!(
        Sys::sendto(socket, message, length, flags, dest_addr, dest_len)
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

#[no_mangle]
pub unsafe extern "C" fn setsockopt(
    socket: c_int,
    level: c_int,
    option_name: c_int,
    option_value: *const c_void,
    option_len: socklen_t,
) -> c_int {
    trace_expr!(
        Sys::setsockopt(socket, level, option_name, option_value, option_len)
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

#[no_mangle]
pub unsafe extern "C" fn shutdown(socket: c_int, how: c_int) -> c_int {
    Sys::shutdown(socket, how).map(|()| 0).or_minus_one_errno()
}

#[no_mangle]
pub unsafe extern "C" fn socket(domain: c_int, kind: c_int, protocol: c_int) -> c_int {
    trace_expr!(
        Sys::socket(domain, kind, protocol).or_minus_one_errno(),
        "socket({}, {}, {})",
        domain,
        kind,
        protocol,
    )
}

#[no_mangle]
pub unsafe extern "C" fn socketpair(
    domain: c_int,
    kind: c_int,
    protocol: c_int,
    sv: *mut c_int,
) -> c_int {
    trace_expr!(
        Sys::socketpair(domain, kind, protocol, &mut *(sv as *mut [c_int; 2]))
            .map(|()| 0)
            .or_minus_one_errno(),
        "socketpair({}, {}, {}, {:p})",
        domain,
        kind,
        protocol,
        sv
    )
}
