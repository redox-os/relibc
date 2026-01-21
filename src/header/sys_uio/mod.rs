//! `sys/uio.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_uio.h.html>.

use alloc::vec::Vec;
use core::slice;

use crate::{
    header::{errno, unistd},
    platform::{
        self,
        types::{c_int, c_void, off_t, size_t, ssize_t},
    },
};

pub const IOV_MAX: c_int = 1024;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_uio.h.html>.
#[repr(C)]
#[derive(Debug, CheckVsLibcCrate)]
pub struct iovec {
    pub iov_base: *mut c_void,
    pub iov_len: size_t,
}

impl iovec {
    unsafe fn to_slice(&self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.iov_base as *mut u8, self.iov_len as usize) }
    }
}

unsafe fn gather(iovs: &[iovec]) -> Vec<u8> {
    let mut vec = Vec::new();
    for iov in iovs.iter() {
        vec.extend_from_slice(unsafe { iov.to_slice() });
    }
    vec
}

unsafe fn scatter(iovs: &[iovec], vec: Vec<u8>) {
    let mut i = 0;
    for iov in iovs.iter() {
        let slice = unsafe { iov.to_slice() };
        slice.copy_from_slice(&vec[i..][..slice.len()]);
        i += slice.len();
    }
}

/// Non-POSIX, see <https://man7.org/linux/man-pages/man2/readv.2.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn preadv(
    fd: c_int,
    iov: *const iovec,
    iovcnt: c_int,
    offset: off_t,
) -> ssize_t {
    if iovcnt < 0 || iovcnt > IOV_MAX {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    let iovs = unsafe { slice::from_raw_parts(iov, iovcnt as usize) };
    let mut vec = unsafe { gather(iovs) };

    let ret = unsafe { unistd::pread(fd, vec.as_mut_ptr() as *mut c_void, vec.len(), offset) };

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
    if iovcnt < 0 || iovcnt > IOV_MAX {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    let iovs = unsafe { slice::from_raw_parts(iov, iovcnt as usize) };
    let vec = unsafe { gather(iovs) };

    unsafe { unistd::pwrite(fd, vec.as_ptr() as *const c_void, vec.len(), offset) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/readv.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn readv(fd: c_int, iov: *const iovec, iovcnt: c_int) -> ssize_t {
    if iovcnt < 0 || iovcnt > IOV_MAX {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    let iovs = unsafe { slice::from_raw_parts(iov, iovcnt as usize) };
    let mut vec = unsafe { gather(iovs) };

    let ret = unsafe { unistd::read(fd, vec.as_mut_ptr() as *mut c_void, vec.len()) };

    unsafe { scatter(iovs, vec) };

    ret
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/writev.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn writev(fd: c_int, iov: *const iovec, iovcnt: c_int) -> ssize_t {
    if iovcnt < 0 || iovcnt > IOV_MAX {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    let iovs = unsafe { slice::from_raw_parts(iov, iovcnt as usize) };
    let vec = unsafe { gather(iovs) };

    unsafe { unistd::write(fd, vec.as_ptr() as *const c_void, vec.len()) }
}
