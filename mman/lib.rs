#[no_mangle]
pub extern "C" fn mlock(addr: *const libc::c_void, len: usize) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mlockall(flags: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mmap(
    addr: *mut libc::c_void,
    len: usize,
    prot: libc::c_int,
    flags: libc::c_int,
    fildes: libc::c_int,
    off: off_t,
) -> *mut libc::c_void {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mprotect(addr: *mut libc::c_void, len: usize, prot: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn msync(addr: *mut libc::c_void, len: usize, flags: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn munlock(addr: *const libc::c_void, len: usize) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn munlockall() -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn munmap(addr: *mut libc::c_void, len: usize) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn shm_open(
    name: *const libc::c_char,
    oflag: libc::c_int,
    mode: mode_t,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn shm_unlink(name: *const libc::c_char) -> libc::c_int {
    unimplemented!();
}
