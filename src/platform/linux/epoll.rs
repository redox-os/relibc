use super::{
    super::{types::*, PalEpoll},
    e_raw, Sys,
};
use crate::{
    errno::Errno,
    header::{signal::sigset_t, sys_epoll::epoll_event},
};

impl PalEpoll for Sys {
    fn epoll_create1(flags: c_int) -> Result<c_int, Errno> {
        e_raw(unsafe { syscall!(EPOLL_CREATE1, flags) }).map(|res| res as c_int)
    }

    fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: *mut epoll_event) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(EPOLL_CTL, epfd, op, fd, event) })?;
        Ok(())
    }

    fn epoll_pwait(
        epfd: c_int,
        events: *mut epoll_event,
        maxevents: c_int,
        timeout: c_int,
        sigmask: *const sigset_t,
    ) -> Result<c_int, Errno> {
        e_raw(unsafe { syscall!(EPOLL_PWAIT, epfd, events, maxevents, timeout, sigmask) })
            .map(|res| res as c_int)
    }
}
