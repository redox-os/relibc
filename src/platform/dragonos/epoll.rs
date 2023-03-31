use super::{
    super::{types::*, PalEpoll},
    e, Sys,
};
use crate::header::{signal::sigset_t, sys_epoll::epoll_event};

impl PalEpoll for Sys {
    fn epoll_create1(flags: c_int) -> c_int {
        unsafe { e(syscall!(EPOLL_CREATE1, flags)) as c_int }
    }

    fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: *mut epoll_event) -> c_int {
        unsafe { e(syscall!(EPOLL_CTL, epfd, op, fd, event)) as c_int }
    }

    fn epoll_pwait(
        epfd: c_int,
        events: *mut epoll_event,
        maxevents: c_int,
        timeout: c_int,
        sigmask: *const sigset_t,
    ) -> c_int {
        unsafe {
            e(syscall!(
                EPOLL_PWAIT,
                epfd,
                events,
                maxevents,
                timeout,
                sigmask
            )) as c_int
        }
    }
}
