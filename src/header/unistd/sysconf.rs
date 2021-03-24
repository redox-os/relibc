use core::convert::TryInto;

use crate::{
    header::errno,
    platform::{self, types::*, Pal, Sys},
};

// POSIX.1 {
pub const _SC_ARG_MAX: c_int = 0;
pub const _SC_CHILD_MAX: c_int = 1;
pub const _SC_CLK_TCK: c_int = 2;
pub const _SC_NGROUPS_MAX: c_int = 3;
pub const _SC_OPEN_MAX: c_int = 4;
pub const _SC_STREAM_MAX: c_int = 5;
pub const _SC_TZNAME_MAX: c_int = 6;
// ...
pub const _SC_VERSION: c_int = 29;
pub const _SC_PAGESIZE: c_int = 30;
pub const _SC_PAGE_SIZE: c_int = 30;
// ...
pub const _SC_RE_DUP_MAX: c_int = 44;
// ...
pub const _SC_GETPW_R_SIZE_MAX: c_int = 70;
pub const _SC_LOGIN_NAME_MAX: c_int = 71;
pub const _SC_TTY_NAME_MAX: c_int = 72;
// ...
pub const _SC_SYMLOOP_MAX: c_int = 173;
// ...
pub const _SC_HOST_NAME_MAX: c_int = 180;
// } POSIX.1

#[no_mangle]
pub extern "C" fn sysconf(name: c_int) -> c_long {
    //TODO: Real values
    match name {
        _SC_ARG_MAX => 4096,
        _SC_CHILD_MAX => 65536,
        _SC_CLK_TCK => 100,
        _SC_NGROUPS_MAX => 65536,
        _SC_OPEN_MAX => 1024,
        _SC_STREAM_MAX => 16,
        _SC_TZNAME_MAX => -1,
        _SC_VERSION => 200809,
        _SC_PAGESIZE => Sys::getpagesize()
            .try_into()
            .expect("page size not representable as type `long`"),
        _SC_RE_DUP_MAX => 32767,
        _SC_GETPW_R_SIZE_MAX => -1,
        _SC_LOGIN_NAME_MAX => 256,
        _SC_TTY_NAME_MAX => 32,
        _SC_SYMLOOP_MAX => -1,
        _SC_HOST_NAME_MAX => 64,
        _ => {
            unsafe {
                platform::errno = errno::EINVAL;
            }
            -1
        }
    }
}
