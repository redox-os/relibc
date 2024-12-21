//! poll implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/poll.h.html

// TODO: set this for entire crate when possible
#![deny(unsafe_op_in_unsafe_fn)]

use core::{mem, slice};

use crate::{
    error::ResultExt,
    fs::File,
    header::sys_epoll::{
        epoll_create1, epoll_ctl, epoll_data, epoll_event, epoll_wait, EPOLLERR, EPOLLHUP, EPOLLIN,
        EPOLLNVAL, EPOLLOUT, EPOLLPRI, EPOLLRDBAND, EPOLLRDNORM, EPOLLWRBAND, EPOLLWRNORM,
        EPOLL_CLOEXEC, EPOLL_CTL_ADD,
    },
    platform::types::*,
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

#[repr(C)]
pub struct pollfd {
    pub fd: c_int,
    pub events: c_short,
    pub revents: c_short,
}

pub fn poll_epoll(fds: &mut [pollfd], timeout: c_int) -> c_int {
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

    for i in 0..fds.len() {
        let mut pfd = &mut fds[i];

        // Ignore the entry with negative fd, set the revents to 0
        if pfd.fd < 0 {
            pfd.revents = 0;
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

        pfd.revents = 0;

        if unsafe { epoll_ctl(*ep, EPOLL_CTL_ADD, pfd.fd, &mut event) } < 0 {
            return -1;
        }
    }

    let mut events: [epoll_event; 32] = unsafe { mem::zeroed() };
    let res = unsafe { epoll_wait(*ep, events.as_mut_ptr(), events.len() as c_int, timeout) };
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

#[no_mangle]
pub unsafe extern "C" fn poll(fds: *mut pollfd, nfds: nfds_t, timeout: c_int) -> c_int {
    trace_expr!(
        poll_epoll(
            unsafe { slice::from_raw_parts_mut(fds, nfds as usize) },
            timeout
        ),
        "poll({:p}, {}, {})",
        fds,
        nfds,
        timeout
    )
}
