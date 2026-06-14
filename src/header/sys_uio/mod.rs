//! `sys/uio.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_uio.h.html>.

use core::slice;

use crate::{
    header::{errno, limits::IOV_MAX, unistd},
    platform::{
        self,
        types::{c_int, c_void, off_t, ssize_t},
    },
};

pub use crate::header::bits_iovec::{gather, iovec, scatter};

// TODO should be guarded by _DEFAULT_SOURCE or _BSD_SOURCE
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

// TODO should be guarded by _DEFAULT_SOURCE or _BSD_SOURCE
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

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/readv.html>.
///
/// Equivalent to `read()` but places the input data into the `iovcnt` buffers
/// specified by the members of the `iov` array.
///
/// When successful, returns a non-negative number indicating the number of
/// bytes actually read. Upon failure, returns `-1`.
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
///
/// Equivalent to `write()` but shall gather output data from the `iovcnt`
/// buffers specified by the members of the `iov` array.
///
/// When successful, returns a non-negative number indicating the number of
/// bytes actually written to the file associated with `fildes`. Upon failure,
/// returns `-1`.
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
