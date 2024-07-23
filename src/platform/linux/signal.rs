use core::mem;

use super::{
    super::{types::*, PalSignal},
    e, e_raw, Sys,
};
use crate::{
    header::{
        signal::{sigaction, siginfo_t, sigset_t, stack_t, NSIG, SA_RESTORER},
        sys_time::itimerval,
        time::timespec,
    },
    pthread::Errno,
};

impl PalSignal for Sys {
    unsafe fn getitimer(which: c_int, out: *mut itimerval) -> c_int {
        e(syscall!(GETITIMER, which, out)) as c_int
    }

    fn kill(pid: pid_t, sig: c_int) -> c_int {
        e(unsafe { syscall!(KILL, pid, sig) }) as c_int
    }

    fn killpg(pgrp: pid_t, sig: c_int) -> c_int {
        e(unsafe { syscall!(KILL, -(pgrp as isize) as pid_t, sig) }) as c_int
    }

    fn raise(sig: c_int) -> Result<(), Errno> {
        let tid = e_raw(unsafe { syscall!(GETTID) })? as pid_t;
        e_raw(unsafe { syscall!(TKILL, tid, sig) })?;
        Ok(())
    }

    unsafe fn setitimer(which: c_int, new: *const itimerval, old: *mut itimerval) -> c_int {
        e(syscall!(SETITIMER, which, new, old)) as c_int
    }

    fn sigaction(
        sig: c_int,
        act: Option<&sigaction>,
        oact: Option<&mut sigaction>,
    ) -> Result<(), Errno> {
        extern "C" {
            fn __restore_rt();
        }
        let act = act.map(|act| {
            let mut act_clone = act.clone();
            act_clone.sa_flags |= SA_RESTORER as c_ulong;
            act_clone.sa_restorer = Some(__restore_rt);
            act_clone
        });
        e_raw(unsafe {
            syscall!(
                RT_SIGACTION,
                sig,
                act.as_ref().map_or_else(core::ptr::null, |x| x as *const _),
                oact.map_or_else(core::ptr::null_mut, |x| x as *mut _),
                mem::size_of::<sigset_t>()
            )
        })
        .map(|_| ())
    }

    unsafe fn sigaltstack(ss: Option<&stack_t>, old_ss: Option<&mut stack_t>) -> Result<(), Errno> {
        e_raw(syscall!(
            SIGALTSTACK,
            ss.map_or_else(core::ptr::null, |x| x as *const _),
            old_ss.map_or_else(core::ptr::null_mut, |x| x as *mut _)
        ))
        .map(|_| ())
    }

    fn sigpending(set: &mut sigset_t) -> Result<(), Errno> {
        e_raw(unsafe {
            syscall!(
                RT_SIGPENDING,
                set as *mut sigset_t as usize,
                mem::size_of::<sigset_t>()
            )
        })
        .map(|_| ())
    }

    fn sigprocmask(
        how: c_int,
        set: Option<&sigset_t>,
        oset: Option<&mut sigset_t>,
    ) -> Result<(), Errno> {
        e_raw(unsafe {
            syscall!(
                RT_SIGPROCMASK,
                how,
                set.map_or_else(core::ptr::null, |x| x as *const _),
                oset.map_or_else(core::ptr::null_mut, |x| x as *mut _),
                mem::size_of::<sigset_t>()
            )
        })
        .map(|_| ())
    }

    unsafe fn sigsuspend(set: *const sigset_t) -> c_int {
        e(syscall!(RT_SIGSUSPEND, set, NSIG / 8)) as c_int
    }

    unsafe fn sigtimedwait(
        set: *const sigset_t,
        sig: *mut siginfo_t,
        tp: *const timespec,
    ) -> c_int {
        e(syscall!(RT_SIGTIMEDWAIT, set, sig, tp, NSIG / 8)) as c_int
    }
}
