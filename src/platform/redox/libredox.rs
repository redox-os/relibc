use core::{slice, str};

use syscall::{Error, Result, WaitFlags, EMFILE};

use crate::{
    header::signal::sigaction,
    platform::types::{c_int, mode_t},
};

pub type RawResult = usize;

pub fn open(path: &str, oflag: c_int, mode: mode_t) -> Result<usize> {
    let usize_fd = super::path::open(
        path,
        ((oflag as usize) & 0xFFFF_0000) | ((mode as usize) & 0xFFFF),
    )?;

    c_int::try_from(usize_fd)
        .map_err(|_| {
            let _ = syscall::close(usize_fd);
            Error::new(EMFILE)
        })
        .map(|f| f as usize)
}

#[no_mangle]
pub unsafe extern "C" fn redox_open_v1(
    path_base: *const u8,
    path_len: usize,
    flags: u32,
    mode: u16,
) -> RawResult {
    Error::mux(open(
        str::from_utf8_unchecked(slice::from_raw_parts(path_base, path_len)),
        flags as c_int,
        mode as mode_t,
    ))
}

#[no_mangle]
pub unsafe extern "C" fn redox_dup_v1(fd: usize, buf: *const u8, len: usize) -> RawResult {
    Error::mux(syscall::dup(fd, core::slice::from_raw_parts(buf, len)))
}
#[no_mangle]
pub unsafe extern "C" fn redox_dup2_v1(
    old_fd: usize,
    new_fd: usize,
    buf: *const u8,
    len: usize,
) -> RawResult {
    Error::mux(syscall::dup2(
        old_fd,
        new_fd,
        core::slice::from_raw_parts(buf, len),
    ))
}
#[no_mangle]
pub unsafe extern "C" fn redox_read_v1(fd: usize, dst_base: *mut u8, dst_len: usize) -> RawResult {
    Error::mux(syscall::read(
        fd,
        slice::from_raw_parts_mut(dst_base, dst_len),
    ))
}
#[no_mangle]
pub unsafe extern "C" fn redox_write_v1(
    fd: usize,
    src_base: *const u8,
    src_len: usize,
) -> RawResult {
    Error::mux(syscall::write(fd, slice::from_raw_parts(src_base, src_len)))
}
#[no_mangle]
pub unsafe extern "C" fn redox_fsync_v1(fd: usize) -> RawResult {
    Error::mux(syscall::fsync(fd))
}
#[no_mangle]
pub unsafe extern "C" fn redox_fdatasync_v1(fd: usize) -> RawResult {
    // TODO
    Error::mux(syscall::fsync(fd))
}
#[no_mangle]
pub unsafe extern "C" fn redox_fchmod_v1(fd: usize, new_mode: u16) -> RawResult {
    Error::mux(syscall::fchmod(fd, new_mode))
}
#[no_mangle]
pub unsafe extern "C" fn redox_fchown_v1(fd: usize, new_uid: u32, new_gid: u32) -> RawResult {
    Error::mux(syscall::fchown(fd, new_uid, new_gid))
}
#[no_mangle]
pub unsafe extern "C" fn redox_fpath_v1(fd: usize, dst_base: *mut u8, dst_len: usize) -> RawResult {
    Error::mux(syscall::fpath(
        fd,
        core::slice::from_raw_parts_mut(dst_base, dst_len),
    ))
}
#[no_mangle]
pub unsafe extern "C" fn redox_close_v1(fd: usize) -> RawResult {
    Error::mux(syscall::close(fd))
}

#[no_mangle]
pub unsafe extern "C" fn redox_get_pid_v1() -> RawResult {
    Error::mux(syscall::getpid())
}

#[no_mangle]
pub unsafe extern "C" fn redox_get_euid_v1() -> RawResult {
    Error::mux(syscall::geteuid())
}
#[no_mangle]
pub unsafe extern "C" fn redox_get_ruid_v1() -> RawResult {
    Error::mux(syscall::getuid())
}
#[no_mangle]
pub unsafe extern "C" fn redox_get_egid_v1() -> RawResult {
    Error::mux(syscall::getegid())
}
#[no_mangle]
pub unsafe extern "C" fn redox_get_rgid_v1() -> RawResult {
    Error::mux(syscall::getgid())
}
#[no_mangle]
pub unsafe extern "C" fn redox_setrens_v1(rns: usize, ens: usize) -> RawResult {
    Error::mux(syscall::setrens(rns, ens))
}
#[no_mangle]
pub unsafe extern "C" fn redox_waitpid_v1(pid: usize, status: *mut i32, options: u32) -> RawResult {
    let mut sts = 0_usize;
    let res = Error::mux(syscall::waitpid(
        pid,
        &mut sts,
        WaitFlags::from_bits_truncate(options as usize),
    ));
    status.write(sts as i32);
    res
}

#[no_mangle]
pub unsafe extern "C" fn redox_kill_v1(pid: usize, signal: u32) -> RawResult {
    Error::mux(syscall::kill(pid, signal as usize))
}

#[no_mangle]
pub unsafe extern "C" fn redox_sigaction_v1(
    signal: u32,
    new: *const sigaction,
    old: *mut sigaction,
) -> RawResult {
    Error::mux(super::signal::sigaction_impl(signal as i32, new.as_ref(), old.as_mut()).map(|()| 0))
}

#[no_mangle]
pub unsafe extern "C" fn redox_sigprocmask_v1(
    how: u32,
    new: *const u64,
    old: *mut u64,
) -> RawResult {
    Error::mux(super::signal::sigprocmask_impl(how as i32, new, old).map(|()| 0))
}
