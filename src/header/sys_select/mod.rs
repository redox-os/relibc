//! sys/select.h implementation

use core::mem;

use cbitset::BitSet;

use crate::{
    fs::File,
    header::{
        errno,
        sys_epoll::{
            epoll_create1, epoll_ctl, epoll_data, epoll_event, epoll_wait, EPOLLERR, EPOLLIN,
            EPOLLOUT, EPOLL_CLOEXEC, EPOLL_CTL_ADD,
        },
        sys_time::timeval,
    },
    platform::{self, types::*},
};

// fd_set is also defined in C because cbindgen is incompatible with mem::size_of booo

pub const FD_SETSIZE: usize = 1024;
type bitset = BitSet<[c_ulong; FD_SETSIZE / (8 * mem::size_of::<c_ulong>())]>;

#[repr(C)]
pub struct fd_set {
    pub fds_bits: bitset,
}

pub fn select_epoll(
    nfds: c_int,
    readfds: Option<&mut fd_set>,
    writefds: Option<&mut fd_set>,
    exceptfds: Option<&mut fd_set>,
    timeout: Option<&mut timeval>,
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

    let mut read_bitset: Option<&mut bitset> = readfds.map(|fd_set| &mut fd_set.fds_bits);
    let mut write_bitset: Option<&mut bitset> = writefds.map(|fd_set| &mut fd_set.fds_bits);
    let mut except_bitset: Option<&mut bitset> = exceptfds.map(|fd_set| &mut fd_set.fds_bits);

    // Keep track of the number of file descriptors that do not support epoll
    let mut not_epoll = 0;
    for fd in 0..nfds {
        let mut events = 0;

        if let Some(ref fd_set) = read_bitset {
            if fd_set.contains(fd as usize) {
                events |= EPOLLIN;
            }
        }

        if let Some(ref fd_set) = write_bitset {
            if fd_set.contains(fd as usize) {
                events |= EPOLLOUT;
            }
        }

        if let Some(ref fd_set) = except_bitset {
            if fd_set.contains(fd as usize) {
                events |= EPOLLERR;
            }
        }

        if events > 0 {
            let mut event = epoll_event {
                events,
                data: epoll_data { fd },
                ..Default::default()
            };
            if epoll_ctl(*ep, EPOLL_CTL_ADD, fd, &mut event) < 0 {
                if unsafe { platform::errno == errno::EPERM } {
                    not_epoll += 1;
                } else {
                    return -1;
                }
            } else {
                if let Some(ref mut fd_set) = read_bitset {
                    if fd_set.contains(fd as usize) {
                        fd_set.remove(fd as usize);
                    }
                }

                if let Some(ref mut fd_set) = write_bitset {
                    if fd_set.contains(fd as usize) {
                        fd_set.remove(fd as usize);
                    }
                }

                if let Some(ref mut fd_set) = except_bitset {
                    if fd_set.contains(fd as usize) {
                        fd_set.remove(fd as usize);
                    }
                }
            }
        }
    }

    let mut events: [epoll_event; 32] = unsafe { mem::zeroed() };
    let epoll_timeout = if not_epoll > 0 {
        // Do not wait if any non-epoll file descriptors were found
        0
    } else {
        match timeout {
            Some(timeout) => {
                //TODO: Check for overflow
                ((timeout.tv_sec as c_int) * 1000) + ((timeout.tv_usec as c_int) / 1000)
            }
            None => -1,
        }
    };
    let res = epoll_wait(
        *ep,
        events.as_mut_ptr(),
        events.len() as c_int,
        epoll_timeout,
    );
    if res < 0 {
        return -1;
    }

    let mut count = not_epoll;
    for event in events.iter().take(res as usize) {
        let fd = unsafe { event.data.fd };
        // TODO: Error status when fd does not match?
        if fd >= 0 && fd < FD_SETSIZE as c_int {
            if event.events & EPOLLIN > 0 {
                if let Some(ref mut fd_set) = read_bitset {
                    fd_set.insert(fd as usize);
                    count += 1;
                }
            }
            if event.events & EPOLLOUT > 0 {
                if let Some(ref mut fd_set) = write_bitset {
                    fd_set.insert(fd as usize);
                    count += 1;
                }
            }
            if event.events & EPOLLERR > 0 {
                if let Some(ref mut fd_set) = except_bitset {
                    fd_set.insert(fd as usize);
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
