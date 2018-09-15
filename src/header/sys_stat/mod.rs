//! stat implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/sysstat.h.html

use c_str::CStr;
use header::fcntl::{O_NOFOLLOW, O_PATH};
use header::time::timespec;
use platform;
use platform::types::*;
use platform::{Pal, Sys};

pub const S_IFMT: c_int = 0o0170000;

pub const S_IFDIR: c_int = 0o040000;
pub const S_IFCHR: c_int = 0o020000;
pub const S_IFBLK: c_int = 0o060000;
pub const S_IFREG: c_int = 0o100000;
pub const S_IFIFO: c_int = 0o010000;
pub const S_IFLNK: c_int = 0o120000;
pub const S_IFSOCK: c_int = 0o140000;

pub const S_IRWXU: c_int = 0o0700;
pub const S_IRUSR: c_int = 0o0400;
pub const S_IWUSR: c_int = 0o0200;
pub const S_IXUSR: c_int = 0o0100;

pub const S_IRWXG: c_int = 0o0070;
pub const S_IRGRP: c_int = 0o0040;
pub const S_IWGRP: c_int = 0o0020;
pub const S_IXGRP: c_int = 0o0010;

pub const S_IRWXO: c_int = 0o0007;
pub const S_IROTH: c_int = 0o0004;
pub const S_IWOTH: c_int = 0o0002;
pub const S_IXOTH: c_int = 0o0001;
pub const S_ISUID: c_int = 0o4000;
pub const S_ISGID: c_int = 0o2000;
pub const S_ISVTX: c_int = 0o1000;

#[repr(C)]
#[derive(Default)]
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
    pub st_blocks: blkcnt_t,

    pub st_atim: timespec,
    pub st_mtim: timespec,
    pub st_ctim: timespec,

    // Compared to glibc, our struct is for some reason 24 bytes too small.
    // Accessing atime works, so clearly the struct isn't incorrect...
    // This works.
    pub _pad: [c_char; 24],
}

#[no_mangle]
pub extern "C" fn chmod(path: *const c_char, mode: mode_t) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    Sys::chmod(path, mode)
}

#[no_mangle]
pub extern "C" fn fchmod(fildes: c_int, mode: mode_t) -> c_int {
    Sys::fchmod(fildes, mode)
}

#[no_mangle]
pub extern "C" fn fstat(fildes: c_int, buf: *mut stat) -> c_int {
    Sys::fstat(fildes, buf)
}

#[no_mangle]
pub extern "C" fn __fxstat(_ver: c_int, fildes: c_int, buf: *mut stat) -> c_int {
    fstat(fildes, buf)
}

#[no_mangle]
pub extern "C" fn futimens(fd: c_int, times: *const timespec) -> c_int {
    Sys::futimens(fd, times)
}

#[no_mangle]
pub extern "C" fn lstat(path: *const c_char, buf: *mut stat) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    let fd = Sys::open(path, O_PATH | O_NOFOLLOW, 0);
    if fd < 0 {
        return -1;
    }

    let res = Sys::fstat(fd, buf);

    Sys::close(fd);

    res
}

#[no_mangle]
pub extern "C" fn mkdir(path: *const c_char, mode: mode_t) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    Sys::mkdir(path, mode)
}

#[no_mangle]
pub extern "C" fn mkfifo(path: *const c_char, mode: mode_t) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    Sys::mkfifo(path, mode)
}

// #[no_mangle]
pub extern "C" fn mknod(path: *const c_char, mode: mode_t, dev: dev_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn stat(file: *const c_char, buf: *mut stat) -> c_int {
    let file = unsafe { CStr::from_ptr(file) };
    let fd = Sys::open(file, O_PATH, 0);
    if fd < 0 {
        return -1;
    }

    let res = Sys::fstat(fd, buf);

    Sys::close(fd);

    res
}

// #[no_mangle]
// pub extern "C" fn umask(mask: mode_t) -> mode_t {
//     Sys::umask(mask)
// }
