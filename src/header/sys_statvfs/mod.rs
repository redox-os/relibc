//! `sys_statvfs.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_statvfs.h.html>.

use crate::{
    c_str::CStr,
    error::ResultExt,
    header::fcntl::O_PATH,
    out::Out,
    platform::{Pal, Sys, types::{c_char, c_int, c_ulong, fsblkcnt_t, fsfilcnt_t}},
};

//pub const ST_RDONLY
//pub const ST_NOSUID

#[repr(C)]
#[derive(Default)]
pub struct statvfs {
    pub f_bsize: c_ulong,
    pub f_frsize: c_ulong,
    pub f_blocks: fsblkcnt_t,
    pub f_bfree: fsblkcnt_t,
    pub f_bavail: fsblkcnt_t,
    pub f_files: fsfilcnt_t,
    pub f_ffree: fsfilcnt_t,
    pub f_favail: fsfilcnt_t,
    pub f_fsid: c_ulong,
    pub f_flag: c_ulong,
    pub f_namemax: c_ulong,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fstatvfs.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fstatvfs(fildes: c_int, buf: *mut statvfs) -> c_int {
    let buf = Out::nonnull(buf);
    Sys::fstatvfs(fildes, buf).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fstatvfs.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn statvfs(file: *const c_char, buf: *mut statvfs) -> c_int {
    let file = CStr::from_ptr(file);
    let buf = Out::nonnull(buf);
    // TODO: Rustify
    let fd = Sys::open(file, O_PATH, 0).or_minus_one_errno();
    if fd < 0 {
        return -1;
    }

    let res = Sys::fstatvfs(fd, buf).map(|()| 0).or_minus_one_errno();

    Sys::close(fd);

    res
}
