use platform;
use platform::{Pal, Sys};
use platform::types::*;

pub use sys::*;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

// #[no_mangle]
pub extern "C" fn mlock(addr: *const c_void, len: usize) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn mlockall(flags: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn mmap(
    addr: *mut c_void,
    len: usize,
    prot: c_int,
    flags: c_int,
    fildes: c_int,
    off: off_t,
) -> *mut c_void {
    Sys::mmap(addr, len, prot, flags, fildes, off)
}

// #[no_mangle]
pub extern "C" fn mprotect(addr: *mut c_void, len: usize, prot: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn msync(addr: *mut c_void, len: usize, flags: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn munlock(addr: *const c_void, len: usize) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn munlockall() -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn munmap(addr: *mut c_void, len: usize) -> c_int {
    Sys::munmap(addr, len)
}

// #[no_mangle]
pub extern "C" fn shm_open(name: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn shm_unlink(name: *const c_char) -> c_int {
    unimplemented!();
}
