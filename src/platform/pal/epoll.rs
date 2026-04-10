use crate::{
    error::Result,
    header::{signal::sigset_t, sys_epoll::epoll_event},
    platform::{Pal, types::c_int},
};

/// Platform abstraction for `epoll` functionality.
pub trait PalEpoll: Pal {
    /// Platform implementation of [`epoll_create1()`](crate::header::sys_epoll::epoll_create1) from [`sys/epoll.h`](crate::header::sys_epoll).
    fn epoll_create1(flags: c_int) -> Result<c_int>;

    /// Platform implementation of [`epoll_ctl()`](crate::header::sys_epoll::epoll_ctl) from [`sys/epoll.h`](crate::header::sys_epoll).
    unsafe fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: *mut epoll_event) -> Result<()>;

    /// Platform implementation of [`epoll_pwait()`](crate::header::sys_epoll::epoll_pwait) from [`sys/epoll.h`](crate::header::sys_epoll).
    unsafe fn epoll_pwait(
        epfd: c_int,
        events: *mut epoll_event,
        maxevents: c_int,
        timeout: c_int,
        sigmask: *const sigset_t,
    ) -> Result<usize>;
}
