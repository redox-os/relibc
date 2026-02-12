use crate::header::signal::sigval;
use core::{mem, ptr::addr_of};

use super::{
    super::{PalSignal, types::*},
    Sys, e_raw,
};
use crate::{
    error::{Errno, Result},
    header::{
        bits_time::timespec,
        signal::{SA_RESTORER, SI_QUEUE, sigaction, siginfo_t, sigset_t, stack_t},
        sys_time::itimerval,
    },
};

impl PalSignal for Sys {
    fn getitimer(which: c_int, out: &mut itimerval) -> Result<()> {
        unsafe {
            e_raw(syscall!(GETITIMER, which, out as *mut _))?;
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
                new as *const _,
                old.map_or_else(core::ptr::null_mut, |r| r as *mut _)
            )
        })?;
        Ok(())
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

    unsafe fn sigaltstack(ss: Option<&stack_t>, old_ss: Option<&mut stack_t>) -> Result<()> {
        e_raw(syscall!(
            SIGALTSTACK,
            ss.map_or_else(core::ptr::null, |x| x as *const _),
            old_ss.map_or_else(core::ptr::null_mut, |x| x as *mut _)
        ))
        .map(|_| ())
    }

    fn sigpending(set: &mut sigset_t) -> Result<()> {
        e_raw(unsafe {
            syscall!(
                RT_SIGPENDING,
                set as *mut sigset_t as usize,
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
                set.map_or_else(core::ptr::null, |x| x as *const _),
                oset.map_or_else(core::ptr::null_mut, |x| x as *mut _),
                mem::size_of::<sigset_t>()
            )
        })
        .map(|_| ())
    }

    fn sigsuspend(mask: &sigset_t) -> Errno {
        unsafe {
            e_raw(syscall!(
                RT_SIGSUSPEND,
                mask as *const sigset_t,
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
                set as *const _,
                sig.map_or_else(core::ptr::null_mut, |s| s as *mut _),
                tp.map_or_else(core::ptr::null, |t| t as *const _),
                size_of::<sigset_t>()
            ))
            .map(|s| s as c_int)
        }
    }
}
