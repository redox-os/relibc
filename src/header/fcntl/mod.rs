//! fcntl implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/fcntl.h.html

use crate::{
    c_str::CStr,
    platform::{types::*, Pal, Sys},
};

pub use self::sys::*;

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

pub const F_RDLCK: c_int = 0;
pub const F_WRLCK: c_int = 1;
pub const F_UNLCK: c_int = 2;

pub const F_ULOCK: c_int = 0;
pub const F_LOCK: c_int = 1;
pub const F_TLOCK: c_int = 2;
pub const F_TEST: c_int = 3;

#[no_mangle]
pub unsafe extern "C" fn creat(path: *const c_char, mode: mode_t) -> c_int {
    open(path, O_WRONLY | O_CREAT | O_TRUNC, mode)
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct flock {
    pub l_type: c_short,
    pub l_whence: c_short,
    pub l_start: off_t,
    pub l_len: off_t,
    pub l_pid: pid_t,
}
#[no_mangle]
pub unsafe extern "C" fn fcntl(fildes: c_int, cmd: c_int, mut __valist: ...) -> c_int {
    // c_ulonglong
    let arg = match cmd {
        F_DUPFD | F_SETFD | F_SETFL | F_SETLK | F_SETLKW | F_GETLK => __valist.arg::<c_ulonglong>(),
        _ => 0,
    };

    Sys::fcntl(fildes, cmd, arg)
}

#[no_mangle]
pub unsafe extern "C" fn open(path: *const c_char, oflag: c_int, mut __valist: ...) -> c_int {
    let mode = if oflag & O_CREAT == O_CREAT
    /* || oflag & O_TMPFILE == O_TMPFILE */
    {
        __valist.arg::<mode_t>()
    } else {
        0
    };

    let path = CStr::from_ptr(path);
    Sys::open(path, oflag, mode)
}

#[no_mangle]
pub unsafe extern "C" fn cbindgen_stupid_struct_user_for_fcntl(a: flock) {}
