//! signal implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/signal.h.html

#![no_std]

extern crate platform;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

pub const SIG_BLOCK: c_int = 0;
pub const SIG_UNBLOCK: c_int = 1;
pub const SIG_SETMASK: c_int = 2;

pub use sys::*;

use platform::types::*;

#[repr(C)]
pub struct sigaction {
    pub sa_handler: extern "C" fn(usize),
    pub sa_mask: sigset_t,
    pub sa_flags: usize,
}

pub type sigset_t = sys_sigset_t;

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
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sigaction(sig: c_int, act: *const sigaction, oact: *const sigaction) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sigaddset(set: *mut sigset_t, signo: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sigdelset(set: *mut sigset_t, signo: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sigemptyset(set: *mut sigset_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sigfillset(set: *mut sigset_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sighold(sig: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sigignore(sig: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn siginterrupt(sig: c_int, flag: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sigismember(set: *const sigset_t, signo: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn signal(sig: c_int, func: fn(c_int)) -> fn(c_int) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sigpause(sig: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sigpending(set: *mut sigset_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sigprocmask(how: c_int, set: *const sigset_t, oset: *mut sigset_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sigrelse(sig: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sigset(sig: c_int, func: fn(c_int)) -> fn(c_int) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sigsuspend(sigmask: *const sigset_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sigwait(set: *const sigset_t, sig: *mut c_int) -> c_int {
    unimplemented!();
}
