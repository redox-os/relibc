use core::convert::TryInto;

use alloc::string::String;

use crate::{
    error::Errno,
    fs::File,
    header::{errno, fcntl, limits},
    io::Read,
    platform::{self, Pal, Sys, types::*},
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

pub const _SC_NPROCESSORS_CONF: c_int = 57;
pub const _SC_NPROCESSORS_ONLN: c_int = 58;
pub const _SC_PHYS_PAGES: c_int = 59;
pub const _SC_AVPHYS_PAGES: c_int = 60;

// ...
pub const _SC_GETGR_R_SIZE_MAX: c_int = 69;
pub const _SC_GETPW_R_SIZE_MAX: c_int = 70;
pub const _SC_LOGIN_NAME_MAX: c_int = 71;
pub const _SC_TTY_NAME_MAX: c_int = 72;
// ...
pub const _SC_SYMLOOP_MAX: c_int = 173;
// ...
pub const _SC_HOST_NAME_MAX: c_int = 180;
// ...
pub const _SC_SIGQUEUE_MAX: c_int = 190;
pub const _SC_REALTIME_SIGNALS: c_int = 191;
// } POSIX.1

pub(super) fn sysconf_impl(name: c_int) -> c_long {
    //TODO: Real values
    match name {
        _SC_ARG_MAX => 4096,
        _SC_CHILD_MAX => 65536,
        _SC_CLK_TCK => 100,
        _SC_NGROUPS_MAX => limits::NGROUPS_MAX as c_long,
        _SC_OPEN_MAX => 1024,
        _SC_STREAM_MAX => 16,
        _SC_TZNAME_MAX => -1,
        _SC_VERSION => 200809,
        _SC_PAGESIZE => Sys::getpagesize().try_into().unwrap_or(-1),
        _SC_RE_DUP_MAX => 32767,
        _SC_GETGR_R_SIZE_MAX => -1,
        _SC_GETPW_R_SIZE_MAX => -1,
        _SC_LOGIN_NAME_MAX => 256,
        _SC_TTY_NAME_MAX => 32,
        _SC_SYMLOOP_MAX => -1,
        _SC_HOST_NAME_MAX => 64,
        _SC_NPROCESSORS_CONF => get_cpu_count().unwrap_or(None).unwrap_or(1),
        _SC_NPROCESSORS_ONLN => get_cpu_count().unwrap_or(None).unwrap_or(1),
        _SC_PHYS_PAGES => 262144,
        _SC_AVPHYS_PAGES => -1,
        _SC_SIGQUEUE_MAX => 32,
        _SC_REALTIME_SIGNALS => 202405,
        _ => {
            platform::ERRNO.set(errno::EINVAL);
            -1
        }
    }
}

pub fn get_cpu_count() -> Result<Option<c_long>, Errno> {
    let mut string = String::new();
    let mut file = File::open(c"/scheme/sys/cpu".into(), fcntl::O_RDONLY)?;
    file.read_to_string(&mut string)
        .map_err(|_| Errno(errno::EIO).sync())?;

    Ok(string
        .lines()
        .find(|line| line.starts_with("CPUs:"))
        .and_then(|line| line.split(':').nth(1))
        .and_then(|num_str| num_str.trim().parse::<c_long>().ok()))
}
