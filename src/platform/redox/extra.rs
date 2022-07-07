use core::{mem::size_of, ptr, slice};

use crate::platform::{sys::e, types::*};

#[no_mangle]
pub unsafe extern "C" fn redox_fpath(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    e(syscall::fpath(
        fd as usize,
        slice::from_raw_parts_mut(buf as *mut u8, count),
    )) as ssize_t
}

#[no_mangle]
pub unsafe extern "C" fn redox_physalloc(size: size_t) -> *mut c_void {
    let res = e(syscall::physalloc(size));
    if res == !0 {
        return ptr::null_mut();
    } else {
        return res as *mut c_void;
    }
}

#[no_mangle]
pub unsafe extern "C" fn redox_physfree(physical_address: *mut c_void, size: size_t) -> c_int {
    e(syscall::physfree(physical_address as usize, size)) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn redox_physmap(
    physical_address: *mut c_void,
    size: size_t,
    flags: c_int,
) -> *mut c_void {
    let res = e(syscall::physmap(
        physical_address as usize,
        size,
        syscall::PhysmapFlags::from_bits(flags as usize).expect("physmap: invalid bit pattern"),
    ));
    if res == !0 {
        return ptr::null_mut();
    } else {
        return res as *mut c_void;
    }
}

#[no_mangle]
pub unsafe extern "C" fn redox_physunmap(virtual_address: *mut c_void) -> c_int {
    e(syscall::physunmap(virtual_address as usize)) as c_int
}

pub struct FdGuard {
    fd: usize,
    taken: bool,
}
impl FdGuard {
    pub fn new(fd: usize) -> Self {
        Self {
            fd, taken: false,
        }
    }
    pub fn take(&mut self) -> usize {
        self.taken = true;
        self.fd
    }
}
impl core::ops::Deref for FdGuard {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.fd
    }
}

impl Drop for FdGuard {
    fn drop(&mut self) {
        if !self.taken {
            let _ = syscall::close(self.fd);
        }
    }
}
pub fn create_set_addr_space_buf(space: usize, ip: usize, sp: usize) -> [u8; size_of::<usize>() * 3] {
    let mut buf = [0_u8; 3 * size_of::<usize>()];
    let mut chunks = buf.array_chunks_mut::<{size_of::<usize>()}>();
    *chunks.next().unwrap() = usize::to_ne_bytes(space);
    *chunks.next().unwrap() = usize::to_ne_bytes(sp);
    *chunks.next().unwrap() = usize::to_ne_bytes(ip);
    buf
}
