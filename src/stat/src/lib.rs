//! stat implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/sysstat.h.html

#![no_std]

extern crate platform;

use platform::types::*;

pub const S_IFMT: c_int =   00170000;
pub const S_IFBLK: c_int =   0060000;
pub const S_IFCHR: c_int =   0020000;
pub const S_IFIFO: c_int =   0010000;
pub const S_IFREG: c_int =   0100000;
pub const S_IFDIR: c_int =   0040000;
pub const S_IFLNK: c_int =   0120000;

pub const S_IRWXU: c_int = 00700;
pub const S_IRUSR: c_int = 00400;
pub const S_IWUSR: c_int = 00200;
pub const S_IXUSR: c_int = 00100;

pub const S_IRWXG: c_int = 00070;
pub const S_IRGRP: c_int = 00040;
pub const S_IWGRP: c_int = 00020;
pub const S_IXGRP: c_int = 00010;

pub const S_IRWXO: c_int = 00007;
pub const S_IROTH: c_int = 00004;
pub const S_IWOTH: c_int = 00002;
pub const S_IXOTH: c_int = 00001;
pub const S_ISUID: c_int = 04000;
pub const S_ISGID: c_int = 02000;
pub const S_ISVTX: c_int = 01000;

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
    platform::mkfifo(path, mode)
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
