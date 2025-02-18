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

#[no_mangle]
pub unsafe extern "C" fn readv(fd: c_int, iov: *const iovec, iovcnt: c_int) -> ssize_t {
    if iovcnt < 0 || iovcnt > IOV_MAX {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    let iovs = unsafe { slice::from_raw_parts(iov, iovcnt as usize) };
    let mut vec = unsafe { gather(iovs) };

    let vec_ptr = vec.as_mut_ptr();
    let vec_len = vec.len();
    let ret = unsafe { unistd::read(fd, vec_ptr as *mut c_void, vec_len) };

    unsafe { scatter(iovs, vec) };

    ret
}

#[no_mangle]
pub unsafe extern "C" fn writev(fd: c_int, iov: *const iovec, iovcnt: c_int) -> ssize_t {
    if iovcnt < 0 || iovcnt > IOV_MAX {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    let iovs = unsafe { slice::from_raw_parts(iov, iovcnt as usize) };
    let vec = unsafe { gather(iovs) };

    let vec_ptr = vec.as_ptr();
    let vec_len = vec.len();
    unsafe { unistd::write(fd, vec_ptr as *const c_void, vec_len) }
}
