use core::convert::TryInto;

use crate::{
    error::Errno,
    fs::File,
    header::{errno, fcntl, limits, sys_statvfs, unistd::sysconf::constants::*},
    io::Read,
    out::Out,
    platform::{
        self, Pal, Sys,
        types::{c_int, c_long},
    },
};

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
        _SC_MONOTONIC_CLOCK => 200112,
        _SC_SEMAPHORES => 200112,
        _SC_BARRIERS => 202405,
        _SC_SHELL => 1,
        _SC_SHARED_MEMORY_OBJECTS => 200112,
        _SC_THREADS => 202405,
        _SC_THREAD_ATTR_STACKADDR => 202405,
        _SC_THREAD_ATTR_STACKSIZE => 202405,
        _SC_TIMEOUTS => 202405,
        _SC_TIMERS => 202405,
        _SC_SYMLOOP_MAX => -1,
        _SC_HOST_NAME_MAX => limits::HOST_NAME_MAX.try_into().unwrap_or(-1),
        _SC_NPROCESSORS_CONF => get_cpu_count().unwrap_or(None).unwrap_or(1),
        _SC_NPROCESSORS_ONLN => get_cpu_count().unwrap_or(None).unwrap_or(1),
        _SC_PHYS_PAGES => get_mem_stat().map(|s| s.f_blocks as c_long).unwrap_or(-1),
        _SC_AVPHYS_PAGES => get_mem_stat().map(|s| s.f_bfree as c_long).unwrap_or(-1),
        _SC_SIGQUEUE_MAX => 32,
        _SC_REALTIME_SIGNALS => 202405,
        _ => {
            platform::ERRNO.set(errno::EINVAL);
            -1
        }
    }
}

pub fn get_cpu_count() -> Result<Option<c_long>, Errno> {
    // As long as CPUs: entry is in the first line, this is safe
    let mut buffer = [0u8; 128];
    let mut file = File::open(c"/scheme/sys/cpu".into(), fcntl::O_RDONLY)?;

    let bytes_read = file
        .read(&mut buffer)
        .map_err(|_| Errno(errno::EIO).sync())?;

    let contents =
        str::from_utf8(&buffer[..bytes_read]).map_err(|_| Errno(errno::EINVAL).sync())?;
    Ok(contents
        .lines()
        .find(|line| line.starts_with("CPUs:"))
        .and_then(|line| line.split(':').nth(1))
        .and_then(|num_str| num_str.trim().parse::<c_long>().ok()))
}

pub fn get_mem_stat() -> Result<sys_statvfs::statvfs, Errno> {
    let fd = Sys::open(c"/scheme/memory".into(), fcntl::O_PATH, 0)?;
    let mut buf = sys_statvfs::statvfs::default();
    let res = Sys::fstatvfs(fd, Out::from_mut(&mut buf));
    if let Ok(()) = Sys::close(fd) {}; // TODO handle error
    let _ = res?;
    return Ok(buf);
}
