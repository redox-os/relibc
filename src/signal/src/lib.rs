//! signal implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/signal.h.html

#![no_std]
#![feature(asm, const_fn, core_intrinsics, global_asm)]

#[macro_use]
extern crate platform;

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
    pub sa_handler: extern "C" fn(c_int),
    pub sa_flags: c_ulong,
    pub sa_restorer: unsafe extern "C" fn(),
    pub sa_mask: sigset_t
}

const NSIG: usize = 64;

pub use sys::*;

use core::{mem, ptr};
use platform::types::*;
use platform::sigset_t;

#[no_mangle]
pub extern "C" fn kill(pid: pid_t, sig: c_int) -> c_int {
    platform::kill(pid, sig)
}

#[no_mangle]
pub extern "C" fn killpg(pgrp: pid_t, sig: c_int) -> c_int {
    platform::killpg(pgrp, sig)
}

#[no_mangle]
pub extern "C" fn raise(sig: c_int) -> c_int {
    platform::raise(sig)
}

#[no_mangle]
pub unsafe extern "C" fn sigaction(sig: c_int, act: *const sigaction, oact: *mut sigaction) -> c_int {
    let mut _sigaction = None;
    let ptr = if !act.is_null() {
        _sigaction = Some((*act).clone());
        _sigaction.as_mut().unwrap().sa_flags |= SA_RESTORER as c_ulong;
        _sigaction.as_mut().unwrap() as *mut _ as *mut platform::sigaction
    } else {
        ptr::null_mut()
    };
    platform::sigaction(sig, ptr, oact as *mut platform::sigaction)
}

// #[no_mangle]
pub extern "C" fn sigaddset(set: *mut sigset_t, signo: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn sigdelset(set: *mut sigset_t, signo: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn sigemptyset(set: *mut sigset_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sigfillset(set: *mut sigset_t) -> c_int {
    for i in unsafe { &mut (*set) } {
        *i = c_ulong::max_value();
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
pub extern "C" fn signal(sig: c_int, func: extern "C" fn(c_int)) -> extern "C" fn(c_int) {
    let sa = sigaction {
        sa_handler: func,
        sa_flags: SA_RESTART as c_ulong,
        sa_restorer: __restore_rt,
        sa_mask: sigset_t::default()
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
    platform::sigprocmask(how, set, oset)
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
