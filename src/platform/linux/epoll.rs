use core::{mem, ptr};

use super::{Sys, e_raw};
use crate::{
    error::Result,
    header::{bits_sigset_t::sigset_t, sys_epoll::epoll_event},
    platform::{
        PalEpoll,
        types::{c_int, size_t},
    },
};

impl PalEpoll for Sys {
    fn epoll_create1(flags: c_int) -> Result<c_int> {
        Ok(unsafe { e_raw(syscall!(EPOLL_CREATE1, flags))? as c_int })
    }

    fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: Option<&mut epoll_event>) -> Result<()> {
        unsafe {
            e_raw(syscall!(
                EPOLL_CTL,
                epfd,
                op,
                fd,
                event.map(ptr::from_mut).unwrap_or(ptr::null_mut())
            ))?;
        }
        Ok(())
    }

    fn epoll_pwait(
        epfd: c_int,
        events: &mut [epoll_event],
        timeout: c_int,
        sigmask: Option<&sigset_t>,
    ) -> Result<usize> {
        let sigsetsize: size_t = mem::size_of::<sigset_t>();
        unsafe {
            e_raw(syscall!(
                EPOLL_PWAIT,
                epfd,
                events.as_mut_ptr(),
                events.len(),
                timeout,
                sigmask.map(|x| ptr::from_ref(x)).unwrap_or(ptr::null()),
                sigsetsize
            ))
        }
    }
}
