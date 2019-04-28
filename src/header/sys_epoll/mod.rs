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

pub const EPOLLIN: u32 =     0x0001;
pub const EPOLLPRI: u32 =    0x0002;
pub const EPOLLOUT: u32 =    0x0004;
pub const EPOLLERR: u32 =    0x0008;
pub const EPOLLHUP: u32 =    0x0010;
pub const EPOLLNVAL: u32 =   0x0020;
pub const EPOLLRDNORM: u32 = 0x0040;
pub const EPOLLRDBAND: u32 = 0x0080;
pub const EPOLLWRNORM: u32 = 0x0100;
pub const EPOLLWRBAND: u32 = 0x0200;
pub const EPOLLMSG: u32 =    0x0400;
pub const EPOLLRDHUP: u32 =  0x2000;

#[repr(C)]
pub union epoll_data {
    pub ptr: *mut c_void,
    pub fd: c_int,
    pub u32: u32,
    pub u64: u64,
}

#[repr(C)]
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
