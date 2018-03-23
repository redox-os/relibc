//! stat implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/sysstat.h.html

#![no_std]

extern crate platform;

use platform::types::*;

#[repr(C)]
pub struct stat {
    pub st_dev: dev_t,
    pub st_ino: ino_t,
    pub st_nlink: nlink_t,
    pub st_mode: mode_t,
    pub st_uid: uid_t,
    pub st_gid: gid_t,
    pub st_rdev: dev_t,
    pub st_size: off_t,
    pub st_blksize: blksize_t,
    pub st_atim: time_t,
    pub st_mtim: time_t,
    pub st_ctim: time_t,
}

#[no_mangle]
pub extern "C" fn chmod(path: *const c_char, mode: mode_t) -> c_int {
    platform::chmod(path, mode)
}

#[no_mangle]
pub extern "C" fn fchmod(fildes: c_int, mode: mode_t) -> c_int {
    platform::fchmod(fildes, mode)
}

#[no_mangle]
pub extern "C" fn fstat(fildes: c_int, buf: *mut stat) -> c_int {
    platform::fstat(fildes, buf)
}

#[no_mangle]
pub extern "C" fn lstat(path: *const c_char, buf: *mut stat) -> c_int {
    platform::lstat(path, buf)
}

#[no_mangle]
pub extern "C" fn mkdir(path: *const c_char, mode: mode_t) -> c_int {
    platform::mkdir(path, mode)
}

#[no_mangle]
pub extern "C" fn mkfifo(path: *const c_char, mode: mode_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mknod(path: *const c_char, mode: mode_t, dev: dev_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn stat(file: *const c_char, buf: *mut stat) -> c_int {
    platform::stat(file, buf)
}

#[no_mangle]
pub extern "C" fn umask(mask: mode_t) -> mode_t {
    unimplemented!();
}

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
