use syscall;

use super::super::types::*;
use super::super::{Pal, PalSignal};
use super::{e, Sys};

#[thread_local]
static mut SIG_HANDLER: Option<extern "C" fn(c_int)> = None;

extern "C" fn sig_handler(sig: usize) {
    if let Some(ref callback) = unsafe { SIG_HANDLER } {
        callback(sig as c_int);
    }
}

impl PalSignal for Sys {
    fn kill(pid: pid_t, sig: c_int) -> c_int {
        e(syscall::kill(pid as usize, sig as usize)) as c_int
    }

    fn killpg(pgrp: pid_t, sig: c_int) -> c_int {
        e(syscall::kill(-(pgrp as isize) as usize, sig as usize)) as c_int
    }

    fn raise(sig: c_int) -> c_int {
        Self::kill(Self::getpid(), sig)
    }

    unsafe fn sigaction(sig: c_int, act: *const sigaction, oact: *mut sigaction) -> c_int {
        if !oact.is_null() {
            // Assumes the last sigaction() call was made by relibc and not a different one
            if SIG_HANDLER.is_some() {
                (*oact).sa_handler = SIG_HANDLER;
            }
        }
        let act = if act.is_null() {
            None
        } else {
            SIG_HANDLER = (*act).sa_handler;
            let m = (*act).sa_mask;
            Some(syscall::SigAction {
                sa_handler: sig_handler,
                sa_mask: [0, m as u64],
                sa_flags: (*act).sa_flags as usize,
            })
        };
        let mut old = syscall::SigAction::default();
        let ret = e(syscall::sigaction(
            sig as usize,
            act.as_ref(),
            if oact.is_null() { None } else { Some(&mut old) },
        )) as c_int;
        if !oact.is_null() {
            let m = old.sa_mask;
            (*oact).sa_mask = m[1] as c_ulong;
            (*oact).sa_flags = old.sa_flags as c_ulong;
        }
        ret
    }

    //fn sigprocmask(how: c_int, set: *const sigset_t, oset: *mut sigset_t) -> c_int;
}
