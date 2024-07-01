use super::super::{types::*, Pal};
use crate::{
    header::{
        signal::{sigaction, siginfo_t, sigset_t, stack_t},
        sys_time::itimerval,
        time::timespec,
    },
    pthread::Errno,
};

pub trait PalSignal: Pal {
    unsafe fn getitimer(which: c_int, out: *mut itimerval) -> c_int;

    fn kill(pid: pid_t, sig: c_int) -> c_int;

    fn killpg(pgrp: pid_t, sig: c_int) -> c_int;

    fn raise(sig: c_int) -> c_int;

    unsafe fn setitimer(which: c_int, new: *const itimerval, old: *mut itimerval) -> c_int;

    fn sigaction(
        sig: c_int,
        act: Option<&sigaction>,
        oact: Option<&mut sigaction>,
    ) -> Result<(), Errno>;

    unsafe fn sigaltstack(ss: Option<&stack_t>, old_ss: Option<&mut stack_t>) -> Result<(), Errno>;

    unsafe fn sigpending(set: *mut sigset_t) -> c_int;

    fn sigprocmask(
        how: c_int,
        set: Option<&sigset_t>,
        oset: Option<&mut sigset_t>,
    ) -> Result<(), Errno>;

    unsafe fn sigsuspend(set: *const sigset_t) -> c_int;

    unsafe fn sigtimedwait(set: *const sigset_t, sig: *mut siginfo_t, tp: *const timespec)
        -> c_int;
}
