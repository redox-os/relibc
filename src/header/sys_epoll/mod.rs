//! sys/epoll.h implementation for Redox, following http://man7.org/linux/man-pages/man7/epoll.7.html

use core::ptr;

use header::signal::sigset_t;
use platform::{PalEpoll, Sys};
use platform::types::*;

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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct epoll_event {
    pub events: u32,
    pub data: epoll_data,
}

#[no_mangle]
pub extern "C" fn epoll_create(_size: c_int) -> c_int {
    epoll_create1(0)
}

#[no_mangle]
pub extern "C" fn epoll_create1(flags: c_int) -> c_int {
    trace_expr!(
        Sys::epoll_create1(flags),
        "epoll_create1({:#x})",
        flags
    )
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
pub extern "C" fn epoll_wait(epfd: c_int, events: *mut epoll_event, maxevents: c_int, timeout: c_int) -> c_int {
    epoll_pwait(epfd, events, maxevents, timeout, ptr::null())
}

#[no_mangle]
pub extern "C" fn epoll_pwait(epfd: c_int, events: *mut epoll_event, maxevents: c_int, timeout: c_int, sigmask: *const sigset_t) -> c_int {
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
