//! sys/select.h implementation

use core::mem;

use fs::File;
use header::errno;
use header::sys_epoll::{
    epoll_create1, EPOLL_CLOEXEC,
    epoll_ctl, EPOLL_CTL_ADD,
    epoll_wait,
    EPOLLIN, EPOLLOUT, EPOLLERR,
    epoll_data, epoll_event
};
use header::sys_time::timeval;
use platform;
use platform::types::*;

// fd_set is also defined in C because cbindgen is incompatible with mem::size_of booo

pub const FD_SETSIZE: usize = 1024;

#[repr(C)]
pub struct fd_set {
    pub fds_bits: [c_ulong; FD_SETSIZE / (8 * mem::size_of::<c_ulong>())],
}

impl fd_set {
    fn index(fd: c_int) -> usize {
        (fd as usize) / (8 * mem::size_of::<c_ulong>())
    }

    fn bitmask(fd: c_int) -> c_ulong {
        1 << ((fd as usize) & (8 * mem::size_of::<c_ulong>() - 1)) as c_ulong
    }

    fn zero(&mut self) {
        for i in 0..self.fds_bits.len() {
            self.fds_bits[i] = 0;
        }
    }

    fn set(&mut self, fd: c_int) {
        self.fds_bits[Self::index(fd)] |= Self::bitmask(fd);
    }

    fn clr(&mut self, fd: c_int) {
        self.fds_bits[Self::index(fd)] &= !Self::bitmask(fd);
    }

    fn isset(&self, fd: c_int) -> bool {
        self.fds_bits[Self::index(fd)] & Self::bitmask(fd) > 0
    }
}

pub fn select_epoll(
    nfds: c_int,
    mut readfds: Option<&mut fd_set>,
    mut writefds: Option<&mut fd_set>,
    mut exceptfds: Option<&mut fd_set>,
    timeout: Option<&mut timeval>
) -> c_int {
    if nfds < 0 || nfds > FD_SETSIZE as i32 {
        unsafe { platform::errno = errno::EINVAL };
        return -1;
    };

    let ep = {
        let epfd = epoll_create1(EPOLL_CLOEXEC);
        if epfd < 0 {
            return -1;
        }
        File::new(epfd)
    };

    for fd in 0..nfds {
        if let Some(ref fd_set) = readfds {
            if fd_set.isset(fd) {
                let mut event = epoll_event {
                    events: EPOLLIN,
                    data: epoll_data {
                        fd: fd,
                    },
                    ..Default::default()
                };
                if epoll_ctl(*ep, EPOLL_CTL_ADD, fd, &mut event) < 0 {
                    return -1;
                }
            }
        }
        if let Some(ref fd_set) = writefds {
            if fd_set.isset(fd) {
                let mut event = epoll_event {
                    events: EPOLLOUT,
                    data: epoll_data {
                        fd: fd,
                    },
                    ..Default::default()
                };
                if epoll_ctl(*ep, EPOLL_CTL_ADD, fd, &mut event) < 0 {
                    return -1;
                }
            }
        }
        if let Some(ref fd_set) = exceptfds {
            if fd_set.isset(fd) {
                let mut event = epoll_event {
                    events: EPOLLERR,
                    data: epoll_data {
                        fd: fd,
                    },
                    ..Default::default()
                };
                if epoll_ctl(*ep, EPOLL_CTL_ADD, fd, &mut event) < 0 {
                    return -1;
                }
            }
        }
    }

    let mut events: [epoll_event; 32] = unsafe { mem::zeroed() };
    let res = epoll_wait(
        *ep,
        events.as_mut_ptr(),
        events.len() as c_int,
        match timeout {
            Some(timeout) => {
                //TODO: Check for overflow
                ((timeout.tv_sec as c_int) * 1000) +
                ((timeout.tv_usec as c_int) / 1000)
            },
            None => -1
        }
    );
    if res < 0 {
        return -1;
    }

    if let Some(ref mut fd_set) = readfds {
        fd_set.zero();
    }
    if let Some(ref mut fd_set) = writefds {
        fd_set.zero();
    }
    if let Some(ref mut fd_set) = exceptfds {
        fd_set.zero();
    }

    let mut count = 0;
    for i in 0..res as usize {
        let event = &events[i];
        let fd = unsafe { event.data.fd };
        // TODO: Error status when fd does not match?
        if fd >= 0 && fd <= FD_SETSIZE as c_int {
            if event.events & EPOLLIN > 0 {
                if let Some(ref mut fd_set) = readfds {
                    fd_set.set(fd);
                    count += 1;
                }
            }
            if event.events & EPOLLOUT > 0 {
                if let Some(ref mut fd_set) = writefds {
                    fd_set.set(fd);
                    count += 1;
                }
            }
            if event.events & EPOLLERR > 0 {
                if let Some(ref mut fd_set) = exceptfds {
                    fd_set.set(fd);
                    count += 1;
                }
            }
        }
    }
    count
}

#[no_mangle]
pub unsafe extern "C" fn select(
    nfds: c_int,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    exceptfds: *mut fd_set,
    timeout: *mut timeval,
) -> c_int {
    trace_expr!(
        select_epoll(
            nfds,
            if readfds.is_null() {
                None
            } else {
                Some(&mut *readfds)
            },
            if writefds.is_null() {
                None
            } else {
                Some(&mut *writefds)
            },
            if exceptfds.is_null() {
                None
            } else {
                Some(&mut *exceptfds)
            },
            if timeout.is_null() {
                None
            } else {
                Some(&mut *timeout)
            }
        ),
        "select({}, {:p}, {:p}, {:p}, {:p})",
        nfds,
        readfds,
        writefds,
        exceptfds,
        timeout
    )
}
