use core::{ptr, slice};

use crate::platform::{sys::e, types::*};

use syscall::{error::*, F_SETFD, F_SETFL};

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

pub fn pipe2(fds: &mut [c_int], flags: usize) -> syscall::error::Result<()> {
    let fds =
        <&mut [c_int; 2]>::try_from(fds).expect("expected Pal pipe2 to have validated pipe2 array");

    let mut read_fd = FdGuard::new(syscall::open("pipe:", flags)?);
    let mut write_fd = FdGuard::new(syscall::dup(*read_fd, b"write")?);
    syscall::fcntl(*write_fd, F_SETFL, flags)?;
    syscall::fcntl(*write_fd, F_SETFD, flags)?;

    *fds = [
        c_int::try_from(*read_fd).map_err(|_| Error::new(EMFILE))?,
        c_int::try_from(*write_fd).map_err(|_| Error::new(EMFILE))?,
    ];

    read_fd.take();
    write_fd.take();

    Ok(())
}

pub use redox_exec::{create_set_addr_space_buf, FdGuard};
