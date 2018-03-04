#![no_std]

extern crate platform;

use platform::types::*;

#[no_mangle]
pub extern "C" fn mlock(addr: *const c_void, len: usize) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mlockall(flags: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mmap(
    addr: *mut c_void,
    len: usize,
    prot: c_int,
    flags: c_int,
    fildes: c_int,
    off: off_t,
) -> *mut c_void {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mprotect(addr: *mut c_void, len: usize, prot: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn msync(addr: *mut c_void, len: usize, flags: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn munlock(addr: *const c_void, len: usize) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn munlockall() -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn munmap(addr: *mut c_void, len: usize) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn shm_open(
    name: *const c_char,
    oflag: c_int,
    mode: mode_t,
) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn shm_unlink(name: *const c_char) -> c_int {
    unimplemented!();
}
