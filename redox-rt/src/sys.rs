use syscall::error::{Result, Error, EINTR};

use crate::arch::manually_enter_trampoline;
use crate::signal::tmp_disable_signals;
use crate::Tcb;

#[inline]
fn wrapper(mut f: impl FnMut() -> Result<usize>) -> Result<usize> {
    loop {
        let _guard = tmp_disable_signals();
        let rt_sigarea = unsafe { &Tcb::current().unwrap().os_specific };
        let res = f();

        if res == Err(Error::new(EINTR)) {
            unsafe {
                manually_enter_trampoline();
            }
            if unsafe { (*rt_sigarea.arch.get()).last_sig_was_restart } {
                continue;
            }
        }

        return res;
    }
}

// TODO: uninitialized memory?
#[inline]
pub fn posix_read(fd: usize, buf: &mut [u8]) -> Result<usize> {
    wrapper(|| syscall::read(fd, buf))
}
#[inline]
pub fn posix_write(fd: usize, buf: &[u8]) -> Result<usize> {
    wrapper(|| syscall::write(fd, buf))
}
#[inline]
pub fn posix_kill(pid: usize, sig: usize) -> Result<()> {
    match wrapper(|| syscall::kill(pid, sig)) {
        Ok(_) | Err(Error { errno: EINTR }) => Ok(()),
        Err(error) => Err(error),
    }
}
#[inline]
pub fn posix_killpg(pgrp: usize, sig: usize) -> Result<()> {
    match wrapper(|| syscall::kill(usize::wrapping_neg(pgrp), sig)) {
        Ok(_) | Err(Error { errno: EINTR }) => Ok(()),
        Err(error) => Err(error),
    }
}
