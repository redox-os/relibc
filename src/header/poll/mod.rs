//! poll implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/poll.h.html

use core::{mem, slice};

use crate::{
    fs::File,
    header::sys_epoll::{
        epoll_create1, epoll_ctl, epoll_data, epoll_event, epoll_wait, EPOLLERR, EPOLLHUP, EPOLLIN,
        EPOLLNVAL, EPOLLOUT, EPOLLPRI, EPOLL_CLOEXEC, EPOLL_CTL_ADD,
    },
    platform::types::*,
};

pub const POLLIN: c_short = 0x001;
pub const POLLPRI: c_short = 0x002;
pub const POLLOUT: c_short = 0x004;
pub const POLLERR: c_short = 0x008;
pub const POLLHUP: c_short = 0x010;
pub const POLLNVAL: c_short = 0x020;

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

        if epoll_ctl(*ep, EPOLL_CTL_ADD, pfd.fd, &mut event) < 0 {
            return -1;
        }
    }

    let mut events: [epoll_event; 32] = unsafe { mem::zeroed() };
    let res = epoll_wait(*ep, events.as_mut_ptr(), events.len() as c_int, timeout);
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
        poll_epoll(slice::from_raw_parts_mut(fds, nfds as usize), timeout),
        "poll({:p}, {}, {})",
        fds,
        nfds,
        timeout
    )
}
