use core::{
    mem::size_of,
    ptr::addr_of,
    sync::atomic::{AtomicU32, Ordering},
};

use syscall::{
    error::{self, Error, Result, EINTR},
    CallFlags, TimeSpec, EINVAL, ERESTART,
};

use crate::{
    arch::manually_enter_trampoline,
    protocol::{ProcCall, ProcKillTarget, RtSigInfo, ThreadCall, WaitFlags},
    signal::tmp_disable_signals,
    DynamicProcInfo, RtTcb, Tcb, DYNAMIC_PROC_INFO,
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
            errno: error::EINTR,
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
pub unsafe fn sys_futex_wait(addr: *mut u32, val: u32, deadline: Option<&TimeSpec>) -> Result<()> {
    wrapper(true, false, || {
        syscall::syscall5(
            syscall::SYS_FUTEX,
            addr as usize,
            syscall::FUTEX_WAIT,
            val as usize,
            deadline.map_or(0, |d| d as *const _ as usize),
            0,
        )
        .map(|_| ())
    })
}
#[inline]
pub unsafe fn sys_futex_wake(addr: *mut u32, num: u32) -> Result<u32> {
    syscall::syscall5(
        syscall::SYS_FUTEX,
        addr as usize,
        syscall::FUTEX_WAKE,
        num as usize,
        0,
        0,
    )
    .map(|awoken| awoken as u32)
}
pub fn sys_call(
    fd: usize,
    payload: &mut [u8],
    flags: CallFlags,
    metadata: &[u64],
) -> Result<usize> {
    unsafe {
        syscall::syscall5(
            syscall::SYS_CALL,
            fd,
            payload.as_mut_ptr() as usize,
            payload.len(),
            metadata.len() | flags.bits(),
            metadata.as_ptr() as usize,
        )
    }
}
pub fn this_proc_call(payload: &mut [u8], flags: CallFlags, metadata: &[u64]) -> Result<usize> {
    proc_call(**crate::current_proc_fd(), payload, flags, metadata)
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
    thread_call(**RtTcb::current().thread_fd(), payload, flags, metadata)
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
pub fn posix_setresugid(ids: &Resugid<Option<u32>>) -> Result<()> {
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

    this_proc_call(&mut buf, CallFlags::empty(), &[ProcCall::SetResugid as u64])?;

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
pub fn posix_exit(status: i32) -> ! {
    this_proc_call(
        &mut [],
        CallFlags::empty(),
        &[ProcCall::Exit as u64, status as u64],
    )
    .expect("failed to call proc mgr with Exit");
    let _ = syscall::write(1, b"redox-rt: ProcCall::Exit FAILED, abort()ing!\n");
    core::intrinsics::abort();
}
pub fn setrens(rns: usize, ens: usize) -> Result<()> {
    this_proc_call(
        &mut [],
        CallFlags::empty(),
        &[ProcCall::Setrens as u64, rns as u64, ens as u64],
    )?;
    Ok(())
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
