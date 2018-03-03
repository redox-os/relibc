//! fcntl implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/fcntl.h.html

#![no_std]

extern crate libc;

use libc::*;

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

pub const O_RDONLY: c_int =     0x0001_0000;
pub const O_WRONLY: c_int =     0x0002_0000;
pub const O_RDWR: c_int =       0x0003_0000;
pub const O_NONBLOCK: c_int =   0x0004_0000;
pub const O_APPEND: c_int =     0x0008_0000;
pub const O_SHLOCK: c_int =     0x0010_0000;
pub const O_EXLOCK: c_int =     0x0020_0000;
pub const O_ASYNC: c_int =      0x0040_0000;
pub const O_FSYNC: c_int =      0x0080_0000;
pub const O_CLOEXEC: c_int =    0x0100_0000;
pub const O_CREAT: c_int =      0x0200_0000;
pub const O_TRUNC: c_int =      0x0400_0000;
pub const O_EXCL: c_int =       0x0800_0000;
pub const O_DIRECTORY: c_int =  0x1000_0000;
pub const O_STAT: c_int =       0x2000_0000;
pub const O_SYMLINK: c_int =    0x4000_0000;
pub const O_NOFOLLOW: c_int =   0x8000_0000;
pub const O_ACCMODE: c_int = O_RDONLY | O_WRONLY | O_RDWR;

#[no_mangle]
pub extern "C" fn creat(path: *const c_char, mode: mode_t) -> c_int {
    open(path, O_WRONLY | O_CREAT | O_TRUNC, mode)
}

#[no_mangle]
pub extern "C" fn fcntl(fildes: c_int, cmd: c_int, arg: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn open(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    unimplemented!();
}

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
