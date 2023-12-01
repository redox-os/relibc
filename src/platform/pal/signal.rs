use super::super::{types::*, Pal};
use crate::{
    errno::Errno,
    header::{
        signal::{sigaction, siginfo_t, sigset_t, stack_t},
        sys_time::itimerval,
        time::timespec,
    },
};

pub trait PalSignal: Pal {
    fn getitimer(which: c_int, out: *mut itimerval) -> Result<(), Errno>;

    fn kill(pid: pid_t, sig: c_int) -> Result<(), Errno>;

    fn killpg(pgrp: pid_t, sig: c_int) -> Result<(), Errno>;

    fn raise(sig: c_int) -> Result<(), Errno>;

    fn setitimer(which: c_int, new: *const itimerval, old: *mut itimerval) -> Result<(), Errno>;

    fn sigaction(
        sig: c_int,
        act: Option<&sigaction>,
        oact: Option<&mut sigaction>,
    ) -> Result<(), Errno>;

    fn sigaltstack(ss: *const stack_t, old_ss: *mut stack_t) -> Result<(), Errno>;

    fn sigpending(set: *mut sigset_t) -> Result<(), Errno>;

    fn sigprocmask(how: c_int, set: *const sigset_t, oset: *mut sigset_t) -> Result<(), Errno>;

    fn sigsuspend(set: *const sigset_t) -> Errno;

    fn sigtimedwait(
        set: *const sigset_t,
        sig: *mut siginfo_t,
        tp: *const timespec,
    ) -> Result<c_int, Errno>;
}
