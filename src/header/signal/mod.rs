//! signal implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/signal.h.html

use core::{mem, ptr};

use header::errno;
use platform;
use platform::{PalSignal, Sys};
use platform::types::*;

pub use self::sys::*;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

const SIG_ERR: usize = !0;

pub const SIG_BLOCK: c_int = 0;
pub const SIG_UNBLOCK: c_int = 1;
pub const SIG_SETMASK: c_int = 2;

// Need both here and in platform because cbindgen :(
#[repr(C)]
#[derive(Clone)]
pub struct sigaction {
    // I don't actually want these to be optional. They can have more than just
    // one invalid value. But because of rust's non-null optimization, this
    // causes Some(sigaction) with a null sa_handler to become None.  Maybe
    // these should be usizes and transmuted when needed... However, then I
    // couldn't let cbindgen do its job.
    pub sa_handler: Option<extern "C" fn(c_int)>,
    pub sa_flags: c_ulong,
    pub sa_restorer: Option<unsafe extern "C" fn()>,
    pub sa_mask: sigset_t,
}

pub const NSIG: usize = 64;
pub type sigset_t = c_ulong;

#[no_mangle]
pub extern "C" fn kill(pid: pid_t, sig: c_int) -> c_int {
    Sys::kill(pid, sig)
}

#[no_mangle]
pub extern "C" fn killpg(pgrp: pid_t, sig: c_int) -> c_int {
    Sys::killpg(pgrp, sig)
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
    let mut _sigaction = None;
    let ptr = if !act.is_null() {
        _sigaction = Some((*act).clone());
        _sigaction.as_mut().unwrap().sa_flags |= SA_RESTORER as c_ulong;
        _sigaction.as_mut().unwrap() as *mut _ as *mut platform::types::sigaction
    } else {
        ptr::null_mut()
    };
    Sys::sigaction(sig, ptr, oact as *mut platform::types::sigaction)
}

#[no_mangle]
pub extern "C" fn sigaddset(set: *mut sigset_t, mut signo: c_int) -> c_int {
    if signo <= 0 || signo as usize > NSIG {
        unsafe {
            platform::errno = errno::EINVAL;
        }
        return -1;
    }

    let signo = signo as usize - 1; // 0-indexed usize, please!
    unsafe {
        *set |= 1 << (signo & (8 * mem::size_of::<sigset_t>() - 1));
    }
    0
}

#[no_mangle]
pub extern "C" fn sigdelset(set: *mut sigset_t, signo: c_int) -> c_int {
    if signo <= 0 || signo as usize > NSIG {
        unsafe {
            platform::errno = errno::EINVAL;
        }
        return -1;
    }

    let signo = signo as usize - 1; // 0-indexed usize, please!
    unsafe {
        *set &= !(1 << (signo & (8 * mem::size_of::<sigset_t>() - 1)));
    }
    0
}

#[no_mangle]
pub extern "C" fn sigemptyset(set: *mut sigset_t) -> c_int {
    unsafe {
        *set = 0;
    }
    0
}

#[no_mangle]
pub extern "C" fn sigfillset(set: *mut sigset_t) -> c_int {
    unsafe {
        *set = c_ulong::max_value();
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

// #[no_mangle]
pub extern "C" fn sigismember(set: *const sigset_t, signo: c_int) -> c_int {
    unimplemented!();
}

extern "C" {
    // Defined in assembly inside platform/x/mod.rs
    fn __restore_rt();
}

#[no_mangle]
pub extern "C" fn signal(sig: c_int, func: Option<extern "C" fn(c_int)>) -> Option<extern "C" fn(c_int)> {
    let sa = sigaction {
        sa_handler: func,
        sa_flags: SA_RESTART as c_ulong,
        sa_restorer: Some(__restore_rt),
        sa_mask: sigset_t::default(),
    };
    let mut old_sa = unsafe { mem::uninitialized() };
    if unsafe { sigaction(sig, &sa, &mut old_sa) } < 0 {
        mem::forget(old_sa);
        return unsafe { mem::transmute(SIG_ERR) };
    }
    old_sa.sa_handler
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

pub const _signal_strings: [&'static str; 32] = [
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
    "Bad system call\0"
];
