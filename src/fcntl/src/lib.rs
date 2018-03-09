//! fcntl implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/fcntl.h.html

#![no_std]

extern crate platform;

use platform::types::*;

pub use sys::*;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

pub const F_DUPFD: c_int = 0;
pub const F_GETFD: c_int = 1;
pub const F_SETFD: c_int = 2;
pub const F_GETFL: c_int = 3;
pub const F_SETFL: c_int = 4;
pub const F_GETLK: c_int = 5;
pub const F_SETLK: c_int = 6;
pub const F_SETLKW: c_int = 7;

pub const FD_CLOEXEC: c_int = 0x0100_0000;

pub const F_RDLCK: c_int = 0;
pub const F_WRLCK: c_int = 1;
pub const F_UNLCK: c_int = 2;

#[no_mangle]
pub extern "C" fn creat(path: *const c_char, mode: mode_t) -> c_int {
    sys_open(path, O_WRONLY | O_CREAT | O_TRUNC, mode)
}

#[no_mangle]
pub extern "C" fn fcntl(fildes: c_int, cmd: c_int, arg: c_int) -> c_int {
    platform::fcntl(fildes, cmd, arg)
}

#[no_mangle]
pub extern "C" fn sys_open(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    platform::open(path, oflag, mode)
}

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
