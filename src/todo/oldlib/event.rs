use core::slice;
use libc::{c_int, c_uint};
use syscall::error::{Error, EINVAL};
use types::{fd_set, pollfd, timeval, FD_SETSIZE, POLLIN, POLLOUT, NFDBITS};

libc_fn!(unsafe poll(fds: *mut pollfd, nfds: c_uint, timeout: c_int) -> Result<c_int> {
    let fds = slice::from_raw_parts_mut(fds, nfds as usize);

    let mut ret = 0;
    for fd in fds.iter_mut() {
        // always ready for read or write
        fd.revents = fd.events & (POLLIN | POLLOUT);
        if fd.revents != 0 {
            ret += 1;
        }
    }

    Ok(ret)
});

libc_fn!(unsafe select(nfds: c_int, readfds: *mut fd_set, writefds: *mut fd_set, errorfds: *mut fd_set, _timeout: *mut timeval) -> Result<c_int> {
    if nfds < 0 || nfds > FD_SETSIZE as i32 {
        return Err(Error::new(EINVAL));
    }

    let mut ret = 0;
    for i in 0..nfds as usize {
        if ! readfds.is_null() {
            // always ready to read
            if ((*readfds).fds_bits[i/NFDBITS] & (1 << (i % NFDBITS))) != 0 {
                ret += 1;
            }
        }

        if ! writefds.is_null() {
            // always ready to write
            if ((*writefds).fds_bits[i/NFDBITS] & (1 << (i % NFDBITS))) != 0 {
                ret += 1;
            }
        }

        if ! errorfds.is_null() {
            // report no errors
            if ((*errorfds).fds_bits[i/NFDBITS] & (1 << (i % NFDBITS))) != 0 {
                (*errorfds).fds_bits[i/NFDBITS] &= !(1 << (i % NFDBITS));
            }
        }
    }

    Ok(ret)
});
