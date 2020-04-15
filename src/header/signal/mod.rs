//! signal implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/signal.h.html

use core::mem;

use cbitset::BitSet;

use crate::{
    header::errno,
    platform::{self, types::*, PalSignal, Sys},
};

pub use self::sys::*;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

type SigSet = BitSet<[c_ulong; 1]>;

pub const SIG_DFL: usize = 0;
pub const SIG_IGN: usize = 1;
pub const SIG_ERR: isize = -1;

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

pub type sigset_t = c_ulong;

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
pub extern "C" fn pthread_sigmask(
    how: c_int,
    set: *const sigset_t,
    oldset: *mut sigset_t,
) -> c_int {
    // On Linux and Redox, pthread_sigmask and sigprocmask are equivalent
    if sigprocmask(how, set, oldset) == 0 {
        0
    } else {
        //TODO: Fix race
        unsafe { platform::errno }
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
        unsafe {
            platform::errno = errno::EINVAL;
        }
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
        unsafe {
            platform::errno = errno::EINVAL;
        }
        return -1;
    }

    if let Some(set) = unsafe { (set as *mut SigSet).as_mut() } {
        set.remove(signo as usize - 1); // 0-indexed usize, please!
    }
    0
}

#[no_mangle]
pub extern "C" fn sigemptyset(set: *mut sigset_t) -> c_int {
    if let Some(set) = unsafe { (set as *mut SigSet).as_mut() } {
        set.clear();
    }
    0
}

#[no_mangle]
pub extern "C" fn sigfillset(set: *mut sigset_t) -> c_int {
    if let Some(set) = unsafe { (set as *mut SigSet).as_mut() } {
        set.fill(.., true);
    }
    0
}

// #[no_mangle]
pub extern "C" fn sighold(sig: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn sigignore(sig: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn siginterrupt(sig: c_int, flag: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sigismember(set: *const sigset_t, signo: c_int) -> c_int {
    if signo <= 0 || signo as usize > NSIG {
        unsafe {
            platform::errno = errno::EINVAL;
        }
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

// #[no_mangle]
pub extern "C" fn sigpause(sig: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn sigpending(set: *mut sigset_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sigprocmask(how: c_int, set: *const sigset_t, oset: *mut sigset_t) -> c_int {
    Sys::sigprocmask(how, set, oset)
}

// #[no_mangle]
pub extern "C" fn sigrelse(sig: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn sigset(sig: c_int, func: fn(c_int)) -> fn(c_int) {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn sigsuspend(sigmask: *const sigset_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn sigwait(set: *const sigset_t, sig: *mut c_int) -> c_int {
    unimplemented!();
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
