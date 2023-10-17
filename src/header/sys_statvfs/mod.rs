//! statvfs implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/sysstatvfs.h.html

use crate::{
    c_str::CStr,
    errno::IntoPosix,
    header::fcntl::O_PATH,
    platform::{types::*, Pal, Sys},
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

#[no_mangle]
pub extern "C" fn fstatvfs(fildes: c_int, buf: *mut statvfs) -> c_int {
    Sys::fstatvfs(fildes, buf).into_posix_style()
}

#[no_mangle]
pub unsafe extern "C" fn statvfs(file: *const c_char, buf: *mut statvfs) -> c_int {
    let file = CStr::from_ptr(file);
    let fd = Sys::open(file, O_PATH, 0).into_posix_style();
    if fd < 0 {
        return -1;
    }

    let res = Sys::fstatvfs(fd, buf).into_posix_style();

    Sys::close(fd).into_posix_style();

    res
}
