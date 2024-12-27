use core::{
    ptr::addr_of,
    sync::atomic::{AtomicU32, Ordering},
};

use syscall::{
    error::{Error, Result, EINTR},
    RtSigInfo, TimeSpec,
};

use crate::{
    arch::manually_enter_trampoline, proc::FdGuard, signal::tmp_disable_signals, Tcb,
    DYNAMIC_PROC_INFO,
};

#[inline]
fn wrapper<T>(restart: bool, mut f: impl FnMut() -> Result<T>) -> Result<T> {
    loop {
        let _guard = tmp_disable_signals();
        let rt_sigarea = unsafe { &Tcb::current().unwrap().os_specific };
        let res = f();

        if let Err(err) = res
            && err == Error::new(EINTR)
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
    wrapper(true, || syscall::read(fd, buf))
}
#[inline]
pub fn posix_write(fd: usize, buf: &[u8]) -> Result<usize> {
    wrapper(true, || syscall::write(fd, buf))
}
#[inline]
pub fn posix_kill(pid: usize, sig: usize) -> Result<()> {
    match wrapper(false, || Ok(todo!("kill"))) {
        Ok(_) | Err(Error { errno: EINTR }) => Ok(()),
        Err(error) => Err(error),
    }
}
#[inline]
pub fn posix_sigqueue(pid: usize, sig: usize, arg: usize) -> Result<()> {
    let siginf = RtSigInfo {
        arg,
        code: -1, // TODO: SI_QUEUE constant
        uid: 0,   // TODO
        pid: posix_getpid(),
    };
    match wrapper(false, || unsafe {
        //syscall::syscall3(syscall::SYS_SIGENQUEUE, pid, sig, addr_of!(siginf) as usize)
        Ok(todo!("sigenqueue"))
    }) {
        Ok(_) | Err(Error { errno: EINTR }) => Ok(()),
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
    // SAFETY: read-only except during program/fork child initialization
    unsafe { addr_of!((*crate::STATIC_PROC_INFO.get()).ppid).read() }
}
#[inline]
pub fn posix_killpg(pgrp: usize, sig: usize) -> Result<()> {
    match wrapper(false, ||
        //syscall::kill(usize::wrapping_neg(pgrp), sig)
        Ok(todo!("killpg")))
    {
        Ok(_) | Err(Error { errno: EINTR }) => Ok(()),
        Err(error) => Err(error),
    }
}
#[inline]
pub unsafe fn sys_futex_wait(addr: *mut u32, val: u32, deadline: Option<&TimeSpec>) -> Result<()> {
    wrapper(true, || {
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
pub fn sys_waitpid(pid: usize, status: &mut usize, flags: usize) -> Result<usize> {
    wrapper(true, || {
        /*syscall::waitpid(
            pid,
            status,
            syscall::WaitFlags::from_bits(flags).expect("waitpid: invalid bit pattern"),
        )*/
        todo!("waitpid")
    })
}
pub fn posix_kill_thread(thread_fd: usize, signal: u32) -> Result<()> {
    let killfd = FdGuard::new(syscall::dup(thread_fd, b"signal")?);
    match wrapper(false, || syscall::write(*killfd, &signal.to_ne_bytes())) {
        Ok(_) | Err(Error { errno: EINTR }) => Ok(()),
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

pub fn posix_setresuid(ruid: Option<u32>, euid: Option<u32>, suid: Option<u32>) -> Result<()> {
    todo!("posix_setresuid")
}
pub fn posix_setresgid(rgid: Option<u32>, egid: Option<u32>, sgid: Option<u32>) -> Result<()> {
    todo!("posix_setresgid")
}
pub fn posix_getruid() -> u32 {
    DYNAMIC_PROC_INFO.lock().ruid
}
pub fn posix_getrgid() -> u32 {
    DYNAMIC_PROC_INFO.lock().rgid
}
pub fn posix_geteuid() -> u32 {
    DYNAMIC_PROC_INFO.lock().euid
}
pub fn posix_getegid() -> u32 {
    DYNAMIC_PROC_INFO.lock().egid
}
pub fn posix_exit(status: i32) -> ! {
    todo!("posix_exit")
}
pub fn setrens(rns: usize, ens: usize) -> Result<()> {
    todo!("setrens")
}
pub fn posix_getpgid(pid: usize) -> Result<usize> {
    todo!("posix_getpgid")
}
pub fn posix_setpgid(pid: usize, pgid: usize) -> Result<()> {
    todo!("posix_setpgid")
}
pub fn posix_getsid(pid: usize) -> Result<usize> {
    todo!("posix_getsid")
}
