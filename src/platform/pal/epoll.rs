use super::super::types::*;
use super::super::Pal;
use header::signal::sigset_t;
use header::sys_epoll::epoll_event;

pub trait PalEpoll: Pal {
    fn epoll_create1(flags: c_int) -> c_int;
    fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: *mut epoll_event) -> c_int;
    fn epoll_pwait(epfd: c_int, events: *mut epoll_event, maxevents: c_int, timeout: c_int, sigmask: *const sigset_t) -> c_int;
}
