//! signal implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/signal.h.html

use core::{mem, ptr};

use cbitset::BitSet;

use crate::{
    header::{errno, time::timespec},
    platform::{self, types::*, Pal, PalSignal, Sys},
    pthread,
};

pub use self::sys::*;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

type SigSet = BitSet<[c_ulong; 1]>;

pub(crate) const SIG_DFL: usize = 0;
pub(crate) const SIG_IGN: usize = 1;
pub(crate) const SIG_ERR: isize = -1;
pub(crate) const SIG_HOLD: isize = 2;

pub const SIG_BLOCK: c_int = 0;
pub const SIG_UNBLOCK: c_int = 1;
pub const SIG_SETMASK: c_int = 2;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct sigaction {
    pub sa_handler: Option<extern "C" fn(c_int)>,
    pub sa_flags: c_ulong,
    pub sa_restorer: Option<unsafe extern "C" fn()>,
    pub sa_mask: sigset_t,
}

#[repr(C)]
#[derive(Clone)]
pub struct sigaltstack {
    pub ss_sp: *mut c_void,
    pub ss_flags: c_int,
    pub ss_size: size_t,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct siginfo_t {
    pub si_signo: c_int,
    pub si_errno: c_int,
    pub si_code: c_int,
    _padding: [c_int; 29],
    _si_align: [usize; 0],
}

pub type sigset_t = c_ulonglong;

pub type stack_t = sigaltstack;

#[no_mangle]
pub extern "C" fn kill(pid: pid_t, sig: c_int) -> c_int {
    Sys::kill(pid, sig)
}

#[no_mangle]
pub extern "C" fn killpg(pgrp: pid_t, sig: c_int) -> c_int {
    Sys::killpg(pgrp, sig)
}

#[no_mangle]
pub unsafe extern "C" fn pthread_kill(thread: pthread_t, sig: c_int) -> c_int {
    let os_tid = {
        let pthread = &*(thread as *const pthread::Pthread);
        pthread.os_tid.get().read()
    };
    crate::header::pthread::e(Sys::rlct_kill(os_tid, sig as usize))
}

#[no_mangle]
pub unsafe extern "C" fn pthread_sigmask(
    how: c_int,
    set: *const sigset_t,
    oldset: *mut sigset_t,
) -> c_int {
    // On Linux and Redox, pthread_sigmask and sigprocmask are equivalent
    if sigprocmask(how, set, oldset) == 0 {
        0
    } else {
        //TODO: Fix race
        platform::ERRNO.get()
    }
}

#[no_mangle]
pub extern "C" fn raise(sig: c_int) -> c_int {
    Sys::raise(sig)
}

#[no_mangle]
pub unsafe extern "C" fn sigaction(
    sig: c_int,
    act: *const sigaction,
    oact: *mut sigaction,
) -> c_int {
    let act_opt = act.as_ref().map(|act| {
        let mut act_clone = act.clone();
        act_clone.sa_flags |= SA_RESTORER as c_ulong;
        act_clone.sa_restorer = Some(__restore_rt);
        act_clone
    });
    Sys::sigaction(sig, act_opt.as_ref(), oact.as_mut())
}

#[no_mangle]
pub extern "C" fn sigaddset(set: *mut sigset_t, signo: c_int) -> c_int {
    if signo <= 0 || signo as usize > NSIG {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    if let Some(set) = unsafe { (set as *mut SigSet).as_mut() } {
        set.insert(signo as usize - 1); // 0-indexed usize, please!
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigaltstack(ss: *const stack_t, old_ss: *mut stack_t) -> c_int {
    if !ss.is_null() {
        if (*ss).ss_flags != SS_DISABLE as c_int {
            return errno::EINVAL;
        }
        if (*ss).ss_size < MINSIGSTKSZ {
            return errno::ENOMEM;
        }
    }

    Sys::sigaltstack(ss, old_ss)
}

#[no_mangle]
pub extern "C" fn sigdelset(set: *mut sigset_t, signo: c_int) -> c_int {
    if signo <= 0 || signo as usize > NSIG {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    if let Some(set) = unsafe { (set as *mut SigSet).as_mut() } {
        set.remove(signo as usize - 1); // 0-indexed usize, please!
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigemptyset(set: *mut sigset_t) -> c_int {
    if let Some(set) = (set as *mut SigSet).as_mut() {
        set.clear();
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigfillset(set: *mut sigset_t) -> c_int {
    if let Some(set) = (set as *mut SigSet).as_mut() {
        set.fill(.., true);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn sighold(sig: c_int) -> c_int {
    let mut pset = mem::MaybeUninit::<sigset_t>::uninit();
    unsafe { sigemptyset(pset.as_mut_ptr()) };
    let mut set = unsafe { pset.assume_init() };
    if sigaddset(&mut set, sig) < 0 {
        return -1;
    }
    sigprocmask(SIG_BLOCK, &set, ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn sigignore(sig: c_int) -> c_int {
    let mut psa = mem::MaybeUninit::<sigaction>::uninit();
    unsafe { sigemptyset(&mut (*psa.as_mut_ptr()).sa_mask) };
    let mut sa = unsafe { psa.assume_init() };
    sa.sa_handler = unsafe { mem::transmute(SIG_IGN) };
    sa.sa_flags = 0;
    unsafe { sigaction(sig, &mut sa, ptr::null_mut()) }
}

#[no_mangle]
pub extern "C" fn siginterrupt(sig: c_int, flag: c_int) -> c_int {
    let mut psa = mem::MaybeUninit::<sigaction>::uninit();
    unsafe { sigaction(sig, ptr::null_mut(), psa.as_mut_ptr()) };
    let mut sa = unsafe { psa.assume_init() };
    if flag != 0 {
        sa.sa_flags &= !SA_RESTART as c_ulong;
    } else {
        sa.sa_flags |= SA_RESTART as c_ulong;
    }

    unsafe { sigaction(sig, &mut sa, ptr::null_mut()) }
}

#[no_mangle]
pub unsafe extern "C" fn sigismember(set: *const sigset_t, signo: c_int) -> c_int {
    if signo <= 0 || signo as usize > NSIG {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    if let Some(set) = unsafe { (set as *mut SigSet).as_mut() } {
        if set.contains(signo as usize - 1) {
            return 1;
        }
    }
    0
}

extern "C" {
    // Defined in assembly inside platform/x/mod.rs
    fn __restore_rt();
}

#[no_mangle]
pub extern "C" fn signal(
    sig: c_int,
    func: Option<extern "C" fn(c_int)>,
) -> Option<extern "C" fn(c_int)> {
    let sa = sigaction {
        sa_handler: func,
        sa_flags: SA_RESTART as c_ulong,
        sa_restorer: Some(__restore_rt),
        sa_mask: sigset_t::default(),
    };
    let mut old_sa = mem::MaybeUninit::uninit();
    if unsafe { sigaction(sig, &sa, old_sa.as_mut_ptr()) } < 0 {
        mem::forget(old_sa);
        return unsafe { mem::transmute(SIG_ERR) };
    }
    unsafe { old_sa.assume_init() }.sa_handler
}

#[no_mangle]
pub unsafe extern "C" fn sigpause(sig: c_int) -> c_int {
    let mut pset = mem::MaybeUninit::<sigset_t>::uninit();
    sigprocmask(0, ptr::null_mut(), pset.as_mut_ptr());
    let mut set = pset.assume_init();
    sigdelset(&mut set, sig);
    sigsuspend(&mut set)
}

#[no_mangle]
pub unsafe extern "C" fn sigpending(set: *mut sigset_t) -> c_int {
    Sys::sigpending(set)
}

const BELOW_SIGRTMIN_MASK: sigset_t = (1 << SIGRTMIN) - 1;
const STANDARD_SIG_MASK: sigset_t = (1 << 32) - 1;
const RLCT_SIGNAL_MASK: sigset_t = BELOW_SIGRTMIN_MASK & !STANDARD_SIG_MASK;

#[no_mangle]
pub unsafe extern "C" fn sigprocmask(
    how: c_int,
    set: *const sigset_t,
    oset: *mut sigset_t,
) -> c_int {
    let set = set.as_ref().map(|&block| block & !RLCT_SIGNAL_MASK);

    Sys::sigprocmask(
        how,
        set.as_ref()
            .map_or(core::ptr::null(), |r| r as *const sigset_t),
        oset,
    )
}

#[no_mangle]
pub unsafe extern "C" fn sigrelse(sig: c_int) -> c_int {
    let mut pset = mem::MaybeUninit::<sigset_t>::uninit();
    sigemptyset(pset.as_mut_ptr());
    let mut set = pset.assume_init();
    if sigaddset(&mut set, sig) < 0 {
        return -1;
    }
    sigprocmask(SIG_UNBLOCK, &mut set, ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn sigset(
    sig: c_int,
    func: Option<extern "C" fn(c_int)>,
) -> Option<extern "C" fn(c_int)> {
    let mut old_sa = mem::MaybeUninit::uninit();
    let mut pset = mem::MaybeUninit::<sigset_t>::uninit();
    let sig_hold: Option<extern "C" fn(c_int)> = mem::transmute(SIG_HOLD);
    let sig_err: Option<extern "C" fn(c_int)> = mem::transmute(SIG_ERR);
    sigemptyset(pset.as_mut_ptr());
    let mut set = pset.assume_init();
    if sigaddset(&mut set, sig) < 0 {
        return sig_err;
    } else {
        if func == sig_hold {
            if sigaction(sig, ptr::null_mut(), old_sa.as_mut_ptr()) < 0
                || sigprocmask(SIG_BLOCK, &mut set, &mut set) < 0
            {
                mem::forget(old_sa);
                return sig_err;
            }
        } else {
            let mut sa = sigaction {
                sa_handler: func,
                sa_flags: 0 as c_ulong,
                sa_restorer: Some(__restore_rt),
                sa_mask: sigset_t::default(),
            };
            sigemptyset(&mut sa.sa_mask);
            if sigaction(sig, &sa, old_sa.as_mut_ptr()) < 0
                || sigprocmask(SIG_UNBLOCK, &mut set, &mut set) < 0
            {
                mem::forget(old_sa);
                return sig_err;
            }
        }
    }
    if sigismember(&mut set, sig) == 1 {
        return sig_hold;
    }
    old_sa.assume_init().sa_handler
}

#[no_mangle]
pub unsafe extern "C" fn sigsuspend(sigmask: *const sigset_t) -> c_int {
    Sys::sigsuspend(sigmask)
}

#[no_mangle]
pub unsafe extern "C" fn sigwait(set: *const sigset_t, sig: *mut c_int) -> c_int {
    let mut pinfo = mem::MaybeUninit::<siginfo_t>::uninit();
    if sigtimedwait(set, pinfo.as_mut_ptr(), ptr::null_mut()) < 0 {
        return -1;
    }
    let info = pinfo.assume_init();
    (*sig) = info.si_signo;
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigtimedwait(
    set: *const sigset_t,
    sig: *mut siginfo_t,
    tp: *const timespec,
) -> c_int {
    Sys::sigtimedwait(set, sig, tp)
}

pub const _signal_strings: [&str; 32] = [
    "Unknown signal\0",
    "Hangup\0",
    "Interrupt\0",
    "Quit\0",
    "Illegal instruction\0",
    "Trace/breakpoint trap\0",
    "Aborted\0",
    "Bus error\0",
    "Arithmetic exception\0",
    "Killed\0",
    "User defined signal 1\0",
    "Segmentation fault\0",
    "User defined signal 2\0",
    "Broken pipe\0",
    "Alarm clock\0",
    "Terminated\0",
    "Stack fault\0",
    "Child process status\0",
    "Continued\0",
    "Stopped (signal)\0",
    "Stopped\0",
    "Stopped (tty input)\0",
    "Stopped (tty output)\0",
    "Urgent I/O condition\0",
    "CPU time limit exceeded\0",
    "File size limit exceeded\0",
    "Virtual timer expired\0",
    "Profiling timer expired\0",
    "Window changed\0",
    "I/O possible\0",
    "Power failure\0",
    "Bad system call\0",
];
