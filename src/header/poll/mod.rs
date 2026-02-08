//! `poll.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/poll.h.html>.

use core::{mem, ptr, slice};

use crate::{
    fs::File,
    header::{
        errno::EBADF,
        signal::sigset_t,
        sys_epoll::{
            EPOLL_CLOEXEC, EPOLL_CTL_ADD, EPOLLERR, EPOLLHUP, EPOLLIN, EPOLLNVAL, EPOLLOUT,
            EPOLLPRI, EPOLLRDBAND, EPOLLRDNORM, EPOLLWRBAND, EPOLLWRNORM, epoll_create1, epoll_ctl,
            epoll_data, epoll_event, epoll_pwait,
        },
        time::timespec,
    },
    platform::{
        self,
        types::{c_int, c_short, c_ulong},
    },
};

pub const POLLIN: c_short = 0x001;
pub const POLLPRI: c_short = 0x002;
pub const POLLOUT: c_short = 0x004;
pub const POLLERR: c_short = 0x008;
pub const POLLHUP: c_short = 0x010;
pub const POLLNVAL: c_short = 0x020;
pub const POLLRDNORM: c_short = 0x040;
pub const POLLRDBAND: c_short = 0x080;
pub const POLLWRNORM: c_short = 0x100;
pub const POLLWRBAND: c_short = 0x200;

pub type nfds_t = c_ulong;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/poll.h.html>.
#[repr(C)]
pub struct pollfd {
    pub fd: c_int,
    pub events: c_short,
    pub revents: c_short,
}

pub unsafe fn poll_epoll(fds: &mut [pollfd], timeout: c_int, sigmask: *const sigset_t) -> c_int {
    let event_map = [
        (POLLIN, EPOLLIN),
        (POLLPRI, EPOLLPRI),
        (POLLOUT, EPOLLOUT),
        (POLLERR, EPOLLERR),
        (POLLHUP, EPOLLHUP),
        (POLLNVAL, EPOLLNVAL),
        (POLLRDNORM, EPOLLRDNORM),
        (POLLWRNORM, EPOLLWRNORM),
        (POLLRDBAND, EPOLLRDBAND),
        (POLLWRBAND, EPOLLWRBAND),
    ];

    let ep = {
        let epfd = epoll_create1(EPOLL_CLOEXEC);
        if epfd < 0 {
            return -1;
        }
        File::new(epfd)
    };

    let mut closed = 0;
    for i in 0..fds.len() {
        let pfd = &mut fds[i];

        pfd.revents = 0;

        // Ignore the entry with negative fd
        if pfd.fd < 0 {
            continue;
        }

        let mut event = epoll_event {
            events: 0,
            data: epoll_data { u64: i as u64 },
            ..Default::default()
        };

        for (p, ep) in event_map.iter() {
            if pfd.events & p > 0 {
                event.events |= ep;
            }
        }

        if unsafe { epoll_ctl(*ep, EPOLL_CTL_ADD, pfd.fd, &mut event) } < 0 {
            if platform::ERRNO.get() == EBADF {
                pfd.revents |= POLLNVAL;
                closed += 1;
            } else {
                return -1;
            }
        }
    }

    // Early exit if there are fds, and all are closed (revents = POLLNVAL)
    if closed > 0 && closed == fds.len() {
        return closed as i32;
    }

    let mut events: [epoll_event; 32] = unsafe { mem::zeroed() };
    let res = unsafe {
        epoll_pwait(
            *ep,
            events.as_mut_ptr(),
            events.len() as c_int,
            timeout,
            sigmask,
        )
    };
    if res < 0 {
        return -1;
    }

    for event in events.iter().take(res as usize) {
        let pi = unsafe { event.data.u64 as usize };
        // TODO: Error status when fd does not match?
        if let Some(pfd) = fds.get_mut(pi) {
            for (p, ep) in event_map.iter() {
                if event.events & ep > 0 {
                    pfd.revents |= p;
                }
            }
        }
    }

    let mut count = 0;
    for pfd in fds.iter() {
        if pfd.revents > 0 {
            count += 1;
        }
    }
    count
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/poll.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn poll(fds: *mut pollfd, nfds: nfds_t, timeout: c_int) -> c_int {
    trace_expr!(
        unsafe {
            poll_epoll(
                slice::from_raw_parts_mut(fds, nfds as usize),
                timeout,
                ptr::null_mut(),
            )
        },
        "poll({:p}, {}, {})",
        fds,
        nfds,
        timeout,
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ppoll.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ppoll(
    fds: *mut pollfd,
    nfds: nfds_t,
    tmo_p: *const timespec,
    sigmask: *const sigset_t,
) -> c_int {
    let timeout = if tmo_p.is_null() {
        -1
    } else {
        let tmo = unsafe { &*tmo_p };
        if tmo.tv_sec > (c_int::MAX / 1000) as _ {
            c_int::MAX
        } else {
            ((tmo.tv_sec as c_int) * 1000) + ((tmo.tv_nsec as c_int) / 1000000)
        }
    };
    trace_expr!(
        unsafe {
            poll_epoll(
                slice::from_raw_parts_mut(fds, nfds as usize),
                timeout,
                sigmask,
            )
        },
        "ppoll({:p}, {}, {:p}, {:p})",
        fds,
        nfds,
        tmo_p,
        sigmask
    )
}
