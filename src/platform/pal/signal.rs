use super::super::{types::*, Pal};
use crate::{
    error::Errno,
    header::{
        signal::{sigaction, siginfo_t, sigset_t, sigval, stack_t},
        sys_time::itimerval,
        time::timespec,
    },
};

pub trait PalSignal: Pal {
    unsafe fn getitimer(which: c_int, out: *mut itimerval) -> c_int;

    fn kill(pid: pid_t, sig: c_int) -> c_int;

    fn sigqueue(pid: pid_t, sig: c_int, val: sigval) -> Result<(), Errno>;

    fn killpg(pgrp: pid_t, sig: c_int) -> c_int;

    fn raise(sig: c_int) -> Result<(), Errno>;

    unsafe fn setitimer(which: c_int, new: *const itimerval, old: *mut itimerval) -> c_int;

    fn sigaction(
        sig: c_int,
        act: Option<&sigaction>,
        oact: Option<&mut sigaction>,
    ) -> Result<(), Errno>;

    unsafe fn sigaltstack(ss: Option<&stack_t>, old_ss: Option<&mut stack_t>) -> Result<(), Errno>;

    fn sigpending(set: &mut sigset_t) -> Result<(), Errno>;

    fn sigprocmask(
        how: c_int,
        set: Option<&sigset_t>,
        oset: Option<&mut sigset_t>,
    ) -> Result<(), Errno>;

    fn sigsuspend(mask: &sigset_t) -> Errno; // always fails

    fn sigtimedwait(set: &sigset_t, sig: &mut siginfo_t, tp: &timespec) -> Result<(), Errno>;
}
