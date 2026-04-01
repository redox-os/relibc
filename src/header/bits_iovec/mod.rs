use alloc::vec::Vec;
use core::slice;

use crate::platform::types::{c_void, size_t};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_uio.h.html>.
#[repr(C)]
#[derive(Debug, CheckVsLibcCrate)]
pub struct iovec {
    pub iov_base: *mut c_void,
    pub iov_len: size_t,
}

impl iovec {
    unsafe fn to_slice(&self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.iov_base.cast::<u8>(), self.iov_len) }
    }
}

pub unsafe fn gather(iovs: &[iovec]) -> Vec<u8> {
    let mut vec = Vec::new();
    for iov in iovs.iter() {
        vec.extend_from_slice(unsafe { iov.to_slice() });
    }
    vec
}

pub unsafe fn scatter(iovs: &[iovec], vec: Vec<u8>) {
    let mut i = 0;
    for iov in iovs.iter() {
        let slice = unsafe { iov.to_slice() };
        slice.copy_from_slice(&vec[i..][..slice.len()]);
        i += slice.len();
    }
}
