//! stat implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/sysstat.h.html

use crate::{
    c_str::CStr,
    error::ResultExt,
    header::{
        fcntl::{O_NOFOLLOW, O_PATH},
        time::timespec,
    },
    out::Out,
    platform::{types::*, Pal, Sys},
};

pub const S_IFMT: c_int = 0o0_170_000;

pub const S_IFDIR: c_int = 0o040_000;
pub const S_IFCHR: c_int = 0o020_000;
pub const S_IFBLK: c_int = 0o060_000;
pub const S_IFREG: c_int = 0o100_000;
pub const S_IFIFO: c_int = 0o010_000;
pub const S_IFLNK: c_int = 0o120_000;
pub const S_IFSOCK: c_int = 0o140_000;

pub const S_IRWXU: c_int = 0o0_700;
pub const S_IRUSR: c_int = 0o0_400;
pub const S_IWUSR: c_int = 0o0_200;
pub const S_IXUSR: c_int = 0o0_100;

// Defined for compatibility
pub const S_IREAD: c_int = S_IRUSR;
pub const S_IWRITE: c_int = S_IWUSR;
pub const S_IEXEC: c_int = S_IXUSR;

pub const S_IRWXG: c_int = 0o0_070;
pub const S_IRGRP: c_int = 0o0_040;
pub const S_IWGRP: c_int = 0o0_020;
pub const S_IXGRP: c_int = 0o0_010;

pub const S_IRWXO: c_int = 0o0_007;
pub const S_IROTH: c_int = 0o0_004;
pub const S_IWOTH: c_int = 0o0_002;
pub const S_IXOTH: c_int = 0o0_001;
pub const S_ISUID: c_int = 0o4_000;
pub const S_ISGID: c_int = 0o2_000;
pub const S_ISVTX: c_int = 0o1_000;

pub const UTIME_NOW: useconds_t = (1 << 30) - 1;
pub const UTIME_OMIT: useconds_t = (1 << 30) - 2;

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
pub unsafe extern "C" fn chmod(path: *const c_char, mode: mode_t) -> c_int {
    let path = CStr::from_ptr(path);
    Sys::chmod(path, mode).map(|()| 0).or_minus_one_errno()
}

#[no_mangle]
pub extern "C" fn fchmod(fildes: c_int, mode: mode_t) -> c_int {
    Sys::fchmod(fildes, mode).map(|()| 0).or_minus_one_errno()
}

#[no_mangle]
pub unsafe extern "C" fn fstat(fildes: c_int, buf: *mut stat) -> c_int {
    let buf = Out::nonnull(buf);
    Sys::fstat(fildes, buf).map(|()| 0).or_minus_one_errno()
}

#[no_mangle]
pub unsafe extern "C" fn fstatat(
    fildes: c_int,
    path: *const c_char,
    buf: *mut stat,
    flags: c_int,
) -> c_int {
    let path = CStr::from_nullable_ptr(path);
    let buf = Out::nonnull(buf);
    Sys::fstatat(fildes, path, buf, flags)
        .map(|()| 0)
        .or_minus_one_errno()
}

#[no_mangle]
pub unsafe extern "C" fn __fxstat(_ver: c_int, fildes: c_int, buf: *mut stat) -> c_int {
    fstat(fildes, buf)
}

#[no_mangle]
pub unsafe extern "C" fn futimens(fd: c_int, times: *const timespec) -> c_int {
    Sys::futimens(fd, times).map(|()| 0).or_minus_one_errno()
}

#[no_mangle]
pub unsafe extern "C" fn lstat(path: *const c_char, buf: *mut stat) -> c_int {
    let path = CStr::from_ptr(path);
    let buf = Out::nonnull(buf);

    // TODO: Rustify
    let fd = Sys::open(path, O_PATH | O_NOFOLLOW, 0).or_minus_one_errno();
    if fd < 0 {
        return -1;
    }

    // TODO: Rustify
    let res = Sys::fstat(fd, buf).map(|()| 0).or_minus_one_errno();

    Sys::close(fd);

    res
}

#[no_mangle]
pub unsafe extern "C" fn mkdir(path: *const c_char, mode: mode_t) -> c_int {
    let path = CStr::from_ptr(path);
    Sys::mkdir(path, mode).map(|()| 0).or_minus_one_errno()
}

#[no_mangle]
pub unsafe extern "C" fn mkfifo(path: *const c_char, mode: mode_t) -> c_int {
    let path = CStr::from_ptr(path);
    Sys::mkfifo(path, mode).map(|()| 0).or_minus_one_errno()
}

#[no_mangle]
pub unsafe extern "C" fn mknod(path: *const c_char, mode: mode_t, dev: dev_t) -> c_int {
    let path = CStr::from_ptr(path);
    Sys::mknod(path, mode, dev).map(|()| 0).or_minus_one_errno()
}

#[no_mangle]
pub unsafe extern "C" fn mknodat(
    dirfd: c_int,
    path: *const c_char,
    mode: mode_t,
    dev: dev_t,
) -> c_int {
    let path = CStr::from_ptr(path);
    Sys::mknodat(dirfd, path, mode, dev)
        .map(|()| 0)
        .or_minus_one_errno()
}

#[no_mangle]
pub unsafe extern "C" fn stat(file: *const c_char, buf: *mut stat) -> c_int {
    let file = CStr::from_ptr(file);
    let buf = Out::nonnull(buf);

    // TODO: Rustify
    let fd = Sys::open(file, O_PATH, 0).or_minus_one_errno();
    if fd < 0 {
        return -1;
    }

    // TODO: Rustify
    let res = Sys::fstat(fd, buf).map(|()| 0).or_minus_one_errno();

    Sys::close(fd);

    res
}

#[no_mangle]
pub extern "C" fn umask(mask: mode_t) -> mode_t {
    Sys::umask(mask)
}
