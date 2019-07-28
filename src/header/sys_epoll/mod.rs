//! sys/epoll.h implementation for Redox, following http://man7.org/linux/man-pages/man7/epoll.7.html

use core::ptr;

use crate::{
    header::signal::sigset_t,
    platform::{types::*, PalEpoll, Sys},
};

pub use self::sys::*;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

pub const EPOLL_CTL_ADD: c_int = 1;
pub const EPOLL_CTL_DEL: c_int = 2;
pub const EPOLL_CTL_MOD: c_int = 3;

#[repr(C)]
#[derive(Clone, Copy)]
pub union epoll_data {
    pub ptr: *mut c_void,
    pub fd: c_int,
    pub u32: u32,
    pub u64: u64,
}
impl Default for epoll_data {
    fn default() -> Self {
        Self { u64: 0 }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
// This will match in size with syscall::Event (24 bytes on 64-bit
// systems) on redox. The `Default` trait is here so we don't need to
// worry about the padding when using this type.
pub struct epoll_event {
    pub events: u32, // 4 bytes
    // 4 automatic alignment bytes
    pub data: epoll_data, // 8 bytes

    #[cfg(target_os = "redox")]
    pub _pad: u64, // 8 bytes
}

#[no_mangle]
pub extern "C" fn epoll_create(_size: c_int) -> c_int {
    epoll_create1(0)
}

#[no_mangle]
pub extern "C" fn epoll_create1(flags: c_int) -> c_int {
    trace_expr!(Sys::epoll_create1(flags), "epoll_create1({:#x})", flags)
}

#[no_mangle]
pub extern "C" fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: *mut epoll_event) -> c_int {
    trace_expr!(
        Sys::epoll_ctl(epfd, op, fd, event),
        "epoll_ctl({}, {}, {}, {:p})",
        epfd,
        op,
        fd,
        event
    )
}

#[no_mangle]
pub extern "C" fn epoll_wait(
    epfd: c_int,
    events: *mut epoll_event,
    maxevents: c_int,
    timeout: c_int,
) -> c_int {
    epoll_pwait(epfd, events, maxevents, timeout, ptr::null())
}

#[no_mangle]
pub extern "C" fn epoll_pwait(
    epfd: c_int,
    events: *mut epoll_event,
    maxevents: c_int,
    timeout: c_int,
    sigmask: *const sigset_t,
) -> c_int {
    trace_expr!(
        Sys::epoll_pwait(epfd, events, maxevents, timeout, sigmask),
        "epoll_pwait({}, {:p}, {}, {}, {:p})",
        epfd,
        events,
        maxevents,
        timeout,
        sigmask
    )
}
