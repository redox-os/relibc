use crate::{
    error::{Errno, Result},
    header::{signal::sigset_t, sys_epoll::epoll_event},
    platform::{types::*, Pal},
};

pub trait PalEpoll: Pal {
    fn epoll_create1(flags: c_int) -> Result<c_int>;
    unsafe fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: *mut epoll_event) -> Result<()>;
    unsafe fn epoll_pwait(
        epfd: c_int,
        events: *mut epoll_event,
        maxevents: c_int,
        timeout: c_int,
        sigmask: *const sigset_t,
    ) -> Result<usize>;
}
