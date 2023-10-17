use core::mem;

use super::{
    super::{types::*, PalSignal},
    e_raw, Sys,
};
use crate::{
    errno::Errno,
    header::{
        signal::{sigaction, siginfo_t, sigset_t, stack_t, NSIG},
        sys_time::itimerval,
        time::timespec,
    },
};

impl PalSignal for Sys {
    fn getitimer(which: c_int, out: *mut itimerval) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(GETITIMER, which, out) })?;
        Ok(())
    }

    fn kill(pid: pid_t, sig: c_int) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(KILL, pid, sig) })?;
        Ok(())
    }

    fn killpg(pgrp: pid_t, sig: c_int) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(KILL, -(pgrp as isize) as pid_t, sig) })?;
        Ok(())
    }

    fn raise(sig: c_int) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(GETTID) })
            .map(|tid| e_raw(unsafe { syscall!(TKILL, tid, sig) }))?;
        Ok(())
    }

    fn setitimer(which: c_int, new: *const itimerval, old: *mut itimerval) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(SETITIMER, which, new, old) })?;
        Ok(())
    }

    fn sigaction(
        sig: c_int,
        act: Option<&sigaction>,
        oact: Option<&mut sigaction>,
    ) -> Result<(), Errno> {
        e_raw(unsafe {
            syscall!(
                RT_SIGACTION,
                sig,
                act.map_or_else(core::ptr::null, |x| x as *const _),
                oact.map_or_else(core::ptr::null_mut, |x| x as *mut _),
                mem::size_of::<sigset_t>()
            )
        })?;
        Ok(())
    }

    fn sigaltstack(ss: *const stack_t, old_ss: *mut stack_t) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(SIGALTSTACK, ss, old_ss) })?;
        Ok(())
    }

    fn sigpending(set: *mut sigset_t) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(RT_SIGPENDING, set, NSIG / 8) })?;
        Ok(())
    }

    fn sigprocmask(how: c_int, set: *const sigset_t, oset: *mut sigset_t) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(RT_SIGPROCMASK, how, set, oset, mem::size_of::<sigset_t>()) })?;
        Ok(())
    }

    fn sigsuspend(set: *const sigset_t) -> Errno {
        Errno(unsafe { syscall!(RT_SIGSUSPEND, set, NSIG / 8) } as c_int)
    }

    fn sigtimedwait(
        set: *const sigset_t,
        sig: *mut siginfo_t,
        tp: *const timespec,
    ) -> Result<c_int, Errno> {
        e_raw(unsafe { syscall!(RT_SIGTIMEDWAIT, set, sig, tp, NSIG / 8) }).map(|res| res as c_int)
    }
}
