use crate::{
    error::{Errno, Result},
    header::{
        bits_sigset_t::sigset_t,
        errno,
        poll::{self, pollfd},
        sys_epoll,
    },
    platform::{Sys, sys::FdGuard, types::c_int},
};
use redox_protocols::protocol::O_CLOEXEC;

impl Sys {
    /// Wrapper to use epoll for poll
    /// TODO: need to use /scheme/event directly with upper table
    pub(crate) fn redox_poll(
        fds: &mut [pollfd],
        timeout: c_int,
        sigmask: Option<&sigset_t>,
    ) -> Result<c_int> {
        let event_map = [
            (poll::POLLIN, sys_epoll::EPOLLIN),
            (poll::POLLPRI, sys_epoll::EPOLLPRI),
            (poll::POLLOUT, sys_epoll::EPOLLOUT),
            (poll::POLLERR, sys_epoll::EPOLLERR),
            (poll::POLLHUP, sys_epoll::EPOLLHUP),
            (poll::POLLNVAL, sys_epoll::EPOLLNVAL),
            (poll::POLLRDNORM, sys_epoll::EPOLLRDNORM),
            (poll::POLLWRNORM, sys_epoll::EPOLLWRNORM),
            (poll::POLLRDBAND, sys_epoll::EPOLLRDBAND),
            (poll::POLLWRBAND, sys_epoll::EPOLLWRBAND),
        ];

        let ep = FdGuard::open_into_upper("/scheme/event", syscall::O_RDWR | O_CLOEXEC)?;

        let mut closed = 0;
        for (i, fd) in fds.iter_mut().enumerate() {
            let pfd = fd;

            pfd.revents = 0;

            // Ignore the entry with negative fd
            if pfd.fd < 0 {
                continue;
            }

            let mut event = sys_epoll::epoll_event::new(0, sys_epoll::epoll_data { u64: i as u64 });

            for (p, ep) in event_map.iter() {
                if pfd.events & p > 0 {
                    event.events |= ep;
                }
            }

            match Self::redox_epoll_ctl(
                ep.as_raw_fd(),
                sys_epoll::EPOLL_CTL_ADD,
                pfd.fd,
                Some(&mut event),
            ) {
                Ok(()) => {}
                Err(Errno(errno::EBADF)) => {
                    pfd.revents |= poll::POLLNVAL;
                    closed += 1;
                }
                Err(Errno(err)) => {
                    return Err(Errno(err));
                }
            }
        }

        // Early exit if there are fds, and all are closed (revents = POLLNVAL)
        if closed > 0 && closed == fds.len() {
            // TODO should we panic if fd.len() is greater than i32::MAX instead?
            // seems unlikely we'll be dealing with that many file descriptors
            return Ok(i32::try_from(closed).unwrap_or(i32::MAX));
        }

        let mut events = [sys_epoll::epoll_event::default(); 32];

        match Self::redox_epoll_pwait(ep.as_raw_fd(), &mut events, timeout, sigmask) {
            Ok(res) => {
                for event in events.iter().take(res) {
                    let pi: usize = event.data.into();
                    // TODO: Error status when fd does not match?
                    if let Some(pfd) = fds.get_mut(pi) {
                        for (p, ep) in event_map.iter() {
                            if event.events & ep > 0 {
                                pfd.revents |= p;
                            }
                        }
                    }
                }
            }
            Err(Errno(err)) => {
                if err == errno::EINTR && closed > 0 {
                    // some fds are closed by signal
                } else {
                    return Err(Errno(err));
                }
            }
        }

        let mut count = 0;
        for pfd in fds.iter() {
            if pfd.revents > 0 {
                count += 1;
            }
        }
        Ok(count)
    }
}
