use core::{
    mem::size_of,
    ptr::{addr_of, null_mut},
    sync::atomic::{AtomicU32, Ordering},
};

use ioslice::IoSlice;
use syscall::{
    Call, CallFlags, EINVAL, ERESTART, StdFsCallKind, TimeSpec,
    data::StdFsCallMeta,
    error::{
        self, EAGAIN, EBADF, EEXIST, EINTR, EMFILE, ENODEV, ENOMEM, EPERM, ESRCH, Error, Result,
    },
};

pub use redox_path::RedoxPath;
use redox_protocols::protocol::{
    F_DUPFD_CLOEXEC, NsDup, O_CLOEXEC, ProcCall, ProcKillTarget, RtSigInfo, ThreadCall, WaitFlags,
};

use crate::{
    DYNAMIC_PROC_INFO, DynamicProcInfo, FILETABLE, RtTcb, Tcb,
    arch::manually_enter_trampoline,
    proc::{FdGuard, FdGuardUpper},
    read_proc_meta,
    signal::tmp_disable_signals,
};
use alloc::{boxed::Box, collections::btree_set::BTreeSet, vec::Vec};

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
        this_proc_call_wo(
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
    this_proc_call(CallFlags::empty(), &[ProcCall::Getppid as u64]).expect("cannot fail") as u32
}

#[inline]
pub fn posix_setpriority(which: i32, who: u32, prio: u32) -> Result<(), syscall::Error> {
    if which != 0 {
        return Err(syscall::Error::new(syscall::EINVAL)); // TODO: Add support for PRIO_PGRP and PRIO_PROCESS
    }

    this_proc_call(
        CallFlags::empty(),
        &[
            ProcCall::SetProcPriority as u64,
            u64::from(who),
            u64::from(prio),
        ],
    )?;

    Ok(())
}

#[inline]
pub fn posix_getpriority(which: i32, who: u32) -> Result<u32, syscall::Error> {
    if which != 0 {
        return Err(syscall::Error::new(syscall::EINVAL));
    }

    let res = this_proc_call(
        CallFlags::empty(),
        &[ProcCall::GetProcPriority as u64, u64::from(who)],
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
                deadline.map_or(0, |d| core::ptr::from_ref(d) as usize),
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

    if !payload.len().is_multiple_of(size_of::<usize>()) {
        return Err(Error::new(EINVAL));
    }

    let fd_slice = unsafe {
        core::slice::from_raw_parts_mut(
            payload.as_mut_ptr().cast::<usize>(),
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
        O_CLOEXEC
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

    if !payload.len().is_multiple_of(size_of::<usize>()) {
        return Err(Error::new(EINVAL));
    }
    let fd_slice = unsafe {
        core::slice::from_raw_parts(
            payload.as_ptr().cast::<usize>(),
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
pub fn sys_call<T: Call>(fd: T, flags: CallFlags, metadata: &[u64]) -> Result<usize> {
    unsafe { fd.raw_call(core::ptr::null_mut(), 0, flags, metadata) }
}

pub fn this_proc_call(flags: CallFlags, metadata: &[u64]) -> Result<usize> {
    sys_call(crate::current_proc_fd().as_raw_fd(), flags, metadata)
}
pub fn this_proc_call_ro(payload: &mut [u8], flags: CallFlags, metadata: &[u64]) -> Result<usize> {
    sys_call_ro(
        crate::current_proc_fd().as_raw_fd(),
        payload,
        flags,
        metadata,
    )
}
pub fn this_proc_call_wo(payload: &[u8], flags: CallFlags, metadata: &[u64]) -> Result<usize> {
    sys_call_wo(
        crate::current_proc_fd().as_raw_fd(),
        payload,
        flags,
        metadata,
    )
}

pub fn this_thread_call(flags: CallFlags, metadata: &[u64]) -> Result<usize> {
    sys_call(RtTcb::current().thread_fd().as_raw_fd(), flags, metadata)
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
        this_proc_call_ro(
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
        sys_call(
            thread_fd,
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
        sys_call_wo(
            pid,
            &buf,
            CallFlags::empty(),
            &[ProcCall::SetResugid as u64],
        )?;
    } else {
        this_proc_call_wo(&buf, CallFlags::empty(), &[ProcCall::SetResugid as u64])?;
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
    sys_call_ro(
        cap_fd,
        buf,
        CallFlags::empty(),
        &[ProcCall::GetProcCredentials as u64, target_pid as u64],
    )
}
pub fn posix_exit(status: i32) -> ! {
    loop {
        match this_proc_call(
            CallFlags::empty(),
            &[ProcCall::Exit as u64, (status & 0xFF) as u64],
        ) {
            Ok(_) => break,
            // procmgr can sometimes send EAGAIN.
            // cannot disable signals so kernel might send EINTR.
            Err(Error { errno: EINTR } | Error { errno: EAGAIN }) => continue,
            Err(e) => panic!("failed to call proc mgr with Exit: {e}"),
        }
    }
    let _ = syscall::write(1, b"redox-rt: ProcCall::Exit FAILED, abort()ing!\n");
    core::intrinsics::abort();
}
pub fn posix_getpgid(pid: usize) -> Result<usize> {
    this_proc_call(
        CallFlags::empty(),
        &[ProcCall::Setpgid as u64, pid as u64, u64::wrapping_neg(1)],
    )
}
pub fn posix_setpgid(pid: usize, pgid: usize) -> Result<()> {
    if pgid == usize::wrapping_neg(1) {
        return Err(Error::new(EINVAL));
    }
    this_proc_call(
        CallFlags::empty(),
        &[ProcCall::Setpgid as u64, pid as u64, pgid as u64],
    )?;
    Ok(())
}
pub fn posix_getsid(pid: usize) -> Result<usize> {
    this_proc_call(CallFlags::empty(), &[ProcCall::Getsid as u64, pid as u64])
}
pub fn posix_setsid() -> Result<u32> {
    this_proc_call(CallFlags::empty(), &[ProcCall::Setsid as u64])?;
    Ok(posix_getpid())
}
pub fn posix_nanosleep(rqtp: &TimeSpec, rmtp: &mut TimeSpec) -> Result<()> {
    wrapper(false, false, || syscall::nanosleep(rqtp, rmtp))?;
    Ok(())
}
pub fn setns(fd: usize) -> Option<FdGuardUpper> {
    let mut info = DYNAMIC_PROC_INFO.lock();
    let new_fd_guard = FdGuard::new(fd).to_upper().unwrap();
    info.ns_fd.replace(new_fd_guard)
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
    let _siglock = tmp_disable_signals();
    let fcntl_flags = flags & syscall::O_FCNTL_MASK;
    let redox_path = RedoxPath::from_absolute(path.as_ref()).ok_or(Error::new(EINVAL))?;
    let (_, reference) = redox_path.as_parts().ok_or(Error::new(EINVAL))?;
    let root_fd = FdGuard::new(openat_into_upper(
        crate::current_namespace_fd()?,
        path.as_ref(),
        syscall::O_DIRECTORY | O_CLOEXEC,
        0,
    )?);
    openat_into_posix(root_fd.as_raw_fd(), reference.as_ref(), flags, fcntl_flags)
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
        guard.add_posix(flags as u32)?
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
    let _siglock = tmp_disable_signals();
    let fcntl_flags = flags & syscall::O_FCNTL_MASK;
    let redox_path = RedoxPath::from_absolute(path.as_ref()).ok_or(Error::new(EINVAL))?;
    let (_, reference) = redox_path.as_parts().ok_or(Error::new(EINVAL))?;
    let root_fd = FdGuard::new(openat_into_upper(
        crate::current_namespace_fd()?,
        path.as_ref(),
        syscall::O_DIRECTORY | O_CLOEXEC,
        0,
    )?);
    openat_into_upper(root_fd.as_raw_fd(), reference.as_ref(), flags, fcntl_flags)
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
    let _siglock = tmp_disable_signals();
    let path = path.as_ref();
    let fcntl_flags = flags & syscall::O_FCNTL_MASK;
    let redox_path = RedoxPath::from_absolute(path).ok_or(Error::new(EINVAL))?;
    let (_, reference) = redox_path.as_parts().ok_or(Error::new(EINVAL))?;
    let root_fd = FdGuard::new(openat_into_upper(
        crate::current_namespace_fd()?,
        path,
        syscall::O_DIRECTORY | O_CLOEXEC,
        0,
    )?);
    let reference_str = reference.as_ref();
    unsafe {
        syscall::syscall4(
            syscall::SYS_UNLINKAT,
            root_fd.as_raw_fd(),
            reference_str.as_ptr() as usize,
            reference_str.len(),
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
    FdGuard::new(dup_into_upper(crate::current_namespace_fd()?, &buf)?).to_upper()
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
    if cmd == syscall::F_DUPFD || cmd == F_DUPFD_CLOEXEC {
        let _siglock = tmp_disable_signals();

        let cloexec_flag = if cmd == F_DUPFD_CLOEXEC { O_CLOEXEC } else { 0 };

        let out = {
            let mut guard = FILETABLE.lock();
            if arg & syscall::UPPER_FDTBL_TAG != 0 {
                guard.insert_upper(cloexec_flag as u32)? | syscall::UPPER_FDTBL_TAG
            } else {
                guard.add_posix(cloexec_flag as u32)?
            }
        };

        let res = unsafe { syscall::syscall3(syscall::SYS_FCNTL, fd, syscall::F_DUPFD, out) };

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
        res?;

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
        guard.insert_upper(flags as u32)?
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

pub fn dup_into_upper(fd: usize, buf: &[u8]) -> Result<usize> {
    let _siglock = tmp_disable_signals();

    let out_idx = {
        let mut guard = FILETABLE.lock();
        guard.insert_upper(0)?
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

pub fn dup_into_upper_raw(fd: usize, buf: &[u8]) -> Result<usize> {
    let out_idx = {
        let mut guard = FILETABLE.lock();
        guard.insert_upper(0)?
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

    if res.is_ok() || res.err().is_some_and(|e| e.errno == EBADF) {
        let mut guard = FILETABLE.lock();
        let _ = guard.remove(fd);
    }

    res
}

pub fn close_raw(fd: usize) -> Result<usize> {
    let res = unsafe { syscall::syscall1(syscall::SYS_CLOSE, fd) };

    if res.is_ok() || res.err().is_some_and(|e| e.errno == EBADF) {
        let mut guard = FILETABLE.lock();
        let _ = guard.remove(fd);
    }

    res
}

pub const NODE_BITS: usize = 9;
pub const NODE_SIZE: usize = 1 << NODE_BITS;
pub const NODE_MASK: usize = NODE_SIZE - 1;
pub const CONTEXT_MAX_FILES: u32 = 65_536;

// TODO: Move into syscall
bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct CapRights: u64 {
        const NONE     = 0;
        const RESERVED = !0;
    }
}

#[repr(C)]
pub struct FileDescriptor {
    pub flags: u32,
    pub is_occupied: u32, // 0 = Vacant, 1 = Occupied
    pub union_field: VacantOrRights,
}

impl core::fmt::Debug for FileDescriptor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_struct("FileDescriptor");
        d.field("flags", &self.flags);
        d.field("is_occupied", &self.is_occupied);
        if self.is_occupied() {
            unsafe {
                d.field("rights", &self.union_field.rights);
            }
        } else {
            unsafe {
                d.field("next_vacant", &self.union_field.next_vacant);
            }
        }
        d.finish()
    }
}

#[repr(C)]
pub union VacantOrRights {
    pub next_vacant: *mut FileDescriptor,
    pub rights: CapRights,
}

impl core::fmt::Debug for VacantOrRights {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VacantOrRights {{ ... }}")
    }
}

const _: () = {
    assert!(
        core::mem::size_of::<FileDescriptor>() == 16,
        "FileDescriptor size must be exactly 16 bytes for C-ABI stability!"
    );
};

impl FileDescriptor {
    pub fn new_vacant(next_vacant: *mut FileDescriptor) -> Self {
        Self {
            flags: 0,
            is_occupied: 0,
            union_field: VacantOrRights { next_vacant },
        }
    }

    pub fn new_occupied(flags: u32) -> Self {
        Self {
            flags,
            is_occupied: 1,
            union_field: VacantOrRights {
                rights: CapRights::empty(),
            },
        }
    }

    #[inline]
    pub fn is_occupied(&self) -> bool {
        self.is_occupied == 1
    }

    #[inline]
    pub fn set_vacant(&mut self, next_vacant: *mut FileDescriptor) {
        self.flags = 0;
        self.is_occupied = 0;
        self.union_field.next_vacant = next_vacant;
    }

    #[inline]
    pub fn set_occupied(&mut self, flags: u32) {
        self.flags = flags;
        self.is_occupied = 1;
        self.union_field.rights = CapRights::empty();
    }

    #[inline]
    pub unsafe fn get_next_vacant(&self) -> *mut FileDescriptor {
        unsafe { self.union_field.next_vacant }
    }

    #[inline]
    pub unsafe fn get_rights(&self) -> CapRights {
        unsafe { self.union_field.rights }
    }

    #[inline]
    pub unsafe fn limit_rights(&mut self, limit: CapRights) {
        unsafe { self.union_field.rights = self.union_field.rights.intersection(limit) };
    }
}

#[repr(C)]
pub struct LeafNode<const N: usize> {
    pub entries: [*mut FileDescriptor; N],
}

const _: () = {
    assert!(
        core::mem::size_of::<LeafNode<NODE_SIZE>>() == syscall::PAGE_SIZE,
        "LeafNode size must be exactly one page"
    );
};

#[repr(C)]
pub struct InnerNode {
    pub children: [*mut LeafNode<NODE_SIZE>; NODE_SIZE],
}

unsafe impl Send for InnerNode {}
unsafe impl Sync for InnerNode {}

impl InnerNode {
    #[expect(
        clippy::new_without_default,
        reason = "explicit initialization expected for radix tree inner nodes"
    )]
    pub const fn new() -> Self {
        Self {
            children: [null_mut(); NODE_SIZE],
        }
    }
}

pub trait LeafAllocator {
    unsafe fn alloc_leaf(&mut self) -> *mut LeafNode<NODE_SIZE>;
    unsafe fn free_leaf(&mut self, ptr: *mut LeafNode<NODE_SIZE>);
    unsafe fn alloc_fd(&mut self, init: FileDescriptor) -> *mut FileDescriptor;
    unsafe fn free_fd(&mut self, ptr: *mut FileDescriptor);
}

pub struct HeapLeafAllocator;

impl LeafAllocator for HeapLeafAllocator {
    unsafe fn alloc_leaf(&mut self) -> *mut LeafNode<NODE_SIZE> {
        let leaf = Box::new(LeafNode {
            entries: [null_mut(); NODE_SIZE],
        });
        Box::into_raw(leaf)
    }

    unsafe fn free_leaf(&mut self, ptr: *mut LeafNode<NODE_SIZE>) {
        if !ptr.is_null() {
            unsafe {
                let leaf = Box::from_raw(ptr);
                for &cap_ptr in leaf.entries.iter() {
                    if !cap_ptr.is_null() {
                        let _ = Box::from_raw(cap_ptr);
                    }
                }
            }
        }
    }
    unsafe fn alloc_fd(&mut self, fd: FileDescriptor) -> *mut FileDescriptor {
        Box::into_raw(Box::new(fd))
    }

    unsafe fn free_fd(&mut self, ptr: *mut FileDescriptor) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }
}

pub struct RadixFdTbl {
    root: InnerNode,
}

impl RadixFdTbl {
    #[expect(
        clippy::new_without_default,
        reason = "explicit initialization expected for internal radix table"
    )]
    pub const fn new() -> Self {
        Self {
            root: InnerNode::new(),
        }
    }

    pub fn get(&self, handle: usize) -> Option<&FileDescriptor> {
        let l1_idx = (handle >> NODE_BITS) & NODE_MASK;
        let l0_idx = handle & NODE_MASK;

        let leaf_ptr = self.root.children[l1_idx];
        if leaf_ptr.is_null() {
            None
        } else {
            unsafe {
                let cap_ptr = (*leaf_ptr).entries[l0_idx];
                if cap_ptr.is_null() {
                    None
                } else {
                    let fd = &*cap_ptr;
                    if fd.is_occupied() { Some(fd) } else { None }
                }
            }
        }
    }

    pub fn get_mut(&mut self, handle: usize) -> Option<&mut FileDescriptor> {
        let l1_idx = (handle >> NODE_BITS) & NODE_MASK;
        let l0_idx = handle & NODE_MASK;

        let leaf_ptr = self.root.children[l1_idx];
        if leaf_ptr.is_null() {
            None
        } else {
            unsafe {
                let cap_ptr = (*leaf_ptr).entries[l0_idx];
                if cap_ptr.is_null() {
                    None
                } else {
                    let fd = &mut *cap_ptr;
                    if fd.is_occupied() { Some(fd) } else { None }
                }
            }
        }
    }

    pub fn get_or_create_entry<A: LeafAllocator>(
        &mut self,
        handle: usize,
        alloc: &mut A,
    ) -> Result<&mut FileDescriptor> {
        let l1_idx = (handle >> NODE_BITS) & NODE_MASK;
        let l0_idx = handle & NODE_MASK;

        let mut leaf_ptr = self.root.children[l1_idx];
        if leaf_ptr.is_null() {
            leaf_ptr = unsafe { alloc.alloc_leaf() };
            if leaf_ptr.is_null() {
                return Err(Error::new(ENOMEM));
            }
            self.root.children[l1_idx] = leaf_ptr;
        }

        unsafe {
            let slot = &mut (*leaf_ptr).entries[l0_idx];
            if slot.is_null() {
                let new_fd = alloc.alloc_fd(FileDescriptor::new_vacant(null_mut()));
                if new_fd.is_null() {
                    return Err(Error::new(ENOMEM));
                }
                *slot = new_fd;
            }
            Ok(&mut **slot)
        }
    }

    pub fn is_occupied(&self, handle: usize) -> bool {
        self.get(handle).is_some()
    }
}

pub struct PosixFdTbl {
    table: RadixFdTbl,
    lowest_idx: u32,
    active_count: usize,
}

impl PosixFdTbl {
    #[expect(
        clippy::new_without_default,
        reason = "explicit initialization expected for posix file table"
    )]
    pub const fn new() -> Self {
        Self {
            table: RadixFdTbl::new(),
            lowest_idx: 0,
            active_count: 0,
        }
    }

    pub fn get_flags(&self, handle: usize) -> Result<u32> {
        let entry = self.table.get(handle).ok_or(Error::new(EBADF))?;
        Ok(entry.flags)
    }

    pub fn set_flags(&mut self, handle: usize, flags: u32) -> Result<()> {
        let entry = self.table.get_mut(handle).ok_or(Error::new(EBADF))?;
        entry.flags = flags;
        Ok(())
    }

    pub fn get_rights(&self, index: usize) -> Result<CapRights> {
        let entry = self.table.get(index).ok_or(Error::new(EBADF))?;
        unsafe { Ok(entry.get_rights()) }
    }

    pub fn limit_rights(&mut self, index: usize, limit: CapRights) -> Result<()> {
        let entry = self.table.get_mut(index).ok_or(Error::new(EBADF))?;
        unsafe { entry.limit_rights(limit) };
        Ok(())
    }

    pub fn check_rights(&self, handle: usize, required: CapRights) -> Result<()> {
        let rights = self.get_rights(handle)?;
        if rights.contains(required) {
            Ok(())
        } else {
            Err(Error::new(EPERM))
        }
    }

    #[inline]
    fn is_occupied(&self, handle: usize) -> bool {
        self.table.is_occupied(handle)
    }

    pub fn len(&self) -> usize {
        self.active_count
    }

    pub const fn is_empty(&self) -> bool {
        self.active_count == 0
    }

    fn update_lowest_idx(&mut self, start_from: usize) {
        let mut next_lowest = start_from;
        while next_lowest < CONTEXT_MAX_FILES as usize && self.is_occupied(next_lowest) {
            next_lowest += 1;
        }
        self.lowest_idx = next_lowest as u32;
    }

    fn validate_handles(&self, handles: &[usize]) -> Result<()> {
        let mut checked_handles = BTreeSet::new();
        for &handle in handles {
            if handle >= CONTEXT_MAX_FILES as usize {
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
            if handle >= CONTEXT_MAX_FILES as usize {
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

        for i in (self.lowest_idx as usize)..(CONTEXT_MAX_FILES as usize) {
            if !self.is_occupied(i) {
                free_slots.push(i);
                if free_slots.len() == count {
                    return free_slots;
                }
            }
        }

        for i in 0..(self.lowest_idx as usize) {
            if !self.is_occupied(i) {
                free_slots.push(i);
                if free_slots.len() == count {
                    return free_slots;
                }
            }
        }

        free_slots
    }

    pub fn add<A: LeafAllocator>(
        &mut self,
        flags: u32,
        sync_fd: Option<&FdGuardUpper>,
        alloc: &mut A,
    ) -> Result<usize> {
        let handle = self.lowest_idx as usize;
        if handle >= CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }

        FdTbl::<A>::sync_size(sync_fd, handle + 1, 0)?;

        let entry = self.table.get_or_create_entry(handle, alloc)?;
        entry.set_occupied(flags);

        self.active_count += 1;
        self.update_lowest_idx(handle + 1);
        Ok(handle)
    }

    pub fn bulk_add_posix<A: LeafAllocator>(
        &mut self,
        entries: Vec<usize>,
        sync_fd: Option<&FdGuardUpper>,
        alloc: &mut A,
    ) -> Result<Vec<usize>> {
        let count = entries.len();
        if count == 0 {
            return Ok(Vec::new());
        }

        let handles = self.find_free_posix_slots(count);
        if handles.len() < count {
            return Err(Error::new(EMFILE));
        }

        let max_index = *handles.iter().max().unwrap();
        FdTbl::<A>::sync_size(sync_fd, max_index + 1, 0)?;

        for (&handle, flags) in handles.iter().zip(entries) {
            let entry = self.table.get_or_create_entry(handle, alloc)?;
            entry.set_occupied(flags as u32);
        }

        self.active_count += count;

        self.update_lowest_idx(self.lowest_idx as usize);

        Ok(handles)
    }

    pub fn insert_at<A: LeafAllocator>(
        &mut self,
        handle: usize,
        flags: u32,
        sync_fd: Option<&FdGuardUpper>,
        alloc: &mut A,
    ) -> Result<usize> {
        if handle >= CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }

        FdTbl::<A>::sync_size(sync_fd, handle + 1, 0)?;

        let entry = self.table.get_or_create_entry(handle, alloc)?;
        let was_occupied = entry.is_occupied();

        entry.set_occupied(flags);

        if !was_occupied {
            self.active_count += 1;
        }

        if (handle as u32) <= self.lowest_idx {
            self.update_lowest_idx(handle);
        }

        Ok(handle)
    }

    pub fn bulk_insert_manual<A: LeafAllocator>(
        &mut self,
        entries: Vec<u32>,
        handles: &[usize],
        sync_fd: Option<&FdGuardUpper>,
        alloc: &mut A,
    ) -> Result<()> {
        if handles.len() != entries.len() {
            return Err(Error::new(EINVAL));
        }
        let count = entries.len();
        if count == 0 {
            return Ok(());
        }

        self.validate_free_slots(handles)?;

        let max_index = handles.iter().max().cloned().unwrap_or(0);
        FdTbl::<A>::sync_size(sync_fd, max_index + 1, 0)?;

        for (flags, &handle) in entries.into_iter().zip(handles) {
            let entry = self.table.get_or_create_entry(handle, alloc)?;
            entry.set_occupied(flags);
        }

        self.active_count += count;
        self.update_lowest_idx(0);
        Ok(())
    }

    pub fn remove(&mut self, handle: usize) -> Option<u32> {
        if handle >= CONTEXT_MAX_FILES as usize {
            return None;
        }
        let entry = self.table.get_mut(handle)?;
        if !entry.is_occupied() {
            return None;
        }

        let old_flags = entry.flags;
        entry.set_vacant(null_mut());

        self.active_count -= 1;
        if (handle as u32) < self.lowest_idx {
            self.lowest_idx = handle as u32;
        }

        Some(old_flags)
    }

    pub fn bulk_remove(&mut self, handles: &[usize]) -> Option<Vec<u32>> {
        self.validate_handles(handles).ok()?;
        let files = handles
            .iter()
            .map(|&i| self.remove(i).expect("fd should exist"))
            .collect();
        Some(files)
    }
}

pub struct UpperFdTbl {
    table: RadixFdTbl,
    len: u32,
    next_alloc_idx: usize,
}

unsafe impl Send for UpperFdTbl {}
unsafe impl Sync for UpperFdTbl {}

impl UpperFdTbl {
    #[expect(
        clippy::new_without_default,
        reason = "explicit initialization expected for upper file table"
    )]
    pub const fn new() -> Self {
        Self {
            table: RadixFdTbl::new(),
            len: 0,
            next_alloc_idx: 0,
        }
    }

    pub fn get_flags(&self, handle: usize) -> Result<u32> {
        let index = Self::strip_tags(handle);
        let entry = self.table.get(index).ok_or(Error::new(EBADF))?;
        Ok(entry.flags)
    }

    pub fn set_flags(&mut self, handle: usize, new_flag: u32) -> Result<()> {
        let index = Self::strip_tags(handle);
        let entry = self.table.get_mut(index).ok_or(Error::new(EBADF))?;
        entry.flags = new_flag;
        Ok(())
    }

    pub fn get_rights(&self, handle: usize) -> Result<CapRights> {
        let index = Self::strip_tags(handle);
        let entry = self.table.get(index).ok_or(Error::new(EBADF))?;
        unsafe { Ok(entry.get_rights()) }
    }

    pub fn limit_rights(&mut self, handle: usize, limit: CapRights) -> Result<()> {
        let index = Self::strip_tags(handle);
        let entry = self.table.get_mut(index).ok_or(Error::new(EBADF))?;
        unsafe { entry.limit_rights(limit) };
        Ok(())
    }

    pub fn check_rights(&self, handle: usize, required: CapRights) -> Result<()> {
        let rights = self.get_rights(handle)?;
        if rights.contains(required) {
            Ok(())
        } else {
            Err(Error::new(EPERM))
        }
    }

    fn strip_tags(index: usize) -> usize {
        index & !syscall::UPPER_FDTBL_TAG
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    fn is_occupied(&self, handle: usize) -> bool {
        self.table.get(handle).is_some_and(|e| e.is_occupied())
    }

    fn validate_handles(&self, handles: &[usize]) -> Result<()> {
        let mut checked_handles = BTreeSet::new();
        for &handle in handles {
            let handle = Self::strip_tags(handle);
            if handle >= CONTEXT_MAX_FILES as usize {
                return Err(Error::new(EMFILE));
            }
            if !checked_handles.insert(handle) {
                return Err(Error::new(EBADF)); // Duplicate handle
            }
            let entry = self.table.get(handle).ok_or(Error::new(EBADF))?;
            if !entry.is_occupied() {
                return Err(Error::new(EBADF));
            }
        }
        Ok(())
    }

    fn validate_free_slots(&self, handles: &[usize]) -> Result<()> {
        let mut checked_handles = BTreeSet::new();
        for &handle in handles {
            let handle = Self::strip_tags(handle);
            if handle >= CONTEXT_MAX_FILES as usize {
                return Err(Error::new(EMFILE));
            }
            if !checked_handles.insert(handle) {
                return Err(Error::new(EBADF)); // Duplicate handle
            }
            if self.is_occupied(handle) {
                return Err(Error::new(EEXIST));
            }
        }
        Ok(())
    }

    fn find_free_block(&self, len: usize) -> usize {
        let mut start = 0;
        let mut count = 0;

        for i in 0..(CONTEXT_MAX_FILES as usize) {
            if !self.is_occupied(i) {
                if count == 0 {
                    start = i;
                }
                count += 1;
                if count == len {
                    return start;
                }
            } else {
                count = 0;
            }
        }

        CONTEXT_MAX_FILES as usize
    }

    fn find_free_slot(&mut self) -> Option<usize> {
        for i in self.next_alloc_idx..(CONTEXT_MAX_FILES as usize) {
            if !self.is_occupied(i) {
                self.next_alloc_idx = i + 1;
                return Some(i);
            }
        }
        for i in 0..self.next_alloc_idx {
            if !self.is_occupied(i) {
                self.next_alloc_idx = i + 1;
                return Some(i);
            }
        }
        None
    }

    pub fn insert<A: LeafAllocator>(
        &mut self,
        flags: u32,
        sync_fd: Option<&FdGuardUpper>,
        alloc: &mut A,
    ) -> Result<usize> {
        let handle = self.find_free_slot().ok_or(Error::new(EMFILE))?;

        FdTbl::<A>::sync_size(sync_fd, handle + 1, syscall::UPPER_FDTBL_TAG)?;

        let entry = self.table.get_or_create_entry(handle, alloc)?;
        entry.set_occupied(flags);

        self.len += 1;
        Ok(handle)
    }

    pub fn bulk_insert<A: LeafAllocator>(
        &mut self,
        entries: Vec<usize>,
        sync_fd: Option<&FdGuardUpper>,
        alloc: &mut A,
    ) -> Result<Vec<usize>> {
        let count = entries.len();
        if count == 0 {
            return Ok(Vec::new());
        }
        if self.len() + count > CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }

        let start_index = self.find_free_block(count);
        if start_index + count > CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }

        FdTbl::<A>::sync_size(sync_fd, start_index + count, syscall::UPPER_FDTBL_TAG)?;

        let mut handles = Vec::with_capacity(count);

        for (i, flags) in entries.into_iter().enumerate() {
            let handle = start_index + i;
            let entry = self.table.get_or_create_entry(handle, alloc)?;

            entry.set_occupied(flags as u32);
            handles.push(handle);
        }

        self.len += count as u32;
        Ok(handles)
    }

    pub fn insert_at<A: LeafAllocator>(
        &mut self,
        handle: usize,
        flags: u32,
        sync_fd: Option<&FdGuardUpper>,
        alloc: &mut A,
    ) -> Result<usize> {
        if handle >= CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }

        FdTbl::<A>::sync_size(sync_fd, handle + 1, syscall::UPPER_FDTBL_TAG)?;

        let entry = self.table.get_or_create_entry(handle, alloc)?;
        let was_occupied = entry.is_occupied();

        entry.set_occupied(flags);

        if !was_occupied {
            self.len += 1;
        }

        if handle == self.next_alloc_idx {
            let mut next = handle + 1;
            while next < CONTEXT_MAX_FILES as usize && self.is_occupied(next) {
                next += 1;
            }
            self.next_alloc_idx = next;
        }

        Ok(handle)
    }

    pub fn bulk_insert_manual<A: LeafAllocator>(
        &mut self,
        entries: Vec<u32>,
        handles: &[usize],
        sync_fd: Option<&FdGuardUpper>,
        alloc: &mut A,
    ) -> Result<()> {
        if handles.len() != entries.len() {
            return Err(Error::new(EINVAL));
        }
        let count = entries.len();
        if count == 0 {
            return Ok(());
        }
        if self.len() + count > CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }

        self.validate_free_slots(handles)?;

        let max_index = handles
            .iter()
            .map(|&h| Self::strip_tags(h))
            .max()
            .unwrap_or(0);
        FdTbl::<A>::sync_size(sync_fd, max_index + 1, syscall::UPPER_FDTBL_TAG)?;

        for (flags, &raw_handle) in entries.into_iter().zip(handles) {
            let handle = Self::strip_tags(raw_handle);
            let entry = self.table.get_or_create_entry(handle, alloc)?;

            entry.set_occupied(flags);
        }

        self.len += count as u32;
        Ok(())
    }

    pub fn remove(&mut self, handle: usize) -> Option<(u32, CapRights)> {
        let handle = Self::strip_tags(handle);
        if handle >= CONTEXT_MAX_FILES as usize {
            return None;
        }
        let entry = self.table.get_mut(handle)?;

        if !entry.is_occupied() {
            return None;
        }

        let old_flags = entry.flags;
        let old_rights = unsafe { entry.get_rights() };

        entry.set_vacant(null_mut());

        if handle < self.next_alloc_idx {
            self.next_alloc_idx = handle;
        }

        self.len -= 1;
        Some((old_flags, old_rights))
    }

    pub fn bulk_remove(&mut self, handles: &[usize]) -> Option<Vec<(u32, CapRights)>> {
        self.validate_handles(handles).ok()?;
        let files = handles
            .iter()
            .map(|&i| self.remove(i).expect("fd should exist"))
            .collect();
        Some(files)
    }
}

pub struct FdTbl<A: LeafAllocator = HeapLeafAllocator> {
    fd: Option<FdGuardUpper>,
    posix_fdtbl: PosixFdTbl,
    upper_fdtbl: UpperFdTbl,
    active_count: usize,
    allocator: A,
}

impl FdTbl<HeapLeafAllocator> {
    #[expect(
        clippy::new_without_default,
        reason = "explicit initialization expected for runtime file table"
    )]
    pub const fn new() -> Self {
        Self {
            fd: None,
            posix_fdtbl: PosixFdTbl::new(),
            upper_fdtbl: UpperFdTbl::new(),
            active_count: 0,
            allocator: HeapLeafAllocator,
        }
    }

    pub fn from_binary_fd(filetable_fd: FdGuardUpper) -> Result<Self> {
        let mut fdtbl = Self::new();
        let files_reader_fd = filetable_fd.as_raw_fd();
        let buf = b"refresh";
        unsafe {
            syscall::syscall4(
                syscall::SYS_DUP2,
                files_reader_fd,
                files_reader_fd,
                buf.as_ptr() as usize,
                buf.len(),
            )
        }?;

        let mut reader = crate::proc::FileBufReader::from_fd(files_reader_fd);
        fdtbl.populate(&mut reader)?;

        // Manually mark the filetable_fd itself as occupied in userspace FILETABLE
        fdtbl.override_at(files_reader_fd, files_reader_fd)?;
        fdtbl.set_fd(filetable_fd);

        Ok(fdtbl)
    }
}

impl<A: LeafAllocator> FdTbl<A> {
    pub fn fd(&self) -> Option<&FdGuardUpper> {
        self.fd.as_ref()
    }

    pub fn take(&mut self) -> Option<FdGuardUpper> {
        self.fd.take()
    }

    pub fn set_fd(&mut self, fd: FdGuardUpper) {
        self.fd = Some(fd);
    }

    pub fn upper_len(&self) -> usize {
        self.upper_fdtbl.len()
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
            Ok((flags & O_CLOEXEC as u32) as usize)
        } else {
            let flags = self.posix_fdtbl.get_flags(fd)?;
            Ok((flags & O_CLOEXEC as u32) as usize)
        }
    }

    pub fn set_fd_flags(&mut self, fd: usize, flags: usize) -> Result<()> {
        if Self::is_upper(fd) {
            let old_flags = self.upper_fdtbl.get_flags(fd)?;
            let mut new_flags = old_flags & !(O_CLOEXEC as u32);
            if flags & O_CLOEXEC != 0 {
                new_flags |= O_CLOEXEC as u32;
            }
            self.upper_fdtbl.set_flags(fd, new_flags)?;
        } else {
            let old_flags = self.posix_fdtbl.get_flags(fd)?;
            let mut new_flags = old_flags & !(O_CLOEXEC as u32);
            if flags & O_CLOEXEC != 0 {
                new_flags |= O_CLOEXEC as u32;
            }
            self.posix_fdtbl.set_flags(fd, new_flags)?;
        }
        Ok(())
    }

    pub fn get_rights(&self, fd: usize) -> Result<CapRights> {
        if Self::is_upper(fd) {
            self.upper_fdtbl.get_rights(fd)
        } else {
            self.posix_fdtbl.get_rights(fd)
        }
    }

    pub fn limit_rights(&mut self, fd: usize, limit: CapRights) -> Result<()> {
        if Self::is_upper(fd) {
            self.upper_fdtbl.limit_rights(fd, limit)
        } else {
            self.posix_fdtbl.limit_rights(fd, limit)
        }
    }

    pub fn check_rights(&self, fd: usize, required: CapRights) -> Result<()> {
        if Self::is_upper(fd) {
            self.upper_fdtbl.check_rights(fd, required)
        } else {
            self.posix_fdtbl.check_rights(fd, required)
        }
    }

    fn sync_size(fd: Option<&FdGuardUpper>, new_size: usize, tag: usize) -> Result<()> {
        if let Some(fd) = fd {
            let res = fd.call_wo(
                &[],
                CallFlags::empty(),
                &[
                    syscall::FileTableVerb::Resize as u64,
                    tag as u64,
                    new_size as u64,
                ],
            );
            if let Err(err) = res {
                if err.errno == EINVAL {
                    // Ignore EINVAL, because it means the kernel table is already larger than new_size.
                } else {
                    return Err(err);
                }
            }
        }
        Ok(())
    }

    pub fn override_at(&mut self, fd: usize, new_fd: usize) -> Result<usize> {
        let _ = self.remove(new_fd);

        if Self::is_upper(new_fd) {
            let handle = UpperFdTbl::strip_tags(new_fd);
            self.upper_fdtbl
                .insert_at(handle, 0, self.fd.as_ref(), &mut self.allocator)?;
        } else {
            self.posix_fdtbl
                .insert_at(new_fd, 0, self.fd.as_ref(), &mut self.allocator)?;
        }

        self.active_count = self.posix_fdtbl.len() + self.upper_fdtbl.len();

        Ok(new_fd)
    }

    pub fn add_posix(&mut self, flags: u32) -> Result<usize> {
        if self.active_count >= CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }

        let out_idx = self
            .posix_fdtbl
            .add(flags, self.fd.as_ref(), &mut self.allocator)?;
        self.active_count = self.posix_fdtbl.len() + self.upper_fdtbl.len();

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

        if self.active_count + cnt > CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }

        let fd_ref = self.fd.as_ref();
        let alloc = &mut self.allocator;
        let entries = alloc::vec![flags; cnt];

        if !Self::is_upper(which) {
            let allocated_fds = self.posix_fdtbl.bulk_add_posix(entries, fd_ref, alloc)?;
            fd_slice.copy_from_slice(&allocated_fds);
        } else {
            let allocated_fds = self.upper_fdtbl.bulk_insert(entries, fd_ref, alloc)?;
            for (i, &handle) in allocated_fds.iter().enumerate() {
                fd_slice[i] = handle | syscall::UPPER_FDTBL_TAG;
            }
        }

        self.active_count = self.posix_fdtbl.len() + self.upper_fdtbl.len();

        Ok(cnt)
    }

    pub fn insert_upper(&mut self, flags: u32) -> Result<usize> {
        if self.active_count >= CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }

        let out_idx = self
            .upper_fdtbl
            .insert(flags, self.fd.as_ref(), &mut self.allocator)?;
        self.active_count = self.posix_fdtbl.len() + self.upper_fdtbl.len();

        Ok(out_idx)
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

        if self.active_count + cnt > CONTEXT_MAX_FILES as usize {
            return Err(Error::new(EMFILE));
        }

        let entries = alloc::vec![flags as u32; cnt];

        let fd_ref = self.fd.as_ref();
        let alloc = &mut self.allocator;

        if !Self::is_upper(which) {
            self.posix_fdtbl
                .bulk_insert_manual(entries, fd_slice, fd_ref, alloc)?;
        } else {
            self.upper_fdtbl
                .bulk_insert_manual(entries, fd_slice, fd_ref, alloc)?;
        }

        self.active_count = self.posix_fdtbl.len() + self.upper_fdtbl.len();

        Ok(cnt)
    }

    pub fn remove(&mut self, fd: usize) -> Result<()> {
        let removed = if Self::is_upper(fd) {
            let handle = UpperFdTbl::strip_tags(fd);
            self.upper_fdtbl.remove(handle).is_some()
        } else {
            self.posix_fdtbl.remove(fd).is_some()
        };

        if removed {
            self.active_count = self.posix_fdtbl.len() + self.upper_fdtbl.len();
            Ok(())
        } else {
            Err(Error::new(EBADF))
        }
    }
}

pub struct FdTblIter<'a, A: LeafAllocator> {
    fdtbl: &'a FdTbl<A>,
    stage: u8,
    cursor: usize,
}

impl<'a, A: LeafAllocator> FdTblIter<'a, A> {
    fn new(fdtbl: &'a FdTbl<A>) -> Self {
        Self {
            fdtbl,
            stage: 0,
            cursor: 0,
        }
    }
}

impl<'a, A: LeafAllocator> Iterator for FdTblIter<'a, A> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.stage {
                0 => {
                    while self.cursor < CONTEXT_MAX_FILES as usize {
                        let idx = self.cursor;
                        self.cursor += 1;

                        if let Ok(flags) = self.fdtbl.posix_fdtbl.get_flags(idx) {
                            return Some((idx, flags as usize));
                        }
                    }
                    self.stage = 1;
                    self.cursor = 0;
                }
                1 => {
                    while self.cursor < CONTEXT_MAX_FILES as usize {
                        let idx = self.cursor;
                        self.cursor += 1;

                        if let Ok(flags) = self.fdtbl.upper_fdtbl.get_flags(idx) {
                            let full_fd = idx | syscall::UPPER_FDTBL_TAG;
                            return Some((full_fd, flags as usize));
                        }
                    }
                    self.stage = 2;
                }
                _ => return None,
            }
        }
    }
}

impl<'a, A: LeafAllocator> IntoIterator for &'a FdTbl<A> {
    type Item = (usize, usize);
    type IntoIter = FdTblIter<'a, A>;

    fn into_iter(self) -> Self::IntoIter {
        FdTblIter::new(self)
    }
}

impl<A: LeafAllocator> FdTbl<A> {
    pub fn iter(&self) -> FdTblIter<'_, A> {
        FdTblIter::new(self)
    }
}
