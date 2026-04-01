//! `sys/uio.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_uio.h.html>.

use core::slice;

use crate::{
    header::{
        bits_iovec::{gather, iovec, scatter},
        errno, unistd,
    },
    platform::{
        self,
        types::{c_int, c_void, off_t, ssize_t},
    },
};

pub const IOV_MAX: c_int = 1024;

/// Non-POSIX, see <https://man7.org/linux/man-pages/man2/readv.2.html>.
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

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/readv.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn readv(fd: c_int, iov: *const iovec, iovcnt: c_int) -> ssize_t {
    if !(0..=IOV_MAX).contains(&iovcnt) {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    let iovs = unsafe { slice::from_raw_parts(iov, iovcnt as usize) };
    let mut vec = unsafe { gather(iovs) };

    let ret = unsafe { unistd::read(fd, vec.as_mut_ptr().cast::<c_void>(), vec.len()) };

    unsafe { scatter(iovs, vec) };

    ret
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/writev.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn writev(fd: c_int, iov: *const iovec, iovcnt: c_int) -> ssize_t {
    if !(0..=IOV_MAX).contains(&iovcnt) {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    let iovs = unsafe { slice::from_raw_parts(iov, iovcnt as usize) };
    let vec = unsafe { gather(iovs) };

    unsafe { unistd::write(fd, vec.as_ptr().cast::<c_void>(), vec.len()) }
}
