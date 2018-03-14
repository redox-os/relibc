//! signal implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/signal.h.html

#![no_std]

extern crate platform;

use platform::types::*;

#[cfg(target_os = "linux")]
#[repr(C)]
pub struct sigset_t {
    pub bits: [u64; 16],
}

#[cfg(target_os = "linux")]
pub const SIGHUP: usize = 1;
#[cfg(target_os = "linux")]
pub const SIGINT: usize = 2;
#[cfg(target_os = "linux")]
pub const SIGQUIT: usize = 3;
#[cfg(target_os = "linux")]
pub const SIGILL: usize = 4;
#[cfg(target_os = "linux")]
pub const SIGTRAP: usize = 5;
#[cfg(target_os = "linux")]
pub const SIGABRT: usize = 6;
#[cfg(target_os = "linux")]
pub const SIGIOT: usize = SIGABRT;
#[cfg(target_os = "linux")]
pub const SIGBUS: usize = 7;
#[cfg(target_os = "linux")]
pub const SIGFPE: usize = 8;
#[cfg(target_os = "linux")]
pub const SIGKILL: usize = 9;
#[cfg(target_os = "linux")]
pub const SIGUSR1: usize = 10;
#[cfg(target_os = "linux")]
pub const SIGSEGV: usize = 11;
#[cfg(target_os = "linux")]
pub const SIGUSR2: usize = 12;
#[cfg(target_os = "linux")]
pub const SIGPIPE: usize = 13;
#[cfg(target_os = "linux")]
pub const SIGALRM: usize = 14;
#[cfg(target_os = "linux")]
pub const SIGTERM: usize = 15;
#[cfg(target_os = "linux")]
pub const SIGSTKFLT: usize = 16;
#[cfg(target_os = "linux")]
pub const SIGCHLD: usize = 17;
#[cfg(target_os = "linux")]
pub const SIGCONT: usize = 18;
#[cfg(target_os = "linux")]
pub const SIGSTOP: usize = 19;
#[cfg(target_os = "linux")]
pub const SIGTSTP: usize = 20;
#[cfg(target_os = "linux")]
pub const SIGTTIN: usize = 21;
#[cfg(target_os = "linux")]
pub const SIGTTOU: usize = 22;
#[cfg(target_os = "linux")]
pub const SIGURG: usize = 23;
#[cfg(target_os = "linux")]
pub const SIGXCPU: usize = 24;
#[cfg(target_os = "linux")]
pub const SIGXFSZ: usize = 25;
#[cfg(target_os = "linux")]
pub const SIGVTALRM: usize = 26;
#[cfg(target_os = "linux")]
pub const SIGPROF: usize = 27;
#[cfg(target_os = "linux")]
pub const SIGWINCH: usize = 28;
#[cfg(target_os = "linux")]
pub const SIGIO: usize = 29;
#[cfg(target_os = "linux")]
pub const SIGPOLL: usize = 29;
#[cfg(target_os = "linux")]
pub const SIGPWR: usize = 30;
#[cfg(target_os = "linux")]
pub const SIGSYS: usize = 31;
#[cfg(target_os = "linux")]
pub const SIGUNUSED: usize = SIGSYS;

#[cfg(target_os = "linux")]
pub const SA_NOCLDSTOP: usize = 1;
#[cfg(target_os = "linux")]
pub const SA_NOCLDWAIT: usize = 2;
#[cfg(target_os = "linux")]
pub const SA_SIGINFO: usize = 4;
#[cfg(target_os = "linux")]
pub const SA_ONSTACK: usize = 0x08000000;
#[cfg(target_os = "linux")]
pub const SA_RESTART: usize = 0x10000000;
#[cfg(target_os = "linux")]
pub const SA_NODEFER: usize = 0x40000000;
#[cfg(target_os = "linux")]
pub const SA_RESETHAND: usize = 0x80000000;
#[cfg(target_os = "linux")]
pub const SA_RESTORER: usize = 0x04000000;

#[cfg(target_os = "redox")]
#[repr(C)]
pub struct sigset_t {
    pub bits: [u64; 2],
}

#[cfg(target_os = "redox")]
pub const SIGHUP: usize = 1;
#[cfg(target_os = "redox")]
pub const SIGINT: usize = 2;
#[cfg(target_os = "redox")]
pub const SIGQUIT: usize = 3;
#[cfg(target_os = "redox")]
pub const SIGILL: usize = 4;
#[cfg(target_os = "redox")]
pub const SIGTRAP: usize = 5;
#[cfg(target_os = "redox")]
pub const SIGBUS: usize = 7;
#[cfg(target_os = "redox")]
pub const SIGFPE: usize = 8;
#[cfg(target_os = "redox")]
pub const SIGKILL: usize = 9;
#[cfg(target_os = "redox")]
pub const SIGUSR1: usize = 10;
#[cfg(target_os = "redox")]
pub const SIGSEGV: usize = 11;
#[cfg(target_os = "redox")]
pub const SIGUSR2: usize = 12;
#[cfg(target_os = "redox")]
pub const SIGPIPE: usize = 13;
#[cfg(target_os = "redox")]
pub const SIGALRM: usize = 14;
#[cfg(target_os = "redox")]
pub const SIGTERM: usize = 15;
#[cfg(target_os = "redox")]
pub const SIGSTKFLT: usize = 16;
#[cfg(target_os = "redox")]
pub const SIGCHLD: usize = 17;
#[cfg(target_os = "redox")]
pub const SIGCONT: usize = 18;
#[cfg(target_os = "redox")]
pub const SIGSTOP: usize = 19;
#[cfg(target_os = "redox")]
pub const SIGTSTP: usize = 20;
#[cfg(target_os = "redox")]
pub const SIGTTIN: usize = 21;
#[cfg(target_os = "redox")]
pub const SIGTTOU: usize = 22;
#[cfg(target_os = "redox")]
pub const SIGURG: usize = 23;
#[cfg(target_os = "redox")]
pub const SIGXCPU: usize = 24;
#[cfg(target_os = "redox")]
pub const SIGXFSZ: usize = 25;
#[cfg(target_os = "redox")]
pub const SIGVTALRM: usize = 26;
#[cfg(target_os = "redox")]
pub const SIGPROF: usize = 27;
#[cfg(target_os = "redox")]
pub const SIGWINCH: usize = 28;
#[cfg(target_os = "redox")]
pub const SIGIO: usize = 29;
#[cfg(target_os = "redox")]
pub const SIGPWR: usize = 30;
#[cfg(target_os = "redox")]
pub const SIGSYS: usize = 31;

#[cfg(target_os = "redox")]
pub const SA_NOCLDSTOP: usize = 0x00000001;
#[cfg(target_os = "redox")]
pub const SA_NOCLDWAIT: usize = 0x00000002;
#[cfg(target_os = "redox")]
pub const SA_SIGINFO: usize = 0x00000004;
#[cfg(target_os = "redox")]
pub const SA_RESTORER: usize = 0x04000000;
#[cfg(target_os = "redox")]
pub const SA_ONSTACK: usize = 0x08000000;
#[cfg(target_os = "redox")]
pub const SA_RESTART: usize = 0x10000000;
#[cfg(target_os = "redox")]
pub const SA_NODEFER: usize = 0x40000000;
#[cfg(target_os = "redox")]
pub const SA_RESETHAND: usize = 0x80000000;

#[repr(C)]
pub struct sigaction {
    pub sa_handler: extern "C" fn(usize),
    pub sa_mask: sigset_t,
    pub sa_flags: usize,
}

#[no_mangle]
pub extern "C" fn kill(pid: pid_t, sig: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn killpg(pgrp: pid_t, sig: c_int) -> c_int {
    unimplemented!();
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
