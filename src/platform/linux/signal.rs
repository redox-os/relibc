use crate::header::signal::sigval;
use core::{mem, ptr::addr_of};

use super::{
    super::{types::*, PalSignal},
    e, e_raw, Sys,
};
use crate::{
    error::Errno,
    header::{
        signal::{sigaction, siginfo_t, sigset_t, stack_t, NSIG, SA_RESTORER, SI_QUEUE},
        sys_time::itimerval,
        time::timespec,
    },
};

// Mirrors the ucontext_t struct from the libc crate on Linux.
#[repr(C)]
pub struct ucontext_t {
    pub uc_flags: c_ulong,
    pub uc_link: *mut ucontext_t,
    pub uc_stack: stack_t,
    pub uc_mcontext: mcontext_t,
    pub uc_sigmask: sigset_t,
    __private: [u8; 512],
}
#[repr(C)]
pub struct _libc_fpstate {
    pub cwd: u16,
    pub swd: u16,
    pub ftw: u16,
    pub fop: u16,
    pub rip: u64,
    pub rdp: u64,
    pub mxcsr: u32,
    pub mxcr_mask: u32,
    pub _st: [_libc_fpxreg; 8],
    pub _xmm: [_libc_xmmreg; 16],
    __private: [u64; 12],
}
#[repr(C)]
pub struct _libc_fpxreg {
    pub significand: [u16; 4],
    pub exponent: u16,
    __private: [u16; 3],
}

#[repr(C)]
pub struct _libc_xmmreg {
    pub element: [u32; 4],
}
#[repr(C)]
pub struct mcontext_t {
    pub gregs: [i64; 23], // TODO: greg_t?
    pub fpregs: *mut _libc_fpstate,
    __private: [u64; 8],
}

impl PalSignal for Sys {
    unsafe fn getitimer(which: c_int, out: *mut itimerval) -> c_int {
        e(syscall!(GETITIMER, which, out)) as c_int
    }

    fn kill(pid: pid_t, sig: c_int) -> c_int {
        e(unsafe { syscall!(KILL, pid, sig) }) as c_int
    }
    fn sigqueue(pid: pid_t, sig: c_int, val: sigval) -> Result<(), Errno> {
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

    fn sigsuspend(set: &sigset_t) -> Errno {
        unsafe {
            e_raw(syscall!(RT_SIGSUSPEND, set as *const sigset_t, NSIG / 8)).expect_err("must fail")
        }
    }

    unsafe fn sigtimedwait(
        set: *const sigset_t,
        sig: *mut siginfo_t,
        tp: *const timespec,
    ) -> c_int {
        e(syscall!(RT_SIGTIMEDWAIT, set, sig, tp, NSIG / 8)) as c_int
    }
}
