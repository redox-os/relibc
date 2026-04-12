use core::{mem, slice, str};

use alloc::vec::Vec;
use ioslice::IoSlice;
use redox_protocols::protocol::{ProcKillTarget, SocketCall, WaitFlags};
use redox_rt::sys::{WaitpidTarget, posix_read, posix_write, std_fs_call_ro, std_fs_call_wo};
use syscall::{
    EMFILE, ENAMETOOLONG, ENOSYS, EOPNOTSUPP, Error, Result, StdFsCallKind,
    data::StdFsCallMeta,
    dirent::{DirentHeader, DirentKind},
};

use crate::{
    header::{
        bits_iovec::iovec, bits_timespec::timespec, errno::EINVAL, signal::sigaction,
        sys_stat::UTIME_NOW,
    },
    out::Out,
    platform::{PalSignal, pal::Pal, types::*},
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

pub fn openat(dirfd: c_int, path: &str, oflag: c_int, mode: mode_t) -> Result<usize> {
    let usize_fd = super::path::openat(
        dirfd,
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
pub fn fchmod(fd: usize, new_mode: u16) -> Result<()> {
    std_fs_call_wo(
        fd,
        &[],
        &StdFsCallMeta::new(StdFsCallKind::Fchmod, new_mode as u64, 0),
    )?;
    Ok(())
}
pub fn fchown(fd: usize, new_uid: u32, new_gid: u32) -> Result<()> {
    /* std_fs_call
    Error::mux(std_fs_call_wo(
        fd,
        &[],
        &StdFsCallMeta::new(
            StdFsCallKind::Fchmod,
            (new_uid as u64) | ((new_gid as u64) << 32),
            0,
        ),
    ))
    */
    syscall::fchown(fd, new_uid, new_gid)?;
    Ok(())
}
pub fn getdents(fd: usize, buf: &mut [u8], opaque: u64) -> Result<usize> {
    //println!("GETDENTS {} into ({:p}+{})", fd, buf.as_ptr(), buf.len());

    const HEADER_SIZE: usize = mem::size_of::<DirentHeader>();

    // Use syscall if it exists.
    match std_fs_call_ro(
        fd,
        buf,
        &StdFsCallMeta::new(StdFsCallKind::Getdents, opaque, HEADER_SIZE as u64),
    ) {
        Err(Error {
            errno: EOPNOTSUPP | ENOSYS,
        }) => (),
        other => {
            //println!("REAL GETDENTS {:?}", other);
            return Ok(other?);
        }
    }

    // Otherwise, for legacy schemes, assume the buffer is pre-arranged (all schemes do this in
    // practice), and just read the name. If multiple names appear, pretend it didn't happen
    // and just use the first entry.

    let (header, name) = buf.split_at_mut(mem::size_of::<DirentHeader>());

    let bytes_read = Sys::pread(fd as c_int, name, opaque as i64)? as usize;
    if bytes_read == 0 {
        return Ok(0);
    }

    let (name_len, advance) = match name[..bytes_read].iter().position(|c| *c == b'\n') {
        Some(idx) => (idx, idx + 1),

        // Insufficient space for NUL byte, or entire entry was not read. Indicate we need a
        // larger buffer.
        None if bytes_read == name.len() => return Err(Error::new(EINVAL)),

        None => (bytes_read, name.len()),
    };
    name[name_len] = b'\0';

    let record_len = u16::try_from(mem::size_of::<DirentHeader>() + name_len + 1)
        .map_err(|_| Error::new(ENAMETOOLONG))?;
    header.copy_from_slice(&DirentHeader {
        inode: 0,
        next_opaque_id: opaque + advance as u64,
        record_len,
        kind: DirentKind::Unspecified as u8,
    });
    //println!("EMULATED GETDENTS");

    Ok(record_len.into())
}
pub unsafe fn fstat(fd: usize, buf: *mut crate::header::sys_stat::stat) -> Result<()> {
    let mut redox_buf: syscall::Stat = Default::default();
    redox_rt::sys::fstat(fd, &mut redox_buf)?;

    if let Some(buf) = unsafe { buf.as_mut() } {
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
        buf.st_blocks = redox_buf.st_blocks as blkcnt_t;
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
pub unsafe fn fstatvfs(fd: usize, buf: *mut crate::header::sys_statvfs::statvfs) -> Result<()> {
    let mut kbuf: syscall::StatVfs = Default::default();
    std_fs_call_ro(
        fd,
        &mut kbuf,
        &StdFsCallMeta::new(StdFsCallKind::Fstatvfs, 0, 0),
    )?;

    if !buf.is_null() {
        unsafe {
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
    }
    Ok(())
}
pub fn fsync(fd: usize) -> Result<()> {
    std_fs_call_wo(fd, &[], &StdFsCallMeta::new(StdFsCallKind::Fsync, 0, 0))?;
    Ok(())
}
pub fn ftruncate(fd: usize, len: usize) -> Result<()> {
    std_fs_call_wo(
        fd,
        &[],
        &StdFsCallMeta::new(StdFsCallKind::Ftruncate, len as u64, 0),
    )?;
    Ok(())
}
pub unsafe fn futimens(fd: usize, times: *const timespec) -> Result<()> {
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
        unsafe { times.cast::<[timespec; 2]>().read() }.map(|ts| syscall::TimeSpec::from(&ts))
    };
    let redox_buf = unsafe {
        slice::from_raw_parts(
            times.as_ptr() as *const u8,
            times.len() * mem::size_of::<syscall::TimeSpec>(),
        )
    };
    std_fs_call_wo(
        fd,
        redox_buf,
        &StdFsCallMeta::new(StdFsCallKind::Futimens, 0, 0),
    )?;
    Ok(())
}
pub fn clock_gettime(clock: usize, mut tp: Out<timespec>) -> Result<()> {
    let mut redox_tp = syscall::TimeSpec::default();
    syscall::clock_gettime(clock as usize, &mut redox_tp)?;
    tp.write(timespec {
        tv_sec: redox_tp.tv_sec as time_t,
        tv_nsec: redox_tp.tv_nsec as c_long,
    });
    Ok(())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_open_v1(
    path_base: *const u8,
    path_len: usize,
    flags: u32,
    mode: u16,
) -> RawResult {
    Error::mux(open(
        unsafe { str::from_utf8_unchecked(slice::from_raw_parts(path_base, path_len)) },
        flags as c_int,
        mode as mode_t,
    ))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_openat_v1(
    fd: usize,
    path_base: *const u8,
    path_len: usize,
    flags: u32,
    fcntl_flags: u32,
) -> RawResult {
    Error::mux(syscall::openat(
        fd,
        unsafe { str::from_utf8_unchecked(slice::from_raw_parts(path_base, path_len)) },
        flags as usize,
        fcntl_flags as usize,
    ))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_dup_v1(fd: usize, buf: *const u8, len: usize) -> RawResult {
    Error::mux(syscall::dup(fd, unsafe {
        core::slice::from_raw_parts(buf, len)
    }))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_dup2_v1(
    old_fd: usize,
    new_fd: usize,
    buf: *const u8,
    len: usize,
) -> RawResult {
    Error::mux(syscall::dup2(old_fd, new_fd, unsafe {
        core::slice::from_raw_parts(buf, len)
    }))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_read_v1(fd: usize, dst_base: *mut u8, dst_len: usize) -> RawResult {
    Error::mux(posix_read(fd, unsafe {
        slice::from_raw_parts_mut(dst_base, dst_len)
    }))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_write_v1(
    fd: usize,
    src_base: *const u8,
    src_len: usize,
) -> RawResult {
    Error::mux(posix_write(fd, unsafe {
        slice::from_raw_parts(src_base, src_len)
    }))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_fchmod_v1(fd: usize, new_mode: u16) -> RawResult {
    Error::mux(fchmod(fd, new_mode).map(|()| 0))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_fchown_v1(fd: usize, new_uid: u32, new_gid: u32) -> RawResult {
    Error::mux(fchown(fd, new_uid, new_gid).map(|()| 0))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_getdents_v0(
    fd: usize,
    buf: *mut u8,
    buf_len: usize,
    opaque: u64,
) -> RawResult {
    Error::mux(
        Sys::getdents(
            fd as c_int,
            unsafe { slice::from_raw_parts_mut(buf, buf_len) },
            opaque,
        )
        .map_err(Into::into),
    )
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_fstat_v1(
    fd: usize,
    stat: *mut crate::header::sys_stat::stat,
) -> RawResult {
    Error::mux(unsafe { fstat(fd, stat) }.map(|()| 0))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_fstatvfs_v1(
    fd: usize,
    stat: *mut crate::header::sys_statvfs::statvfs,
) -> RawResult {
    Error::mux(unsafe { fstatvfs(fd, stat) }.map(|()| 0))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_fsync_v1(fd: usize) -> RawResult {
    Error::mux(fsync(fd).map(|()| 0))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_fdatasync_v1(fd: usize) -> RawResult {
    // TODO
    Error::mux(fsync(fd).map(|()| 0))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_ftruncate_v0(fd: usize, len: usize) -> RawResult {
    Error::mux(ftruncate(fd, len).map(|()| 0))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_futimens_v1(fd: usize, times: *const timespec) -> RawResult {
    Error::mux(unsafe { futimens(fd, times) }.map(|()| 0))
}
/* TODO: Support unlinkat
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_unlinkat_v0(
    fd: usize,
    path_base: *const u8,
    path_len: usize,
    flags: u32,
) -> RawResult {
    Error::mux(std_fs_call_wo(
        fd,
        unsafe { slice::from_raw_parts(path_base, path_len) },
        &StdFsCallMeta::new(StdFsCallKind::Unlinkat, flags as u64, 0),
    ))
}
*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_fpath_v1(fd: usize, dst_base: *mut u8, dst_len: usize) -> RawResult {
    Error::mux(syscall::fpath(fd, unsafe {
        core::slice::from_raw_parts_mut(dst_base, dst_len)
    }))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_close_v1(fd: usize) -> RawResult {
    Error::mux(syscall::close(fd))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_get_pid_v1() -> RawResult {
    redox_rt::sys::posix_getpid() as _
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_get_euid_v1() -> RawResult {
    redox_rt::sys::posix_getresugid().euid as _
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_get_ruid_v1() -> RawResult {
    redox_rt::sys::posix_getresugid().ruid as _
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_get_egid_v1() -> RawResult {
    redox_rt::sys::posix_getresugid().egid as _
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_get_rgid_v1() -> RawResult {
    redox_rt::sys::posix_getresugid().rgid as _
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_get_ens_v0() -> RawResult {
    Error::mux(redox_rt::sys::getens())
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_get_ns_v0() -> RawResult {
    Error::mux(redox_rt::sys::getns())
}
#[allow(improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_get_proc_credentials_v1(
    cap_fd: usize,
    target_pid: usize,
    buf: &mut [u8], // not FFI safe
) -> RawResult {
    Error::mux(redox_rt::sys::get_proc_credentials(cap_fd, target_pid, buf))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_setrens_v1(rns: usize, ens: usize) -> RawResult {
    let _ = if ens == 0 {
        let null_namespace: [IoSlice; 2] = [IoSlice::new(b"memory"), IoSlice::new(b"pipe")];
        match redox_rt::sys::mkns(&null_namespace) {
            Ok(new_ns_fd) => redox_rt::sys::setns(new_ns_fd.take()),
            Err(e) => return Error::mux(Err(e)),
        }
    } else {
        redox_rt::sys::setns(ens)
    };
    0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_waitpid_v1(pid: usize, status: *mut i32, options: u32) -> RawResult {
    let mut sts = 0_usize;
    let res = Error::mux(redox_rt::sys::sys_waitpid(
        WaitpidTarget::from_posix_arg(pid as isize),
        &mut sts,
        WaitFlags::from_bits_truncate(options as usize),
    ));
    unsafe { status.write(sts as i32) };
    res
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_kill_v1(pid: usize, signal: u32) -> RawResult {
    Error::mux(
        redox_rt::sys::posix_kill(ProcKillTarget::from_raw(pid), signal as usize).map(|()| 0),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_sigaction_v1(
    signal: u32,
    new: *const sigaction,
    old: *mut sigaction,
) -> RawResult {
    Error::mux(
        Sys::sigaction(signal as c_int, unsafe { new.as_ref() }, unsafe {
            old.as_mut()
        })
        .map(|()| 0)
        .map_err(Into::into),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_sigprocmask_v1(
    how: u32,
    new: *const u64,
    old: *mut u64,
) -> RawResult {
    Error::mux(
        Sys::sigprocmask(how as c_int, unsafe { new.as_ref() }, unsafe {
            old.as_mut()
        })
        .map(|()| 0)
        .map_err(Into::into),
    )
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_mmap_v1(
    addr: *mut (),
    unaligned_len: usize,
    prot: u32,
    flags: u32,
    fd: usize,
    offset: u64,
) -> RawResult {
    Error::mux(unsafe {
        syscall::fmap(
            fd,
            &syscall::Map {
                address: addr as usize,
                offset: offset as usize,
                size: unaligned_len,
                flags: syscall::MapFlags::from_bits_truncate(
                    ((prot << 16) | (flags & 0xffff)) as usize,
                ),
            },
        )
    })
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_munmap_v1(addr: *mut (), unaligned_len: usize) -> RawResult {
    Error::mux(unsafe { syscall::funmap(addr as usize, unaligned_len) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_clock_gettime_v1(clock: usize, ts: *mut timespec) -> RawResult {
    Error::mux(clock_gettime(clock, unsafe { Out::nonnull(ts) }).map(|()| 0))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_strerror_v1(
    buf: *mut u8,
    buflen: *mut usize,
    error: u32,
) -> RawResult {
    let dst = unsafe { core::slice::from_raw_parts_mut(buf, buflen.read()) };

    Error::mux((|| {
        // TODO: Merge syscall::error::STR_ERROR into crate::header::error::?

        let src = syscall::error::STR_ERROR
            .get(error as usize)
            .ok_or(Error::new(EINVAL))?;

        // This API ensures that the returned buffer is proper UTF-8. Thus, it returns both the
        // copied length and the actual length.

        unsafe { buflen.write(src.len()) };

        let raw_len = core::cmp::min(dst.len(), src.len());
        let len = match core::str::from_utf8(&src.as_bytes()[..raw_len]) {
            Ok(_valid) => raw_len,
            Err(error) => error.valid_up_to(),
        };

        dst[..len].copy_from_slice(&src.as_bytes()[..len]);
        Ok(len)
    })())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_mkns_v1(
    names: *const iovec,
    num_names: usize,
    flags: u32,
) -> RawResult {
    Error::mux((|| {
        if flags != 0 {
            return Err(Error::new(EINVAL));
        }
        let raw_iovecs = unsafe { slice::from_raw_parts(names, num_names) };
        let names_ioslice: Vec<IoSlice> = raw_iovecs
            .iter()
            .map(|iov| {
                IoSlice::new(unsafe {
                    slice::from_raw_parts(iov.iov_base as *const u8, iov.iov_len)
                })
            })
            .collect();
        redox_rt::sys::mkns(&names_ioslice).map(|fd| fd.take())
    })())
}

// ABI-UNSTABLE
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_cur_procfd_v0() -> usize {
    redox_rt::current_proc_fd().as_raw_fd()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_cur_thrfd_v0() -> usize {
    redox_rt::RtTcb::current().thread_fd().as_raw_fd()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_sys_call_v0(
    fd: usize,
    payload: *mut u8,
    payload_len: usize,
    flags: usize,
    metadata: *const u64,
    metadata_len: usize,
) -> RawResult {
    Error::mux(redox_rt::sys::sys_call(
        fd,
        unsafe { slice::from_raw_parts_mut(payload, payload_len) },
        syscall::CallFlags::from_bits_retain(flags),
        unsafe { slice::from_raw_parts(metadata, metadata_len) },
    ))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_get_socket_token_v0(
    fd: usize,
    payload: *mut u8,
    payload_len: usize,
) -> RawResult {
    let metadata = [SocketCall::GetToken as u64];
    Error::mux(redox_rt::sys::sys_call_ro(
        fd,
        unsafe { slice::from_raw_parts_mut(payload, payload_len) },
        syscall::CallFlags::empty(),
        &metadata,
    ))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_setns_v0(fd: usize) -> RawResult {
    match redox_rt::sys::setns(fd) {
        Some(guard) => guard.take(),
        None => usize::MAX,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_register_scheme_to_ns_v0(
    ns_fd: usize,
    name_base: *const u8,
    name_len: usize,
    cap_fd: usize,
) -> RawResult {
    Error::mux(
        redox_rt::sys::register_scheme_to_ns(
            ns_fd,
            unsafe { str::from_utf8_unchecked(slice::from_raw_parts(name_base, name_len)) },
            cap_fd,
        )
        .map(|()| 0),
    )
}
