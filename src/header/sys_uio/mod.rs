//! `sys/uio.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_uio.h.html>.

use core::slice;

use crate::{
    header::{errno, limits::IOV_MAX, unistd},
    platform::{
        self,
        types::{c_int, c_void, ssize_t},
    },
};

pub use crate::header::bits_iovec::{gather, iovec, scatter};

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
