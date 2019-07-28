//! sys/uio implementation for Redox, following http://pubs.opengroup.org/onlinepubs/007904875/basedefs/sys/uio.h.html

use alloc::vec::Vec;
use core::slice;

use crate::{
    header::{errno, unistd},
    platform::{self, types::*},
};

pub const IOV_MAX: c_int = 1024;

#[repr(C)]
pub struct iovec {
    iov_base: *mut c_void,
    iov_len: size_t,
}

impl iovec {
    unsafe fn to_slice(&self) -> &mut [u8] {
        slice::from_raw_parts_mut(self.iov_base as *mut u8, self.iov_len as usize)
    }
}

unsafe fn gather(iovs: &[iovec]) -> Vec<u8> {
    let mut vec = Vec::new();
    for iov in iovs.iter() {
        vec.extend_from_slice(iov.to_slice());
    }
    vec
}

unsafe fn scatter(iovs: &[iovec], vec: Vec<u8>) {
    let mut i = 0;
    for iov in iovs.iter() {
        let slice = iov.to_slice();
        slice.copy_from_slice(&vec[i..]);
        i += slice.len();
    }
}

#[no_mangle]
pub unsafe extern "C" fn readv(fd: c_int, iov: *const iovec, iovcnt: c_int) -> ssize_t {
    if iovcnt < 0 || iovcnt > IOV_MAX {
        platform::errno = errno::EINVAL;
        return -1;
    }

    let iovs = slice::from_raw_parts(iov, iovcnt as usize);
    let mut vec = gather(iovs);

    let ret = unistd::read(fd, vec.as_mut_ptr() as *mut c_void, vec.len());

    scatter(iovs, vec);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn writev(fd: c_int, iov: *const iovec, iovcnt: c_int) -> ssize_t {
    if iovcnt < 0 || iovcnt > IOV_MAX {
        platform::errno = errno::EINVAL;
        return -1;
    }

    let iovs = slice::from_raw_parts(iov, iovcnt as usize);
    let vec = gather(iovs);

    unistd::write(fd, vec.as_ptr() as *const c_void, vec.len())
}
