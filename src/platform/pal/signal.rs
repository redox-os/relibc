use super::super::{Pal, types::*};
use crate::{
    error::{Errno, Result},
    header::{
        bits_time::timespec,
        signal::{sigaction, siginfo_t, sigset_t, sigval, stack_t},
        sys_time::itimerval,
    },
};

pub trait PalSignal: Pal {
    fn getitimer(which: c_int, out: &mut itimerval) -> Result<()>;

    fn kill(pid: pid_t, sig: c_int) -> Result<()>;

    fn sigqueue(pid: pid_t, sig: c_int, val: sigval) -> Result<()>;

    fn killpg(pgrp: pid_t, sig: c_int) -> Result<()>;

    fn raise(sig: c_int) -> Result<()>;

    fn setitimer(which: c_int, new: &itimerval, old: Option<&mut itimerval>) -> Result<()>;

    fn sigaction(sig: c_int, act: Option<&sigaction>, oact: Option<&mut sigaction>) -> Result<()>;

    unsafe fn sigaltstack(ss: Option<&stack_t>, old_ss: Option<&mut stack_t>) -> Result<()>;

    fn sigpending(set: &mut sigset_t) -> Result<()>;

    fn sigprocmask(how: c_int, set: Option<&sigset_t>, oset: Option<&mut sigset_t>) -> Result<()>;

    fn sigsuspend(mask: &sigset_t) -> Errno; // always fails

    fn sigtimedwait(
        set: &sigset_t,
        sig: Option<&mut siginfo_t>,
        tp: Option<&timespec>,
    ) -> Result<c_int>;
}
