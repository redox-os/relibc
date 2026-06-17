use core::{
    mem::{replace, size_of},
    ptr::addr_of,
    sync::atomic::{AtomicU32, Ordering},
};

use ioslice::IoSlice;
use syscall::{
    Call, CallFlags, EINVAL, ERESTART, StdFsCallKind, TimeSpec,
    data::StdFsCallMeta,
    error::{self, EBADF, EEXIST, EINTR, EMFILE, ENODEV, ESRCH, Error, Result},
};

use crate::{
    DYNAMIC_PROC_INFO, DynamicProcInfo, FILETABLE, RtTcb, Tcb,
    arch::manually_enter_trampoline,
    proc::{FdGuard, FdGuardUpper},
    read_proc_meta,
    signal::tmp_disable_signals,
};
use alloc::{collections::btree_set::BTreeSet, vec::Vec};
use redox_protocols::protocol::{
    NsDup, ProcCall, ProcKillTarget, RtSigInfo, ThreadCall, WaitFlags,
};

#[inline]
fn wrapper<T>(restart: bool, erestart: bool, mut f: impl FnMut() -> Result<T>) -> Result<T> {
    loop {
        let _guard = tmp_disable_signals();
        let rt_sigarea = unsafe { &Tcb::current().unwrap().os_specific };
        let res = f();
        let code = if erestart { ERESTART } else { EINTR };

        if let Err(err) = res
            && err == Error::new(code)
        {
            unsafe {
                manually_enter_trampoline();
            }
            if restart && unsafe { (*rt_sigarea.arch.get()).last_sig_was_restart } {
                continue;
            }
        }

        return res;
    }
}
// TODO: uninitialized memory?
#[inline]
pub fn posix_read(fd: usize, buf: &mut [u8]) -> Result<usize> {
    wrapper(true, false, || syscall::read(fd, buf))
}
#[inline]
pub fn posix_write(fd: usize, buf: &[u8]) -> Result<usize> {
    wrapper(true, false, || syscall::write(fd, buf))
}
#[inline]
pub fn posix_kill(target: ProcKillTarget, sig: usize) -> Result<()> {
    if sig > 64 {
        return Err(Error::new(EINVAL));
    }

    match wrapper(false, true, || {
        this_proc_call(
            &mut [],
            CallFlags::empty(),
            &[ProcCall::Kill as u64, target.raw() as u64, sig as u64],
        )
    }) {
        Ok(_) | Err(Error { errno: ERESTART }) => Ok(()),
        Err(error) => Err(error),
    }
}
#[inline]
pub fn posix_sigqueue(pid: usize, sig: usize, arg: usize) -> Result<()> {
    let target = ProcKillTarget::from_raw(pid);
    if !matches!(target, ProcKillTarget::SingleProc(_)) {
        return Err(Error::new(ESRCH));
    }
    if sig <= 32 {
        return posix_kill(target, sig);
    }
    let mut siginf = RtSigInfo {
        arg,
        code: -1, // TODO: SI_QUEUE constant
        uid: 0,   // TODO
        pid: posix_getpid(),
    };
    match wrapper(false, true, || {
        this_proc_call(
            unsafe { plain::as_mut_bytes(&mut siginf) },
            CallFlags::empty(),
            &[ProcCall::Sigq as u64, pid as u64, sig as u64],
        )
    }) {
        Ok(_)
        | Err(Error {
            errno: error::ERESTART,
        }) => Ok(()),
        Err(error) => Err(error),
    }
}
#[inline]
pub fn posix_getpid() -> u32 {
    // SAFETY: read-only except during program/fork child initialization
    unsafe { addr_of!((*crate::STATIC_PROC_INFO.get()).pid).read() }
}
#[inline]
pub fn posix_getppid() -> u32 {
    this_proc_call(&mut [], CallFlags::empty(), &[ProcCall::Getppid as u64]).expect("cannot fail")
        as u32
}

#[inline]
pub fn posix_setpriority(which: i32, who: u32, prio: u32) -> Result<(), syscall::Error> {
    if which != 0 {
        return Err(syscall::Error::new(syscall::EINVAL)); // TODO: Add support for PRIO_PGRP and PRIO_PROCESS
    }

    this_proc_call(
        &mut [],
        CallFlags::empty(),
        &[ProcCall::SetProcPriority as u64, who as u64, prio as u64],
    )?;

    Ok(())
}

#[inline]
pub fn posix_getpriority(which: i32, who: u32) -> Result<u32, syscall::Error> {
    if which != 0 {
        return Err(syscall::Error::new(syscall::EINVAL));
    }

    let res = this_proc_call(
        &mut [],
        CallFlags::empty(),
        &[ProcCall::GetProcPriority as u64, who as u64],
    )?;

    Ok(res as u32)
}

#[inline]
pub unsafe fn sys_futex_wait(addr: *mut u32, val: u32, deadline: Option<&TimeSpec>) -> Result<()> {
    wrapper(true, false, || {
        unsafe {
            syscall::syscall5(
                syscall::SYS_FUTEX,
                addr as usize,
                syscall::FUTEX_WAIT,
                val as usize,
                deadline.map_or(0, |d| d as *const _ as usize),
                0,
            )
        }
        .map(|_| ())
    })
}
#[inline]
pub unsafe fn sys_futex_wake(addr: *mut u32, num: u32) -> Result<u32> {
    unsafe {
        syscall::syscall5(
            syscall::SYS_FUTEX,
            addr as usize,
            syscall::FUTEX_WAKE,
            num as usize,
            0,
            0,
        )
    }
    .map(|awoken| awoken as u32)
}
pub fn sys_call_ro<T: Call>(
    fd: T,
    payload: &mut [u8],
    flags: CallFlags,
    metadata: &[u64],
) -> Result<usize> {
    if !flags.contains(CallFlags::FD) {
        return unsafe {
            fd.raw_call(
                payload.as_mut_ptr(),
                payload.len(),
                flags | CallFlags::READ,
                metadata,
            )
        };
    }

    let _siglock = tmp_disable_signals();

    if payload.len() % size_of::<usize>() != 0 {
        return Err(Error::new(EINVAL));
    }

    let fd_slice = unsafe {
        core::slice::from_raw_parts_mut(
            payload.as_mut_ptr() as *mut usize,
            payload.len() / size_of::<usize>(),
        )
    };

    if fd_slice.is_empty() {
        return Err(Error::new(EINVAL));
    }

    let is_automated = fd_slice[0] == usize::MAX;
    let mut backup_handles = Vec::with_capacity(fd_slice.len());

    if !is_automated {
        backup_handles.extend_from_slice(fd_slice);
    }

    let which = if flags.contains(CallFlags::FD_UPPER) {
        syscall::UPPER_FDTBL_TAG
    } else {
        0
    };
    let entry_flags = if flags.contains(CallFlags::FD_CLOEXEC) {
        syscall::O_CLOEXEC
    } else {
        0
    };

    FILETABLE.lock().bulk_insert(which, fd_slice, entry_flags)?;

    if is_automated {
        backup_handles.extend_from_slice(fd_slice);
    }

    let res = unsafe {
        fd.raw_call(
            payload.as_mut_ptr(),
            payload.len(),
            flags | CallFlags::READ,
            metadata,
        )
    };

    if res.is_err() {
        let mut guard = FILETABLE.lock();
        for &handle in &backup_handles {
            let _ = guard.remove(handle);
        }
        return res;
    }

    res
}

pub fn sys_call_wo<T: Call>(
    fd: T,
    payload: &[u8],
    flags: CallFlags,
    metadata: &[u64],
) -> Result<usize> {
    if !flags.contains(CallFlags::FD) {
        return unsafe {
            fd.raw_call(
                payload.as_ptr(),
                payload.len(),
                flags | CallFlags::WRITE,
                metadata,
            )
        };
    }
    let _siglock = tmp_disable_signals();

    if payload.len() % size_of::<usize>() != 0 {
        return Err(Error::new(EINVAL));
    }
    let fd_slice = unsafe {
        core::slice::from_raw_parts(
            payload.as_ptr() as *const usize,
            payload.len() / size_of::<usize>(),
        )
    };

    let res = unsafe {
        fd.raw_call(
            payload.as_ptr(),
            payload.len(),
            flags | CallFlags::WRITE,
            metadata,
        )
    };

    if res.is_ok() && !flags.contains(CallFlags::FD_CLONE) {
        let mut guard = FILETABLE.lock();
        for &handle in fd_slice {
            let _ = guard.remove(handle);
        }
    }

    res
}
pub fn sys_call_rw<T: Call>(
    fd: T,
    payload: &mut [u8],
    flags: CallFlags,
    metadata: &[u64],
) -> Result<usize> {
    unsafe {
        fd.raw_call(
            payload.as_mut_ptr(),
            payload.len(),
            flags | CallFlags::READ | CallFlags::WRITE,
            metadata,
        )
    }
}
pub fn sys_call<T: Call>(
    fd: T,
    payload: &mut [u8],
    flags: CallFlags,
    metadata: &[u64],
) -> Result<usize> {
    unsafe { fd.raw_call(payload.as_mut_ptr(), payload.len(), flags, metadata) }
}

pub fn this_proc_call(payload: &mut [u8], flags: CallFlags, metadata: &[u64]) -> Result<usize> {
    proc_call(
        crate::current_proc_fd().as_raw_fd(),
        payload,
        flags,
        metadata,
    )
}
pub fn proc_call(
    proc_fd: usize,
    payload: &mut [u8],
    flags: CallFlags,
    metadata: &[u64],
) -> Result<usize> {
    sys_call(proc_fd, payload, flags, metadata)
}
pub fn thread_call(
    thread_fd: usize,
    payload: &mut [u8],
    flags: CallFlags,
    metadata: &[u64],
) -> Result<usize> {
    sys_call(thread_fd, payload, flags, metadata)
}
pub fn this_thread_call(payload: &mut [u8], flags: CallFlags, metadata: &[u64]) -> Result<usize> {
    thread_call(
        RtTcb::current().thread_fd().as_raw_fd(),
        payload,
        flags,
        metadata,
    )
}

#[derive(Clone, Copy, Debug)]
pub enum WaitpidTarget {
    AnyChild,
    AnyGroupMember,
    SingleProc { pid: usize },
    ProcGroup { pgid: usize },
}
impl WaitpidTarget {
    pub fn from_posix_arg(raw: isize) -> Self {
        match raw {
            0 => Self::AnyGroupMember,
            -1 => Self::AnyChild,
            1.. => Self::SingleProc { pid: raw as usize },
            ..-1 => Self::ProcGroup {
                pgid: -raw as usize,
            },
        }
    }
}

pub fn sys_waitpid(target: WaitpidTarget, status: &mut usize, flags: WaitFlags) -> Result<usize> {
    let (call, pid) = match target {
        WaitpidTarget::AnyChild => (ProcCall::Waitpid, 0),
        WaitpidTarget::SingleProc { pid } => (ProcCall::Waitpid, pid),
        WaitpidTarget::AnyGroupMember => (ProcCall::Waitpgid, 0),
        WaitpidTarget::ProcGroup { pgid } => (ProcCall::Waitpgid, pgid),
    };
    wrapper(true, false, || {
        this_proc_call(
            unsafe { plain::as_mut_bytes(status) },
            CallFlags::empty(),
            &[call as u64, pid as u64, flags.bits() as u64],
        )
    })
}
pub fn posix_kill_thread(thread_fd: usize, signal: u32) -> Result<()> {
    // TODO: don't hardcode?
    if signal > 64 {
        return Err(Error::new(EINVAL));
    }

    match wrapper(false, true, || {
        thread_call(
            thread_fd,
            &mut [],
            CallFlags::empty(),
            &[ThreadCall::SignalThread as u64, signal.into()],
        )
    }) {
        Ok(_) | Err(Error { errno: ERESTART }) => Ok(()),
        Err(error) => Err(error),
    }
}

static UMASK: AtomicU32 = AtomicU32::new(0o022);

/// Controls the set of bits removed from the `mode` mask when new file descriptors are created.
///
/// Must be validated by the caller
//
// TODO: validate here?
#[inline]
pub fn swap_umask(mask: u32) -> u32 {
    UMASK.swap(mask, Ordering::AcqRel)
}

#[inline]
pub fn get_umask() -> u32 {
    UMASK.load(Ordering::Acquire)
}

/// Real/Effective/Set-User/Group ID
pub struct Resugid<T> {
    pub ruid: T,
    pub euid: T,
    pub suid: T,
    pub rgid: T,
    pub egid: T,
    pub sgid: T,
}

/// Sets [res][ug]id, fields that are None will be unchanged.
pub fn posix_setresugid(ids: &Resugid<Option<u32>>, pid: Option<usize>) -> Result<()> {
    // TODO: not sure how "tmp" an IPC call is?
    let _sig_guard = tmp_disable_signals();
    let mut guard = DYNAMIC_PROC_INFO.lock();

    let mut buf = [0_u8; size_of::<u32>() * 6];
    plain::slice_from_mut_bytes(&mut buf)
        .unwrap()
        .copy_from_slice(&[
            ids.ruid.unwrap_or(u32::MAX),
            ids.euid.unwrap_or(u32::MAX),
            ids.suid.unwrap_or(u32::MAX),
            ids.rgid.unwrap_or(u32::MAX),
            ids.egid.unwrap_or(u32::MAX),
            ids.sgid.unwrap_or(u32::MAX),
        ]);

    if let Some(pid) = pid {
        proc_call(
            pid,
            &mut buf,
            CallFlags::empty(),
            &[ProcCall::SetResugid as u64],
        )?;
    } else {
        this_proc_call(&mut buf, CallFlags::empty(), &[ProcCall::SetResugid as u64])?;
    }

    if let Some(ruid) = ids.ruid {
        guard.ruid = ruid;
    }
    if let Some(euid) = ids.euid {
        guard.euid = euid;
    }
    if let Some(suid) = ids.suid {
        guard.suid = suid;
    }
    if let Some(rgid) = ids.rgid {
        guard.rgid = rgid;
    }
    if let Some(egid) = ids.egid {
        guard.egid = egid;
    }
    if let Some(sgid) = ids.sgid {
        guard.sgid = sgid;
    }

    Ok(())
}
pub fn posix_getresugid() -> Resugid<u32> {
    let _sig_guard = tmp_disable_signals();
    let DynamicProcInfo {
        ruid,
        euid,
        suid,
        rgid,
        egid,
        sgid,
        ..
    } = *DYNAMIC_PROC_INFO.lock();
    Resugid {
        ruid,
        euid,
        suid,
        rgid,
        egid,
        sgid,
    }
}
pub fn getens() -> Result<usize> {
    read_proc_meta(crate::current_proc_fd()).map(|meta| meta.ens as usize)
}
pub fn get_proc_credentials(cap_fd: usize, target_pid: usize, buf: &mut [u8]) -> Result<usize> {
    if buf.len() < size_of::<redox_protocols::protocol::ProcMeta>() {
        return Err(Error::new(EINVAL));
    }
    proc_call(
        cap_fd,
        buf,
        CallFlags::empty(),
        &[ProcCall::GetProcCredentials as u64, target_pid as u64],
    )
}
pub fn posix_exit(status: i32) -> ! {
    // TODO: probably not correct place to handle EINTR
    loop {
        match this_proc_call(
            &mut [],
            CallFlags::empty(),
            &[ProcCall::Exit as u64, (status & 0xFF) as u64],
        ) {
            Ok(_) => break,
            Err(Error { errno: EINTR }) => continue,
            Err(e) => panic!("failed to call proc mgr with Exit: {e}"),
        }
    }
    let _ = syscall::write(1, b"redox-rt: ProcCall::Exit FAILED, abort()ing!\n");
    core::intrinsics::abort();
}
pub fn posix_getpgid(pid: usize) -> Result<usize> {
    this_proc_call(
        &mut [],
        CallFlags::empty(),
        &[ProcCall::Setpgid as u64, pid as u64, u64::wrapping_neg(1)],
    )
}
pub fn posix_setpgid(pid: usize, pgid: usize) -> Result<()> {
    if pgid == usize::wrapping_neg(1) {
        return Err(Error::new(EINVAL));
    }
    this_proc_call(
        &mut [],
        CallFlags::empty(),
        &[ProcCall::Setpgid as u64, pid as u64, pgid as u64],
    )?;
    Ok(())
}
pub fn posix_getsid(pid: usize) -> Result<usize> {
    this_proc_call(
        &mut [],
        CallFlags::empty(),
        &[ProcCall::Getsid as u64, pid as u64],
    )
}
pub fn posix_setsid() -> Result<u32> {
    this_proc_call(&mut [], CallFlags::empty(), &[ProcCall::Setsid as u64])?;
    Ok(posix_getpid())
}
pub fn posix_nanosleep(rqtp: &TimeSpec, rmtp: &mut TimeSpec) -> Result<()> {
    wrapper(false, false, || syscall::nanosleep(rqtp, rmtp))?;
    Ok(())
}
pub fn setns(fd: usize) -> Option<FdGuardUpper> {
    let mut info = DYNAMIC_PROC_INFO.lock();
    let new_fd_guard = FdGuard::new(fd).to_upper().unwrap();
    let old_fd_guard = replace(&mut info.ns_fd, Some(new_fd_guard));
    old_fd_guard
}
pub fn getns() -> Result<usize> {
    let cur_ns = crate::current_namespace_fd()?;
    if cur_ns == usize::MAX {
        Err(Error::new(ENODEV))
    } else {
        Ok(cur_ns)
    }
}

pub fn open<T: AsRef<str>>(path: T, flags: usize) -> Result<usize> {
    let fcntl_flags = flags & syscall::O_FCNTL_MASK;
    openat_into_posix(crate::current_namespace_fd()?, path, flags, fcntl_flags)
}
pub fn openat<T: AsRef<str>>(
    fd: usize,
    path: T,
    flags: usize,
    fcntl_flags: usize,
) -> Result<usize> {
    openat_into_posix(fd, path, flags, fcntl_flags)
}
fn openat_into_posix<T: AsRef<str>>(
    fd: usize,
    path: T,
    flags: usize,
    fcntl_flags: usize,
) -> Result<usize> {
    let _siglock = tmp_disable_signals();
    let path = path.as_ref();

    let out = {
        let mut guard = FILETABLE.lock();
        guard.add_posix(flags)?
    };

    let res = unsafe {
        syscall::syscall6(
            syscall::SYS_OPENAT_INTO,
            fd,
            path.as_ptr() as usize,
            path.len(),
            flags,
            fcntl_flags,
            out,
        )
    };

    if res.is_err() {
        let mut guard = FILETABLE.lock();
        let _ = guard.remove(out);
        return res;
    }

    Ok(out)
}
pub fn open_into_upper<T: AsRef<str>>(path: T, flags: usize) -> Result<usize> {
    let fcntl_flags = flags & syscall::O_FCNTL_MASK;
    openat_into_upper(crate::current_namespace_fd()?, path, flags, fcntl_flags)
}
pub fn dup(fd: usize, buf: &[u8]) -> Result<usize> {
    let _siglock = tmp_disable_signals();

    let out = {
        let mut guard = FILETABLE.lock();
        guard.add_posix(0)?
    };

    let res = unsafe {
        syscall::syscall4(
            syscall::SYS_DUP_INTO,
            fd,
            buf.as_ptr() as usize,
            buf.len(),
            out,
        )
    };

    if res.is_err() {
        let mut guard = FILETABLE.lock();
        let _ = guard.remove(out);
        return res;
    }

    Ok(out)
}

pub fn dup2(fd: usize, newfd: usize, buf: &[u8]) -> Result<usize> {
    let _siglock = tmp_disable_signals();

    let out = {
        let mut guard = FILETABLE.lock();
        guard.override_at(fd, newfd)?
    };

    let res = unsafe {
        syscall::syscall4(
            syscall::SYS_DUP2,
            fd,
            newfd,
            buf.as_ptr() as usize,
            buf.len(),
        )
    };

    if res.is_err() {
        let mut guard = FILETABLE.lock();
        let _ = guard.remove(out);
        return res;
    }

    Ok(out)
}

pub fn unlink<T: AsRef<str>>(path: T, flags: usize) -> Result<usize> {
    let path = path.as_ref();
    unsafe {
        syscall::syscall4(
            syscall::SYS_UNLINKAT,
            crate::current_namespace_fd()?,
            path.as_ptr() as usize,
            path.len(),
            flags,
        )
    }
}
pub fn mkns(names: &[IoSlice]) -> Result<FdGuardUpper> {
    let mut buf = Vec::from((NsDup::ForkNs as usize).to_ne_bytes());
    for name in names {
        let name_bytes = name.as_slice();
        let len = name_bytes.len();
        let _scheme_name = core::str::from_utf8(name_bytes).map_err(|_| Error::new(EINVAL))?;
        buf.extend_from_slice(&len.to_ne_bytes());
        buf.extend_from_slice(name_bytes);
    }
    FdGuard::new(dup_into_upper(crate::current_namespace_fd()?, &buf, 0)?).to_upper()
}
pub fn register_scheme_to_ns(ns_fd: usize, name: &str, cap_fd: usize) -> Result<()> {
    let mut buf = alloc::vec::Vec::from((NsDup::IssueRegister as usize).to_ne_bytes());
    buf.extend_from_slice(name.as_bytes());
    let ns_this_scheme = FdGuard::new(crate::sys::dup(ns_fd, &buf)?);
    let cap_bytes = cap_fd.to_ne_bytes();
    ns_this_scheme.call_wo(&cap_bytes, CallFlags::FD, &[])?;
    Ok(())
}
pub fn std_fs_call_ro<T: Call>(
    fd: T,
    payload: &mut [u8],
    metadata: &StdFsCallMeta,
) -> Result<usize> {
    sys_call_ro(fd, payload, CallFlags::STD_FS, metadata)
}
pub fn std_fs_call_wo<T: Call>(fd: T, payload: &[u8], metadata: &StdFsCallMeta) -> Result<usize> {
    sys_call_wo(fd, payload, CallFlags::STD_FS, metadata)
}
pub fn std_fs_call_rw<T: Call>(
    fd: T,
    payload: &mut [u8],
    metadata: &StdFsCallMeta,
) -> Result<usize> {
    sys_call_rw(fd, payload, CallFlags::STD_FS, metadata)
}
pub fn fstat(fd: usize, stat: &mut syscall::Stat) -> Result<usize> {
    std_fs_call_ro(fd, stat, &StdFsCallMeta::new(StdFsCallKind::Fstat, 0, 0))
}

pub fn fcntl(fd: usize, cmd: usize, arg: usize) -> Result<usize> {
    if cmd == syscall::F_DUPFD || cmd == syscall::F_DUPFD_CLOEXEC {
        let _siglock = tmp_disable_signals();

        let cloexec_flag = if cmd == syscall::F_DUPFD_CLOEXEC {
            syscall::O_CLOEXEC
        } else {
            0
        };

        let out = {
            let mut guard = FILETABLE.lock();
            if arg & syscall::UPPER_FDTBL_TAG != 0 {
                guard.insert_upper(cloexec_flag)? | syscall::UPPER_FDTBL_TAG
            } else {
                guard.add_posix(cloexec_flag)?
            }
        };

        let res = unsafe { syscall::syscall3(syscall::SYS_FCNTL, fd, cmd, out) };

        if res.is_err() {
            let mut guard = FILETABLE.lock();
            let _ = guard.remove(out);
            return res;
        }

        let actual_fd = res.unwrap();
        if actual_fd != out {
            let mut guard = FILETABLE.lock();
            let _ = guard.remove(out);
            guard.override_at(actual_fd, actual_fd)?;
            if cloexec_flag != 0 {
                guard.set_fd_flags(actual_fd, cloexec_flag)?;
            }
        }

        return Ok(actual_fd);
    }

    if cmd == syscall::F_GETFD {
        let _siglock = tmp_disable_signals();
        return FILETABLE.lock().get_fd_flags(fd);
    }

    if cmd == syscall::F_SETFD {
        let _siglock = tmp_disable_signals();

        let res = unsafe { syscall::syscall3(syscall::SYS_FCNTL, fd, cmd, arg) };
        if res.is_err() {
            return res;
        }

        FILETABLE.lock().set_fd_flags(fd, arg)?;
        return Ok(0);
    }

    unsafe { syscall::syscall3(syscall::SYS_FCNTL, fd, cmd, arg) }
}

pub fn openat_into_upper<T: AsRef<str>>(
    fd: usize,
    path: T,
    flags: usize,
    fcntl_flags: usize,
) -> Result<usize> {
    let _siglock = tmp_disable_signals();
    let path = path.as_ref();

    let out_idx = {
        let mut guard = FILETABLE.lock();
        guard.insert_upper(flags)?
    };
    let out = out_idx | syscall::UPPER_FDTBL_TAG;

    let res = unsafe {
        syscall::syscall6(
            syscall::SYS_OPENAT_INTO,
            fd,
            path.as_ptr() as usize,
            path.len(),
            flags,
            fcntl_flags,
            out,
        )
    };

    if res.is_err() {
        let mut guard = FILETABLE.lock();
        let _ = guard.remove(out);
        return res;
    }

    Ok(out)
}

pub fn dup_into_upper(fd: usize, buf: &[u8], flags: usize) -> Result<usize> {
    let _siglock = tmp_disable_signals();

    let out_idx = {
        let mut guard = FILETABLE.lock();
        guard.insert_upper(flags)?
    };
    let out = out_idx | syscall::UPPER_FDTBL_TAG;

    let res = unsafe {
        syscall::syscall4(
            syscall::SYS_DUP_INTO,
            fd,
            buf.as_ptr() as usize,
            buf.len(),
            out,
        )
    };

    if res.is_err() {
        let mut guard = FILETABLE.lock();
        let _ = guard.remove(out);
        return res;
    }

    Ok(out)
}

pub fn dup_into_upper_raw(fd: usize, buf: &[u8], flags: usize) -> Result<usize> {
    let out_idx = {
        let mut guard = FILETABLE.lock();
        guard.insert_upper(flags)?
    };
    let out = out_idx | syscall::UPPER_FDTBL_TAG;

    let res = unsafe {
        syscall::syscall4(
            syscall::SYS_DUP_INTO,
            fd,
            buf.as_ptr() as usize,
            buf.len(),
            out,
        )
    };

    if res.is_err() {
        let mut guard = FILETABLE.lock();
        let _ = guard.remove(out);
        return res;
    }

    Ok(out)
}

pub fn close(fd: usize) -> Result<usize> {
    let _siglock = tmp_disable_signals();

    let is_upper = (fd & syscall::UPPER_FDTBL_TAG) != 0;

    let res = unsafe { syscall::syscall1(syscall::SYS_CLOSE, fd) };

    if res.is_ok() {
        let mut guard = FILETABLE.lock();
        let _ = guard.remove(fd);
    }

    res
}

pub fn close_raw(fd: usize) -> Result<usize> {
    let res = unsafe { syscall::syscall1(syscall::SYS_CLOSE, fd) };

    if res.is_ok() {
        let mut guard = FILETABLE.lock();
        let _ = guard.remove(fd);
    }

    res
}

pub struct FdTbl {
    fd: Option<FdGuardUpper>,
    posix_fdtbl: PosixFdTbl,
    upper_fdtbl: UpperFdTbl,
    active_count: usize,
}

impl FdTbl {
    pub const CONTEXT_MAX_FILES: u32 = 65_536;
    pub const DEFAULT_CAPACITY: usize = usize::BITS as usize;

    pub const fn new() -> Self {
        Self {
            fd: None,
            posix_fdtbl: PosixFdTbl::new(),
            upper_fdtbl: UpperFdTbl::new(),
            active_count: 0,
        }
    }

    pub fn from_binary_fd(filetable_fd: FdGuardUpper) -> Result<Self> {
        let mut fdtbl = Self::new();
        let files_reader_fd = filetable_fd.as_raw_fd();
        let _ = filetable_fd.lseek(0, syscall::flag::SEEK_SET);
        fdtbl.set_fd(filetable_fd);
        fdtbl.reserve(Self::DEFAULT_CAPACITY);

        let mut reader = crate::proc::FileBufReader::from_fd(files_reader_fd);
        fdtbl.populate(&mut reader)?;

        // Manually mark the filetable_fd itself as occupied in userspace FILETABLE
        fdtbl.override_at(files_reader_fd, files_reader_fd)?;

        Ok(fdtbl)
    }

    pub fn with_capacity(capacity: usize, fd: FdGuardUpper) -> Self {
        Self {
            fd: Some(fd),
            posix_fdtbl: PosixFdTbl::with_capacity(capacity),
            upper_fdtbl: UpperFdTbl::with_capacity(capacity),
            active_count: 0,
        }
    }

    pub fn fd(&self) -> Option<&FdGuardUpper> {
        self.fd.as_ref()
    }

    pub fn take(&mut self) -> Option<FdGuardUpper> {
        self.fd.take()
    }

    pub fn set_fd(&mut self, fd: FdGuardUpper) {
        self.fd = Some(fd);
    }

    pub fn reserve(&mut self, additional: usize) {
        self.posix_fdtbl.reserve(additional);
        self.upper_fdtbl.reserve(additional);
    }

    pub fn upper_capacity(&self) -> usize {
        self.upper_fdtbl.capacity()
    }

    pub fn upper_len(&self) -> usize {
        self.upper_fdtbl.len()
    }

    fn strip_tags(index: usize) -> usize {
        index & !syscall::UPPER_FDTBL_TAG
    }

    fn is_upper(index: usize) -> bool {
        (index & syscall::UPPER_FDTBL_TAG) != 0
    }

    pub(crate) fn populate(&mut self, reader: &mut crate::proc::FileBufReader) -> Result<()> {
        while let Some(fd) = reader.read_le_u64()? {
            let fd = fd as usize;
            self.override_at(fd, fd)?;
        }
        Ok(())
    }

    pub fn get_fd_flags(&self, fd: usize) -> Result<usize> {
        if Self::is_upper(fd) {
            let flags = self.upper_fdtbl.get_flags(fd)?;
            let mut raw_flags = 0;
            if flags & (syscall::O_CLOEXEC as u32) != 0 {
                raw_flags |= syscall::O_CLOEXEC;
            }
            Ok(raw_flags)
        } else {
            let flags = self.posix_fdtbl.get_flags(fd)?;
            let mut raw_flags = 0;
            if flags.contains(FdFlags::CLOEXEC) {
                raw_flags |= syscall::O_CLOEXEC;
            }
            Ok(raw_flags)
        }
    }

    pub fn set_fd_flags(&mut self, fd: usize, raw_flags: usize) -> Result<()> {
        if Self::is_upper(fd) {
            let old_flags = self.upper_fdtbl.get_flags(fd)?;
            let mut new_flags = old_flags & !(syscall::O_CLOEXEC as u32);
            if raw_flags & syscall::O_CLOEXEC != 0 {
                new_flags |= syscall::O_CLOEXEC as u32;
            }
            self.upper_fdtbl.set_flags(fd, new_flags)?;
        } else {
            let mut new_flags = FdFlags::empty();
            if raw_flags & syscall::O_CLOEXEC != 0 {
                new_flags.insert(FdFlags::CLOEXEC);
            }
            self.posix_fdtbl.set_flags(fd, new_flags)?;
        }
        Ok(())
    }

    fn sync_capacity(&self, old_capacity: usize, tag: usize) -> Result<()> {
        let (new_capacity, current_len, tag) = if tag & syscall::UPPER_FDTBL_TAG == 0 {
            (self.posix_fdtbl.capacity(), self.posix_fdtbl.len(), 0)
        } else {
            (
                self.upper_fdtbl.capacity(),
                self.upper_fdtbl.len(),
                syscall::UPPER_FDTBL_TAG,
            )
        };

        if old_capacity != new_capacity {
            let available_slots = new_capacity - current_len;

            if let Some(ref fd) = self.fd {
                let _ = fd.call_wo(
                    &[],
                    CallFlags::empty(),
                    &[
                        syscall::FileTableVerb::Reserve as u64,
                        tag as u64,
                        available_slots as u64,
                    ],
                )?;
            }
        }
        Ok(())
    }

    pub fn override_at(&mut self, fd: usize, new_fd: usize) -> Result<usize> {
        let existed = self.remove(new_fd).is_ok();

        if Self::is_upper(new_fd) {
            let handle = Self::strip_tags(new_fd);
            self.upper_fdtbl
                .insert_at(handle, UpperFdTbl::flags_into_entry(0))?;
        } else {
            self.posix_fdtbl
                .insert_at(new_fd, PosixFdTbl::flags_into_entry(0))?;
        }

        if !existed {
            self.active_count += 1;
        }
        Ok(new_fd)
    }
    pub fn add_posix(&mut self, entry: usize) -> Result<usize> {
        if self.active_count >= Self::CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }

        let old_capacity = self.posix_fdtbl.capacity();

        let out_idx = self.posix_fdtbl.add(PosixFdTbl::flags_into_entry(entry))?;
        self.active_count += 1;

        self.sync_capacity(old_capacity, 0)?;

        Ok(out_idx)
    }

    pub fn insert_upper(&mut self, entry: usize) -> Result<usize> {
        if self.active_count >= Self::CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }

        let old_capacity = self.upper_fdtbl.capacity();

        let out_idx = self
            .upper_fdtbl
            .insert(UpperFdTbl::flags_into_entry(entry))?;
        self.active_count += 1;

        self.sync_capacity(old_capacity, syscall::UPPER_FDTBL_TAG)?;

        Ok(out_idx)
    }

    pub fn insert_at_upper(&mut self, new_fd: usize, entry: usize) -> Result<usize> {
        if self.active_count >= Self::CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }

        let old_capacity = self.upper_fdtbl.capacity();
        if !Self::is_upper(new_fd) {
            return Err(Error::new(EINVAL));
        }
        let handle = Self::strip_tags(new_fd);

        let out_idx = self
            .upper_fdtbl
            .insert_at(handle, UpperFdTbl::flags_into_entry(entry))?;
        self.active_count += 1;

        self.sync_capacity(old_capacity, syscall::UPPER_FDTBL_TAG)?;

        Ok(out_idx)
    }

    pub fn bulk_add(
        &mut self,
        which: usize,
        fd_slice: &mut [usize],
        flags: usize,
    ) -> Result<usize> {
        let cnt = fd_slice.len();
        if cnt == 0 {
            return Ok(0);
        }

        if self.active_count + cnt > Self::CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }

        if which & syscall::UPPER_FDTBL_TAG == 0 {
            let old_capacity = self.posix_fdtbl.capacity();

            let initial_flag = PosixFdTbl::flags_into_entry(flags);
            let entries = alloc::vec![initial_flag; cnt];

            let handles = self.posix_fdtbl.bulk_add_posix(entries)?;
            self.active_count += cnt;

            self.sync_capacity(old_capacity, 0)?;

            for (i, &handle) in handles.iter().enumerate() {
                fd_slice[i] = handle;
            }
        } else {
            let old_capacity = self.upper_fdtbl.capacity();

            let entries = alloc::vec![UpperFdTbl::flags_into_entry(flags); cnt];
            let handles = self.upper_fdtbl.bulk_insert(entries)?;
            self.active_count += cnt;

            self.sync_capacity(old_capacity, syscall::UPPER_FDTBL_TAG)?;

            for (i, &handle) in handles.iter().enumerate() {
                fd_slice[i] = handle | syscall::UPPER_FDTBL_TAG;
            }
        }

        Ok(cnt)
    }

    pub fn bulk_insert(
        &mut self,
        which: usize,
        fd_slice: &mut [usize],
        flags: usize,
    ) -> Result<usize> {
        let cnt = fd_slice.len();
        if cnt == 0 {
            return Ok(0);
        }

        if fd_slice[0] == usize::MAX {
            return self.bulk_add(which, fd_slice, flags);
        }

        if self.active_count + cnt > Self::CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }

        if which & syscall::UPPER_FDTBL_TAG == 0 {
            let old_capacity = self.posix_fdtbl.capacity();

            let initial_flag = PosixFdTbl::flags_into_entry(flags);
            let entries = alloc::vec![initial_flag; cnt];

            self.posix_fdtbl.bulk_insert_manual(entries, fd_slice)?;
            self.active_count += cnt;

            self.sync_capacity(old_capacity, 0)?;
        } else {
            let old_capacity = self.upper_fdtbl.capacity();

            let entries = alloc::vec![UpperFdTbl::flags_into_entry(flags); cnt];
            self.upper_fdtbl.bulk_insert_manual(entries, fd_slice)?;
            self.active_count += cnt;

            self.sync_capacity(old_capacity, syscall::UPPER_FDTBL_TAG)?;
        }

        Ok(cnt)
    }

    pub fn remove(&mut self, fd: usize) -> Result<()> {
        if Self::is_upper(fd) {
            let handle = Self::strip_tags(fd);
            if self.upper_fdtbl.remove(handle).is_some() {
                self.active_count -= 1;
                Ok(())
            } else {
                Err(Error::new(EBADF))
            }
        } else {
            if self.posix_fdtbl.remove(fd).is_some() {
                self.active_count -= 1;
                Ok(())
            } else {
                Err(Error::new(EBADF))
            }
        }
    }
}

pub struct PosixFdTbl {
    table: Vec<FdFlags>,
    lowest_idx: u32,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Default, PartialEq, Eq)]
    pub struct FdFlags: u8 {
        const VACANT   = 0;
        const OCCUPIED = 1 << 0;
        const CLOEXEC  = 1 << 1;
        const CLOFORK  = 1 << 2;
    }
}

impl PosixFdTbl {
    pub const fn new() -> Self {
        Self {
            table: Vec::new(),
            lowest_idx: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            table: Vec::with_capacity(capacity),
            lowest_idx: 0,
        }
    }

    pub fn get_flags(&self, handle: usize) -> Result<FdFlags> {
        self.table
            .get(handle)
            .copied()
            .filter(|flags| flags.contains(FdFlags::OCCUPIED))
            .ok_or(Error::new(EBADF))
    }

    pub fn set_flags(&mut self, handle: usize, flags: FdFlags) -> Result<()> {
        if !self.is_occupied(handle) {
            return Err(Error::new(EBADF));
        }
        let entry = self.table[handle];
        self.table[handle] = (entry & FdFlags::OCCUPIED) | flags;
        Ok(())
    }

    pub fn flags_into_entry(flags: usize) -> FdFlags {
        let mut new_entry = FdFlags::OCCUPIED;
        if flags & syscall::O_CLOEXEC != 0 {
            new_entry.insert(FdFlags::CLOEXEC);
        }
        /* TODO: Support O_CLOFORK
        if flags & syscall::O_CLOFORK != 0 {
            new_entry.insert(FdFlags::CLOFORK);
        }
        */
        new_entry
    }

    fn is_vacant(&self, handle: usize) -> bool {
        self.table
            .get(handle)
            .map_or(true, |&flags| !flags.contains(FdFlags::OCCUPIED))
    }

    fn is_occupied(&self, handle: usize) -> bool {
        self.table
            .get(handle)
            .map_or(false, |&flags| flags.contains(FdFlags::OCCUPIED))
    }

    pub fn reserve(&mut self, additional: usize) {
        self.table.reserve(additional);
    }

    pub fn capacity(&self) -> usize {
        self.table.capacity()
    }

    pub fn len(&self) -> usize {
        self.table
            .iter()
            .filter(|&&e| e.contains(FdFlags::OCCUPIED))
            .count()
    }

    pub fn with_transaction<T, F>(&mut self, rollback_len: usize, f: F) -> Result<T>
    where
        F: FnOnce(&mut Self) -> Result<T>,
    {
        match f(self) {
            Ok(res) => Ok(res),
            Err(e) => {
                self.table.truncate(rollback_len);
                if (self.lowest_idx as usize) > self.table.len() {
                    self.lowest_idx = self.table.len() as u32;
                }
                Err(e)
            }
        }
    }

    fn update_lowest_idx(&mut self, start_from: usize) {
        let mut next_lowest = start_from;
        while next_lowest < self.table.len() && self.is_occupied(next_lowest) {
            next_lowest += 1;
        }
        self.lowest_idx = next_lowest as u32;
    }

    fn validate_handles(&self, handles: &[usize]) -> Result<()> {
        let mut checked_handles = BTreeSet::new();
        for &handle in handles {
            if handle >= FdTbl::CONTEXT_MAX_FILES as usize {
                return Err(Error::new(EMFILE));
            }
            if !checked_handles.insert(handle) || !self.is_occupied(handle) {
                return Err(Error::new(EBADF));
            }
        }
        Ok(())
    }

    fn validate_free_slots(&self, handles: &[usize]) -> Result<()> {
        let mut checked_handles = BTreeSet::new();
        for &handle in handles {
            if handle >= FdTbl::CONTEXT_MAX_FILES as usize {
                return Err(Error::new(EMFILE));
            }
            if !checked_handles.insert(handle) {
                return Err(Error::new(EBADF));
            }
            if self.is_occupied(handle) {
                return Err(Error::new(EEXIST));
            }
        }
        Ok(())
    }

    pub fn find_free_posix_slots(&self, count: usize) -> Vec<usize> {
        let mut free_slots = Vec::with_capacity(count);

        for i in (self.lowest_idx as usize)..self.table.len() {
            if self.is_vacant(i) {
                free_slots.push(i);
                if free_slots.len() == count {
                    return free_slots;
                }
            }
        }

        let mut current_len = self.table.len();
        while free_slots.len() < count {
            free_slots.push(current_len);
            current_len += 1;
        }
        free_slots
    }

    pub fn add(&mut self, flags: FdFlags) -> Result<usize> {
        let handle = self.lowest_idx as usize;
        let old_len = self.table.len();
        let entry_flags = flags | FdFlags::OCCUPIED;

        if handle >= old_len {
            self.with_transaction(old_len, |this| {
                this.table.push(entry_flags);
                this.lowest_idx = (handle + 1) as u32;
                Ok(handle)
            })
        } else {
            self.table[handle] = entry_flags;
            self.update_lowest_idx(handle + 1);
            Ok(handle)
        }
    }

    pub fn bulk_add_posix(&mut self, entries: Vec<FdFlags>) -> Result<Vec<usize>> {
        let count = entries.len();
        if count == 0 {
            return Ok(Vec::new());
        }

        let handles = self.find_free_posix_slots(count);
        let max_index = handles[count - 1];

        if max_index >= FdTbl::CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }

        let old_len = self.table.len();

        self.with_transaction(old_len, |this| {
            if old_len <= max_index {
                this.table.resize(max_index + 1, FdFlags::VACANT);
            }

            for (&handle, flags) in handles.iter().zip(entries) {
                this.table[handle] = flags | FdFlags::OCCUPIED;
            }

            let mut next_lowest = this.lowest_idx as usize;
            while next_lowest < this.table.len() && this.is_occupied(next_lowest) {
                next_lowest += 1;
            }
            this.lowest_idx = next_lowest as u32;

            Ok(handles)
        })
    }

    pub fn insert_at(&mut self, handle: usize, flags: FdFlags) -> Result<usize> {
        if handle >= FdTbl::CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }
        let old_len = self.table.len();
        let entry_flags = flags | FdFlags::OCCUPIED;

        self.with_transaction(old_len, |this| {
            if handle >= old_len {
                this.table.resize(handle + 1, FdFlags::VACANT);
            }
            this.table[handle] = entry_flags;

            if handle <= this.lowest_idx as usize {
                this.update_lowest_idx(handle);
            }
            Ok(handle)
        })
    }

    pub fn bulk_insert_manual(&mut self, entries: Vec<FdFlags>, handles: &[usize]) -> Result<()> {
        if handles.len() != entries.len() {
            return Err(Error::new(EINVAL));
        }
        let count = entries.len();
        if count == 0 {
            return Ok(());
        }

        self.validate_free_slots(handles)?;

        let max_index = handles.iter().max().cloned().unwrap_or(0);
        let old_len = self.table.len();

        self.with_transaction(old_len, |this| {
            if old_len <= max_index {
                this.table.resize(max_index + 1, FdFlags::VACANT);
            }

            for (entry, &index) in entries.into_iter().zip(handles) {
                this.table[index] = entry | FdFlags::OCCUPIED;
            }

            this.update_lowest_idx(0);

            Ok(())
        })
    }

    pub fn remove(&mut self, handle: usize) -> Option<FdFlags> {
        if !self.is_occupied(handle) {
            return None;
        }

        let old_entry = core::mem::replace(&mut self.table[handle], FdFlags::VACANT);

        if (handle as u32) < self.lowest_idx {
            self.lowest_idx = handle as u32;
        }

        Some(old_entry)
    }

    pub fn bulk_remove(&mut self, handles: &[usize]) -> Option<Vec<FdFlags>> {
        self.validate_handles(handles).ok()?;

        let files = handles
            .iter()
            .map(|&i| self.remove(i).expect("fd should exist"))
            .collect();

        Some(files)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FdTblEntry {
    Vacant { next_vacant_idx: u32 },
    Occupied { flag: u32 },
}

impl FdTblEntry {
    pub fn new_occupied(flag: u32) -> Self {
        Self::Occupied { flag }
    }
}

pub struct UpperFdTbl {
    table: Vec<FdTblEntry>,
    len: u32,
    first_vacant_idx: u32,
}

impl UpperFdTbl {
    pub const fn new() -> Self {
        Self {
            table: Vec::new(),
            len: 0,
            first_vacant_idx: FdTbl::CONTEXT_MAX_FILES,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            table: Vec::with_capacity(capacity),
            len: 0,
            first_vacant_idx: FdTbl::CONTEXT_MAX_FILES,
        }
    }

    pub fn get_flags(&self, handle: usize) -> Result<u32> {
        let index = Self::strip_tags(handle);
        match self.table.get(index) {
            Some(FdTblEntry::Occupied { flag }) => Ok(*flag),
            _ => Err(Error::new(EBADF)),
        }
    }

    pub fn set_flags(&mut self, handle: usize, new_flag: u32) -> Result<()> {
        let index = Self::strip_tags(handle);
        match self.table.get_mut(index) {
            Some(entry @ FdTblEntry::Occupied { .. }) => {
                *entry = FdTblEntry::Occupied { flag: new_flag };
                Ok(())
            }
            _ => Err(Error::new(EBADF)),
        }
    }

    pub fn flags_into_entry(flags: usize) -> FdTblEntry {
        FdTblEntry::new_occupied(flags as u32)
    }

    fn strip_tags(index: usize) -> usize {
        index & !syscall::UPPER_FDTBL_TAG
    }

    pub fn reserve(&mut self, additional: usize) {
        self.table.reserve(additional);
    }

    pub fn capacity(&self) -> usize {
        self.table.capacity()
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn with_transaction<T, F>(&mut self, rollback_len: usize, f: F) -> Result<T>
    where
        F: FnOnce(&mut Self) -> Result<T>,
    {
        match f(self) {
            Ok(res) => Ok(res),
            Err(e) => {
                self.table.truncate(rollback_len);
                Err(e)
            }
        }
    }

    fn validate_handles(&self, handles: &[usize]) -> Result<()> {
        let mut checked_handles = BTreeSet::new();
        for &handle in handles {
            let handle = Self::strip_tags(handle);
            if Self::strip_tags(handle) >= FdTbl::CONTEXT_MAX_FILES as usize {
                return Err(Error::new(EMFILE));
            }
            if !checked_handles.insert(handle) {
                return Err(Error::new(EBADF)); // Duplicate handle
            }
            if !matches!(self.table.get(handle), Some(FdTblEntry::Occupied { .. })) {
                return Err(Error::new(EBADF));
            }
        }
        Ok(())
    }

    fn validate_free_slots(&self, handles: &[usize]) -> Result<()> {
        let mut checked_handles = BTreeSet::new();
        for &handle in handles {
            let handle = Self::strip_tags(handle);
            if handle >= FdTbl::CONTEXT_MAX_FILES as usize {
                return Err(Error::new(EMFILE));
            }
            if !checked_handles.insert(handle) {
                return Err(Error::new(EBADF)); // Duplicate handle
            }
            if matches!(self.table.get(handle), Some(FdTblEntry::Occupied { .. })) {
                return Err(Error::new(EEXIST));
            }
        }
        Ok(())
    }

    fn find_free_block(&mut self, len: usize) -> usize {
        let mut start = 0;
        let mut count = 0;

        for (i, entry) in self.table.iter().enumerate() {
            if matches!(entry, FdTblEntry::Vacant { .. }) {
                if count == 0 {
                    start = i;
                }
                count += 1;
                if count == len {
                    break;
                }
            } else {
                count = 0;
            }
        }

        if count < len {
            if count == 0 {
                start = self.table.len();
            }
            let needed = len - count;
            self.table.resize(
                self.table.len() + needed,
                FdTblEntry::Vacant {
                    next_vacant_idx: FdTbl::CONTEXT_MAX_FILES,
                },
            );
        }
        start
    }

    pub fn insert(&mut self, entry: FdTblEntry) -> Result<usize> {
        let handle = self.first_vacant_idx as usize;

        if self.first_vacant_idx == FdTbl::CONTEXT_MAX_FILES {
            let old_len = self.table.len();
            self.with_transaction(old_len, |this| {
                let handle = this.table.len();
                this.table.push(entry);
                this.len += 1;
                Ok(handle)
            })
        } else {
            if let FdTblEntry::Vacant { next_vacant_idx } = self.table[handle] {
                self.first_vacant_idx = next_vacant_idx;
                self.table[handle] = entry;
                self.len += 1;
                Ok(handle)
            } else {
                unreachable!();
            }
        }
    }

    pub fn bulk_insert(&mut self, entries: Vec<FdTblEntry>) -> Result<Vec<usize>> {
        let count = entries.len();
        if count == 0 {
            return Ok(Vec::new());
        }
        if self.len() + count > FdTbl::CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }

        let old_len = self.table.len();

        self.with_transaction(old_len, |this| {
            let start_index = this.find_free_block(count);

            let mut handles = Vec::with_capacity(count);
            for (i, entry) in entries.into_iter().enumerate() {
                let current_index = start_index + i;
                this.table[current_index] = entry;
                handles.push(current_index);
            }

            this.len += count as u32;
            this.rebuild_free_list();

            Ok(handles)
        })
    }

    pub fn insert_at(&mut self, handle: usize, entry: FdTblEntry) -> Result<usize> {
        let old_len = self.table.len();

        self.with_transaction(old_len, |this| {
            if handle >= old_len {
                this.table.resize(
                    handle + 1,
                    FdTblEntry::Vacant {
                        next_vacant_idx: FdTbl::CONTEXT_MAX_FILES,
                    },
                );
            }

            if matches!(this.table[handle], FdTblEntry::Vacant { .. }) {
                this.len += 1;
            }

            this.table[handle] = entry;

            this.rebuild_free_list();
            Ok(handle)
        })
    }

    pub fn bulk_insert_manual(
        &mut self,
        entries: Vec<FdTblEntry>,
        handles: &[usize],
    ) -> Result<()> {
        if handles.len() != entries.len() {
            return Err(Error::new(EINVAL));
        }
        let count = entries.len();
        if count == 0 {
            return Ok(());
        }
        if self.len() + count > FdTbl::CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }

        self.validate_free_slots(handles)?;

        let max_index = handles
            .iter()
            .map(|&h| Self::strip_tags(h))
            .max()
            .unwrap_or(0);
        let old_len = self.table.len();

        self.with_transaction(old_len, |this| {
            if old_len <= max_index {
                this.table.resize(
                    max_index + 1,
                    FdTblEntry::Vacant {
                        next_vacant_idx: FdTbl::CONTEXT_MAX_FILES,
                    },
                );
            }

            for (entry, &index) in entries.into_iter().zip(handles) {
                this.table[Self::strip_tags(index)] = entry;
            }

            this.len += count as u32;
            this.rebuild_free_list();

            Ok(())
        })
    }

    pub fn remove(&mut self, handle: usize) -> Option<FdTblEntry> {
        if handle >= self.table.len() || matches!(self.table[handle], FdTblEntry::Vacant { .. }) {
            return None;
        }

        let old_entry = core::mem::replace(
            &mut self.table[handle],
            FdTblEntry::Vacant {
                next_vacant_idx: self.first_vacant_idx,
            },
        );
        self.first_vacant_idx = handle as u32;
        self.len -= 1;

        Some(old_entry)
    }

    pub fn bulk_remove(&mut self, handles: &[usize]) -> Option<Vec<FdTblEntry>> {
        self.validate_handles(handles).ok()?;

        let files = handles
            .iter()
            .map(|&i| self.remove(i).expect("fd should exist"))
            .collect();

        Some(files)
    }

    fn rebuild_free_list(&mut self) {
        let mut next_vacant = FdTbl::CONTEXT_MAX_FILES;
        for i in (0..self.table.len()).rev() {
            if let FdTblEntry::Vacant { next_vacant_idx } = &mut self.table[i] {
                *next_vacant_idx = next_vacant;
                next_vacant = i as u32;
            }
        }
        self.first_vacant_idx = next_vacant;
    }
}

pub struct FdTblIter<'a> {
    fdtbl: &'a FdTbl,
    stage: u8,
    cursor: usize,
}

impl<'a> FdTblIter<'a> {
    fn new(fdtbl: &'a FdTbl) -> Self {
        Self {
            fdtbl,
            stage: 0,
            cursor: 0,
        }
    }
}

impl<'a> Iterator for FdTblIter<'a> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.stage {
                0 => {
                    let table = &self.fdtbl.posix_fdtbl.table;
                    if self.cursor < table.len() {
                        let idx = self.cursor;
                        self.cursor += 1;

                        let internal_flags = table[idx];
                        if internal_flags.contains(FdFlags::OCCUPIED) {
                            let mut raw_flags = 0;
                            if internal_flags.contains(FdFlags::CLOEXEC) {
                                raw_flags |= syscall::O_CLOEXEC;
                            }
                            /*
                            if internal_flags.contains(FdFlags::CLOFORK) {
                                raw_flags |= syscall::O_CLOFORK;
                            }
                            */
                            return Some((idx, raw_flags));
                        }
                    } else {
                        self.stage = 1;
                        self.cursor = 0;
                    }
                }
                1 => {
                    let table = &self.fdtbl.upper_fdtbl.table;
                    if self.cursor < table.len() {
                        let idx = self.cursor;
                        self.cursor += 1;

                        if let FdTblEntry::Occupied { flag } = table[idx] {
                            let raw_flags = flag as usize;

                            let full_fd = idx | syscall::UPPER_FDTBL_TAG;
                            return Some((full_fd, raw_flags));
                        }
                    } else {
                        self.stage = 2;
                    }
                }
                _ => return None,
            }
        }
    }
}

impl FdTbl {
    pub fn iter(&self) -> FdTblIter<'_> {
        FdTblIter::new(self)
    }
}

impl<'a> IntoIterator for &'a FdTbl {
    type Item = (usize, usize);
    type IntoIter = FdTblIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
