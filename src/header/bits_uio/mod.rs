//! `preadv` and `pwritev` implementation for `sys/uio.h`.
//!
//! Non-POSIX extensions, see <https://man7.org/linux/man-pages/man2/readv.2.html>.

use core::slice;

use crate::{
    header::{
        bits_iovec::{gather, iovec, scatter},
        errno,
        limits::IOV_MAX,
        unistd,
    },
    platform::{
        self,
        types::{c_int, c_void, off_t, ssize_t},
    },
};

/// Non-POSIX, see <https://man7.org/linux/man-pages/man2/readv.2.html>.
///
/// Combines the functionality of `readv()` and `pread()`.
///
/// When successful, returns a non-negative number indicating the number of
/// bytes actually read. Upon failure, returns `-1`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn preadv(
    fd: c_int,
    iov: *const iovec,
    iovcnt: c_int,
    offset: off_t,
) -> ssize_t {
    if !(0..=IOV_MAX).contains(&iovcnt) {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    let iovs = unsafe { slice::from_raw_parts(iov, iovcnt as usize) };
    let mut vec = unsafe { gather(iovs) };

    let ret = unsafe { unistd::pread(fd, vec.as_mut_ptr().cast::<c_void>(), vec.len(), offset) };

    unsafe { scatter(iovs, vec) };

    ret
}

/// Non-POSIX, see <https://man7.org/linux/man-pages/man2/readv.2.html>.
///
/// Combined the functionality of `writev()` and `pwrite()`.
///
/// When successful, returns a non-negative number indicating the number of
/// bytes actually written. Upon failure, returns `-1`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pwritev(
    fd: c_int,
    iov: *const iovec,
    iovcnt: c_int,
    offset: off_t,
) -> ssize_t {
    if !(0..=IOV_MAX).contains(&iovcnt) {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    let iovs = unsafe { slice::from_raw_parts(iov, iovcnt as usize) };
    let vec = unsafe { gather(iovs) };

    unsafe { unistd::pwrite(fd, vec.as_ptr().cast::<c_void>(), vec.len(), offset) }
}
