use core::{
    mem,
    ptr::{self, addr_of},
};

use super::{
    super::{
        PalSignal,
        types::{c_int, c_uint, pid_t, time_t},
    },
    Sys, e_raw,
};
use crate::{
    error::{Errno, Result},
    header::{
        bits_timespec::timespec,
        signal::{SA_RESTORER, SI_QUEUE, sigaction, siginfo_t, sigset_t, sigval, stack_t},
        sys_select::timeval,
        sys_time::{self, itimerval},
    },
    platform,
};

impl PalSignal for Sys {
    fn getitimer(which: c_int, out: &mut itimerval) -> Result<()> {
        unsafe {
            e_raw(syscall!(GETITIMER, which, ptr::from_mut(out)))?;
        }
        Ok(())
    }

    fn kill(pid: pid_t, sig: c_int) -> Result<()> {
        e_raw(unsafe { syscall!(KILL, pid, sig) })?;
        Ok(())
    }
    fn sigqueue(pid: pid_t, sig: c_int, val: sigval) -> Result<()> {
        let info = siginfo_t {
            si_addr: core::ptr::null_mut(),
            si_code: SI_QUEUE,
            si_errno: 0,
            si_pid: 0, // TODO: GETPID?
            si_signo: sig,
            si_status: 0,
            si_uid: 0, // TODO: GETUID?
            si_value: val,
        };
        e_raw(unsafe { syscall!(RT_SIGQUEUEINFO, pid, sig, addr_of!(info)) }).map(|_| ())
    }

    fn killpg(pgrp: pid_t, sig: c_int) -> Result<()> {
        e_raw(unsafe { syscall!(KILL, -(pgrp as isize) as pid_t, sig) })?;
        Ok(())
    }

    fn raise(sig: c_int) -> Result<()> {
        let tid = e_raw(unsafe { syscall!(GETTID) })? as pid_t;
        e_raw(unsafe { syscall!(TKILL, tid, sig) })?;
        Ok(())
    }

    fn setitimer(which: c_int, new: &itimerval, old: Option<&mut itimerval>) -> Result<()> {
        e_raw(unsafe {
            syscall!(
                SETITIMER,
                which,
                ptr::from_ref(new),
                old.map_or_else(ptr::null_mut, ptr::from_mut)
            )
        })?;
        Ok(())
    }

    /// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/alarm.html>.
    fn alarm(seconds: c_uint) -> c_uint {
        let timer = itimerval {
            it_value: timeval {
                tv_sec: time_t::from(seconds),
                tv_usec: 0,
            },
            ..Default::default()
        };
        let mut otimer = itimerval::default();
        let errno_backup = platform::ERRNO.get();
        let secs = if Self::setitimer(sys_time::ITIMER_REAL, &timer, Some(&mut otimer)).is_err() {
            0
        } else {
            otimer.it_value.tv_sec as c_uint + if otimer.it_value.tv_usec > 0 { 1 } else { 0 }
        };
        platform::ERRNO.set(errno_backup);
        secs
    }

    fn sigaction(
        sig: c_int,
        act: Option<&sigaction>,
        oact: Option<&mut sigaction>,
    ) -> Result<(), Errno> {
        unsafe extern "C" {
            fn __restore_rt();
        }
        let act = act.map(|act| {
            let mut act_clone = act.clone();
            act_clone.sa_flags |= SA_RESTORER as c_int;
            act_clone.sa_restorer = Some(__restore_rt);
            act_clone
        });
        e_raw(unsafe {
            syscall!(
                RT_SIGACTION,
                sig,
                act.as_ref().map_or_else(ptr::null, ptr::from_ref),
                oact.map_or_else(ptr::null_mut, ptr::from_mut),
                mem::size_of::<sigset_t>()
            )
        })
        .map(|_| ())
    }

    unsafe fn sigaltstack(ss: Option<&stack_t>, old_ss: Option<&mut stack_t>) -> Result<()> {
        e_raw(syscall!(
            SIGALTSTACK,
            ss.map_or_else(ptr::null, ptr::from_ref),
            old_ss.map_or_else(ptr::null_mut, ptr::from_mut)
        ))
        .map(|_| ())
    }

    fn sigpending(set: &mut sigset_t) -> Result<()> {
        e_raw(unsafe {
            syscall!(
                RT_SIGPENDING,
                ptr::from_mut::<sigset_t>(set) as usize,
                mem::size_of::<sigset_t>()
            )
        })
        .map(|_| ())
    }

    fn sigprocmask(how: c_int, set: Option<&sigset_t>, oset: Option<&mut sigset_t>) -> Result<()> {
        e_raw(unsafe {
            syscall!(
                RT_SIGPROCMASK,
                how,
                set.map_or_else(ptr::null, ptr::from_ref),
                oset.map_or_else(ptr::null_mut, ptr::from_mut),
                mem::size_of::<sigset_t>()
            )
        })
        .map(|_| ())
    }

    fn sigsuspend(mask: &sigset_t) -> Errno {
        unsafe {
            e_raw(syscall!(
                RT_SIGSUSPEND,
                ptr::from_ref::<sigset_t>(mask),
                size_of::<sigset_t>()
            ))
            .expect_err("must fail")
        }
    }

    fn sigtimedwait(
        set: &sigset_t,
        sig: Option<&mut siginfo_t>,
        tp: Option<&timespec>,
    ) -> Result<c_int> {
        unsafe {
            e_raw(syscall!(
                RT_SIGTIMEDWAIT,
                ptr::from_ref(set),
                sig.map_or_else(ptr::null_mut, ptr::from_mut),
                tp.map_or_else(ptr::null, ptr::from_ref),
                size_of::<sigset_t>()
            ))
            .map(|s| s as c_int)
        }
    }
}
