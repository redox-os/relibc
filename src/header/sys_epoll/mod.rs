//! `sys/epoll.h` implementation.
//!
//! Non-POSIX, see <http://man7.org/linux/man-pages/man7/epoll.7.html>.

use core::ptr;

use crate::{
    error::ResultExt,
    header::bits_sigset_t::sigset_t,
    platform::{
        PalEpoll, Sys,
        types::{c_int, c_uint, c_ulonglong, c_void},
    },
};

/// Set the close-on-exec (`FD_CLOEXEC`) flag on the new file descriptor.
#[cfg(target_os = "linux")]
pub const EPOLL_CLOEXEC: c_int = 0x8_0000;

/// Set the close-on-exec (`FD_CLOEXEC`) flag on the new file descriptor.
#[cfg(target_os = "redox")]
pub const EPOLL_CLOEXEC: c_int = 0x0100_0000;

/// The associated file is available for read operations.
pub const EPOLLIN: c_uint = 0x001;
/// There is an exceptional condition on the file descriptor.
pub const EPOLLPRI: c_uint = 0x002;
/// The associated file is available for write operations.
pub const EPOLLOUT: c_uint = 0x004;
/// Error condition happened on the associated file descriptor.
pub const EPOLLERR: c_uint = 0x008;
/// Hang up happened onthe associated file descriptor.
pub const EPOLLHUP: c_uint = 0x010;
pub const EPOLLNVAL: c_uint = 0x020;
pub const EPOLLRDNORM: c_uint = 0x040;
pub const EPOLLRDBAND: c_uint = 0x080;
pub const EPOLLWRNORM: c_uint = 0x100;
pub const EPOLLWRBAND: c_uint = 0x200;
pub const EPOLLMSG: c_uint = 0x400;
/// Stream socket peer closed connection, or shut down writing half of
/// connection.
pub const EPOLLRDHUP: c_uint = 0x2000;
/// Sets an exclusive wakeup mode for the epoll file descriptor that is being
/// attached to the target file descriptor, `fd`.
pub const EPOLLEXCLUSIVE: c_uint = 1 << 28;
/// If `EPOLLONESHOT` and `EPOLLET` are clear and the process has the
/// `CAP_BLOCK_SUSPEND` capability, ensure that the system does not enter
/// "suspend" or "hibernate" while this event is pending or being processed.
pub const EPOLLWAKEUP: c_uint = 1 << 29;
/// Requests one-shot notification for the associated file descriptor.
pub const EPOLLONESHOT: c_uint = 1 << 30;
/// Requests edge-triggered notification for the associated file descriptor.
pub const EPOLLET: c_uint = 1 << 31;

/// Add an entry to the interest list of the epoll file descriptor, `epfd`.
pub const EPOLL_CTL_ADD: c_int = 1;
/// Remove (deregister) the target file descriptor `fd` from the interest list.
pub const EPOLL_CTL_DEL: c_int = 2;
/// Change the settings associated with `fd` in the interest list to the new
/// settings specified in `event`.
pub const EPOLL_CTL_MOD: c_int = 3;

/// Non-POSIX, see <https://man7.org/linux/man-pages/man3/epoll_event.3type.html>.
#[repr(C)]
#[derive(Clone, Copy)]
pub union epoll_data {
    pub ptr: *mut c_void,
    pub fd: c_int,
    pub u32: c_uint,
    pub u64: c_ulonglong,
}
impl Default for epoll_data {
    fn default() -> Self {
        Self { u64: 0 }
    }
}

/// Non-POSIX, see <https://man7.org/linux/man-pages/man3/epoll_event.3type.html>.
#[cfg(all(target_os = "redox", target_pointer_width = "64"))]
#[repr(C)]
#[derive(Clone, Copy, Default)]
// This will match in size with syscall::Event (24 bytes on 64-bit
// systems) on redox. The `Default` trait is here so we don't need to
// worry about the padding when using this type.
pub struct epoll_event {
    pub events: c_uint, // 4 bytes
    // 4 automatic alignment bytes
    pub data: epoll_data, // 8 bytes

    pub _pad: c_ulonglong, // 8 bytes
}

/// Non-POSIX, see <https://man7.org/linux/man-pages/man3/epoll_event.3type.html>.
#[cfg(not(all(target_os = "redox", target_pointer_width = "64")))]
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct epoll_event {
    pub events: c_uint,
    pub data: epoll_data,
}

/// Non-POSIX, see <https://man7.org/linux/man-pages/man2/epoll_create.2.html>.
#[unsafe(no_mangle)]
pub extern "C" fn epoll_create(_size: c_int) -> c_int {
    epoll_create1(0)
}

/// Non-POSIX, see <https://man7.org/linux/man-pages/man2/epoll_create1.2.html>.
#[unsafe(no_mangle)]
pub extern "C" fn epoll_create1(flags: c_int) -> c_int {
    trace_expr!(
        Sys::epoll_create1(flags).or_minus_one_errno(),
        "epoll_create1({:#x})",
        flags
    )
}

/// Non-POSIX, see <https://man7.org/linux/man-pages/man2/epoll_ctl.2.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn epoll_ctl(
    epfd: c_int,
    op: c_int,
    fd: c_int,
    event: *mut epoll_event,
) -> c_int {
    trace_expr!(
        unsafe { Sys::epoll_ctl(epfd, op, fd, event) }
            .map(|()| 0)
            .or_minus_one_errno(),
        "epoll_ctl({}, {}, {}, {:p})",
        epfd,
        op,
        fd,
        event
    )
}

/// Non-POSIX, see <https://man7.org/linux/man-pages/man2/epoll_wait.2.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn epoll_wait(
    epfd: c_int,
    events: *mut epoll_event,
    maxevents: c_int,
    timeout: c_int,
) -> c_int {
    unsafe { epoll_pwait(epfd, events, maxevents, timeout, ptr::null()) }
}

/// Non-POSIX, see <https://man7.org/linux/man-pages/man2/epoll_wait.2.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn epoll_pwait(
    epfd: c_int,
    events: *mut epoll_event,
    maxevents: c_int,
    timeout: c_int,
    sigmask: *const sigset_t,
) -> c_int {
    trace_expr!(
        unsafe { Sys::epoll_pwait(epfd, events, maxevents, timeout, sigmask) }
            .map(|e| e as c_int)
            .or_minus_one_errno(),
        "epoll_pwait({}, {:p}, {}, {}, {:p})",
        epfd,
        events,
        maxevents,
        timeout,
        sigmask
    )
}
