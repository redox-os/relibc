use core::mem;

use super::{
    super::{types::*, PalSignal},
    e, Sys,
};
use crate::header::{
    signal::{sigaction, siginfo_t, sigset_t, stack_t, NSIG},
    sys_time::itimerval,
    time::timespec,
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

    fn raise(sig: c_int) -> c_int {
        let tid = e(unsafe { syscall!(GETTID) }) as pid_t;
        if tid == !0 {
            -1
        } else {
            e(unsafe { syscall!(TKILL, tid, sig) }) as c_int
        }
    }

    unsafe fn setitimer(which: c_int, new: *const itimerval, old: *mut itimerval) -> c_int {
        e(syscall!(SETITIMER, which, new, old)) as c_int
    }

    fn sigaction(sig: c_int, act: Option<&sigaction>, oact: Option<&mut sigaction>) -> c_int {
        e(unsafe {
            syscall!(
                RT_SIGACTION,
                sig,
                act.map_or_else(core::ptr::null, |x| x as *const _),
                oact.map_or_else(core::ptr::null_mut, |x| x as *mut _),
                mem::size_of::<sigset_t>()
            )
        }) as c_int
    }

    unsafe fn sigaltstack(ss: *const stack_t, old_ss: *mut stack_t) -> c_int {
        e(syscall!(SIGALTSTACK, ss, old_ss)) as c_int
    }

    unsafe fn sigpending(set: *mut sigset_t) -> c_int {
        e(syscall!(RT_SIGPENDING, set, NSIG / 8)) as c_int
    }

    unsafe fn sigprocmask(how: c_int, set: *const sigset_t, oset: *mut sigset_t) -> c_int {
        e(syscall!(
            RT_SIGPROCMASK,
            how,
            set,
            oset,
            mem::size_of::<sigset_t>()
        )) as c_int
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
