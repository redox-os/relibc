use super::super::{Pal, types::*};
use crate::{
    error::{Errno, Result},
    header::{
        bits_time::timespec,
        signal::{sigaction, siginfo_t, sigset_t, sigval, stack_t},
        sys_time::itimerval,
    },
};

/// Platform abstraction of signal-related functionality.
pub trait PalSignal: Pal {
    /// Platform implementation of [`getitimer()`](crate::header::sys_time::getitimer) from [`sys/time.h`](crate::header::sys_time).
    fn getitimer(which: c_int, out: &mut itimerval) -> Result<()>;

    /// Platform implementation of [`kill()`](crate::header::signal::kill) from [`signal.h`](crate::header::signal).
    fn kill(pid: pid_t, sig: c_int) -> Result<()>;

    /// Platform implementation of [`sigqueue()`](crate::header::signal::sigqueue) from [`signal.h`](crate::header::signal).
    fn sigqueue(pid: pid_t, sig: c_int, val: sigval) -> Result<()>;

    /// Platform implementation of [`killpg()`](crate::header::signal::killpg) from [`signal.h`](crate::header::signal).
    fn killpg(pgrp: pid_t, sig: c_int) -> Result<()>;

    /// Platform implementation of [`raise()`](crate::header::signal::raise) from [`signal.h`](crate::header::signal).
    fn raise(sig: c_int) -> Result<()>;

    /// Platform implementation of [`setitimer()`](crate::header::sys_time::setitimer) from [`sys/time.h`](crate::header::sys_time).
    fn setitimer(which: c_int, new: &itimerval, old: Option<&mut itimerval>) -> Result<()>;

    /// Platform implementation of [`alarm()`](crate::header::unistd::alarm) from [`unistd.h`](crate::header::unistd).
    fn alarm(seconds: c_uint) -> c_uint;

    /// Platform implementation of [`sigaction()`](crate::header::signal::sigaction()) from [`signal.h`](crate::header::signal).
    fn sigaction(sig: c_int, act: Option<&sigaction>, oact: Option<&mut sigaction>) -> Result<()>;

    /// Platform implementation of [`sigaltstack()`](crate::header::signal::sigaltstack()) from [`signal.h`](crate::header::signal).
    unsafe fn sigaltstack(ss: Option<&stack_t>, old_ss: Option<&mut stack_t>) -> Result<()>;

    /// Platform implementation of [`sigpending()`](crate::header::signal::sigpending) from [`signal.h`](crate::header::signal).
    fn sigpending(set: &mut sigset_t) -> Result<()>;

    /// Platform implementation of [`sigprocmask()`](crate::header::signal::sigprocmask) from [`signal.h`](crate::header::signal).
    fn sigprocmask(how: c_int, set: Option<&sigset_t>, oset: Option<&mut sigset_t>) -> Result<()>;

    /// Platform implementation of [`sigsuspend()`](crate::header::signal::sigsuspend) from [`signal.h`](crate::header::signal).
    fn sigsuspend(mask: &sigset_t) -> Errno; // always fails

    /// Platform implementation of [`sigtimedwait()`](crate::header::signal::sigtimedwait) from [`signal.h`](crate::header::signal).
    fn sigtimedwait(
        set: &sigset_t,
        sig: Option<&mut siginfo_t>,
        tp: Option<&timespec>,
    ) -> Result<c_int>;
}
