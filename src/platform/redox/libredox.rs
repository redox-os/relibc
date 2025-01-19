use core::{slice, str};

use redox_rt::{
    protocol::WaitFlags,
    sys::{posix_read, posix_write, WaitpidTarget},
};
use syscall::{Error, Result, EMFILE};

use crate::{
    header::{
        errno::EINVAL,
        signal::{sigaction, SIG_BLOCK, SIG_SETMASK, SIG_UNBLOCK},
        sys_stat::UTIME_NOW,
        sys_uio::iovec,
        time::timespec,
    },
    platform::{types::*, PalSignal},
};

use super::Sys;

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

pub unsafe fn fstat(fd: usize, buf: *mut crate::header::sys_stat::stat) -> syscall::Result<()> {
    let mut redox_buf: syscall::Stat = Default::default();
    syscall::fstat(fd, &mut redox_buf)?;

    if let Some(buf) = buf.as_mut() {
        buf.st_dev = redox_buf.st_dev as dev_t;
        buf.st_ino = redox_buf.st_ino as ino_t;
        buf.st_nlink = redox_buf.st_nlink as nlink_t;
        buf.st_mode = redox_buf.st_mode as mode_t;
        buf.st_uid = redox_buf.st_uid as uid_t;
        buf.st_gid = redox_buf.st_gid as gid_t;
        // TODO st_rdev
        buf.st_rdev = 0;
        buf.st_size = redox_buf.st_size as off_t;
        buf.st_blksize = redox_buf.st_blksize as blksize_t;
        buf.st_atim = timespec {
            tv_sec: redox_buf.st_atime as time_t,
            tv_nsec: redox_buf.st_atime_nsec as c_long,
        };
        buf.st_mtim = timespec {
            tv_sec: redox_buf.st_mtime as time_t,
            tv_nsec: redox_buf.st_mtime_nsec as c_long,
        };
        buf.st_ctim = timespec {
            tv_sec: redox_buf.st_ctime as time_t,
            tv_nsec: redox_buf.st_ctime_nsec as c_long,
        };
    }
    Ok(())
}
pub unsafe fn fstatvfs(
    fd: usize,
    buf: *mut crate::header::sys_statvfs::statvfs,
) -> syscall::Result<()> {
    let mut kbuf: syscall::StatVfs = Default::default();
    syscall::fstatvfs(fd, &mut kbuf)?;

    if !buf.is_null() {
        (*buf).f_bsize = kbuf.f_bsize as c_ulong;
        (*buf).f_frsize = kbuf.f_bsize as c_ulong;
        (*buf).f_blocks = kbuf.f_blocks as c_ulong;
        (*buf).f_bfree = kbuf.f_bfree as c_ulong;
        (*buf).f_bavail = kbuf.f_bavail as c_ulong;
        //TODO
        (*buf).f_files = 0;
        (*buf).f_ffree = 0;
        (*buf).f_favail = 0;
        (*buf).f_fsid = 0;
        (*buf).f_flag = 0;
        (*buf).f_namemax = 0;
    }
    Ok(())
}
pub unsafe fn futimens(fd: usize, times: *const timespec) -> syscall::Result<()> {
    let times = if times.is_null() {
        // null means set to current time using special UTIME_NOW value (tv_sec is ignored in that case)
        [
            syscall::TimeSpec {
                tv_sec: 0,
                tv_nsec: UTIME_NOW as c_int,
            },
            syscall::TimeSpec {
                tv_sec: 0,
                tv_nsec: UTIME_NOW as c_int,
            },
        ]
    } else {
        times
            .cast::<[timespec; 2]>()
            .read()
            .map(|ts| syscall::TimeSpec::from(&ts))
    };
    syscall::futimens(fd as usize, &times)?;
    Ok(())
}
pub unsafe fn clock_gettime(clock: usize, tp: *mut timespec) -> syscall::Result<()> {
    let mut redox_tp = syscall::TimeSpec::from(&*tp);
    syscall::clock_gettime(clock as usize, &mut redox_tp)?;
    (*tp).tv_sec = redox_tp.tv_sec as time_t;
    (*tp).tv_nsec = redox_tp.tv_nsec as c_long;
    Ok(())
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
pub unsafe extern "C" fn redox_openat_v1(
    fd: usize,
    path_base: *const u8,
    path_len: usize,
    flags: u32,
) -> RawResult {
    Error::mux(syscall::openat(
        fd,
        str::from_utf8_unchecked(slice::from_raw_parts(path_base, path_len)),
        flags as usize,
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
    Error::mux(posix_read(fd, slice::from_raw_parts_mut(dst_base, dst_len)))
}
#[no_mangle]
pub unsafe extern "C" fn redox_write_v1(
    fd: usize,
    src_base: *const u8,
    src_len: usize,
) -> RawResult {
    Error::mux(posix_write(fd, slice::from_raw_parts(src_base, src_len)))
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
pub unsafe extern "C" fn redox_fstat_v1(
    fd: usize,
    stat: *mut crate::header::sys_stat::stat,
) -> RawResult {
    Error::mux(fstat(fd, stat).map(|()| 0))
}
#[no_mangle]
pub unsafe extern "C" fn redox_fstatvfs_v1(
    fd: usize,
    stat: *mut crate::header::sys_statvfs::statvfs,
) -> RawResult {
    Error::mux(fstatvfs(fd, stat).map(|()| 0))
}
#[no_mangle]
pub unsafe extern "C" fn redox_futimens_v1(fd: usize, times: *const timespec) -> RawResult {
    Error::mux(futimens(fd, times).map(|()| 0))
}
#[no_mangle]
pub unsafe extern "C" fn redox_close_v1(fd: usize) -> RawResult {
    Error::mux(syscall::close(fd))
}

#[no_mangle]
pub unsafe extern "C" fn redox_get_pid_v1() -> RawResult {
    redox_rt::sys::posix_getpid() as _
}

#[no_mangle]
pub unsafe extern "C" fn redox_get_euid_v1() -> RawResult {
    redox_rt::sys::posix_geteuid() as _
}
#[no_mangle]
pub unsafe extern "C" fn redox_get_ruid_v1() -> RawResult {
    redox_rt::sys::posix_getruid() as _
}
#[no_mangle]
pub unsafe extern "C" fn redox_get_egid_v1() -> RawResult {
    redox_rt::sys::posix_getegid() as _
}
#[no_mangle]
pub unsafe extern "C" fn redox_get_rgid_v1() -> RawResult {
    redox_rt::sys::posix_getrgid() as _
}
#[no_mangle]
pub unsafe extern "C" fn redox_setrens_v1(rns: usize, ens: usize) -> RawResult {
    Error::mux(redox_rt::sys::setrens(rns, ens).map(|()| 0))
}
#[no_mangle]
pub unsafe extern "C" fn redox_waitpid_v1(pid: usize, status: *mut i32, options: u32) -> RawResult {
    let mut sts = 0_usize;
    let res = Error::mux(redox_rt::sys::sys_waitpid(
        WaitpidTarget::from_posix_arg(pid as isize),
        &mut sts,
        WaitFlags::from_bits_truncate(options as usize),
    ));
    status.write(sts as i32);
    res
}

#[no_mangle]
pub unsafe extern "C" fn redox_kill_v1(pid: usize, signal: u32) -> RawResult {
    Error::mux(redox_rt::sys::posix_kill(pid, signal as usize).map(|()| 0))
}

#[no_mangle]
pub unsafe extern "C" fn redox_sigaction_v1(
    signal: u32,
    new: *const sigaction,
    old: *mut sigaction,
) -> RawResult {
    Error::mux(
        Sys::sigaction(signal as c_int, new.as_ref(), old.as_mut())
            .map(|()| 0)
            .map_err(Into::into),
    )
}

#[no_mangle]
pub unsafe extern "C" fn redox_sigprocmask_v1(
    how: u32,
    new: *const u64,
    old: *mut u64,
) -> RawResult {
    Error::mux(
        Sys::sigprocmask(how as c_int, new.as_ref(), old.as_mut())
            .map(|()| 0)
            .map_err(Into::into),
    )
}
#[no_mangle]
pub unsafe extern "C" fn redox_mmap_v1(
    addr: *mut (),
    unaligned_len: usize,
    prot: u32,
    flags: u32,
    fd: usize,
    offset: u64,
) -> RawResult {
    Error::mux(syscall::fmap(
        fd,
        &syscall::Map {
            address: addr as usize,
            offset: offset as usize,
            size: unaligned_len,
            flags: syscall::MapFlags::from_bits_truncate(
                ((prot << 16) | (flags & 0xffff)) as usize,
            ),
        },
    ))
}
#[no_mangle]
pub unsafe extern "C" fn redox_munmap_v1(addr: *mut (), unaligned_len: usize) -> RawResult {
    Error::mux(syscall::funmap(addr as usize, unaligned_len))
}

#[no_mangle]
pub unsafe extern "C" fn redox_clock_gettime_v1(clock: usize, ts: *mut timespec) -> RawResult {
    Error::mux(clock_gettime(clock, ts).map(|()| 0))
}

#[no_mangle]
pub unsafe extern "C" fn redox_strerror_v1(
    buf: *mut u8,
    buflen: *mut usize,
    error: u32,
) -> RawResult {
    let dst = core::slice::from_raw_parts_mut(buf, buflen.read());

    Error::mux((|| {
        // TODO: Merge syscall::error::STR_ERROR into crate::header::error::?

        let src = syscall::error::STR_ERROR
            .get(error as usize)
            .ok_or(Error::new(EINVAL))?;

        // This API ensures that the returned buffer is proper UTF-8. Thus, it returns both the
        // copied length and the actual length.

        buflen.write(src.len());

        let raw_len = core::cmp::min(dst.len(), src.len());
        let len = match core::str::from_utf8(&src.as_bytes()[..raw_len]) {
            Ok(_valid) => raw_len,
            Err(error) => error.valid_up_to(),
        };

        dst[..len].copy_from_slice(&src.as_bytes()[..len]);
        Ok(len)
    })())
}

#[no_mangle]
pub unsafe extern "C" fn redox_mkns_v1(
    names: *const iovec,
    num_names: usize,
    flags: u32,
) -> RawResult {
    Error::mux((|| {
        if flags != 0 {
            return Err(Error::new(EINVAL));
        }
        // Kernel does the UTF-8 validation.
        syscall::mkns(core::slice::from_raw_parts(names.cast(), num_names))
    })())
}
