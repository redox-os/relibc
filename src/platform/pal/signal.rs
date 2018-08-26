use super::super::Pal;
use super::super::types::*;

pub trait PalSignal: Pal {
    fn kill(pid: pid_t, sig: c_int) -> c_int {
        Self::no_pal("kill")
    }

    fn killpg(pgrp: pid_t, sig: c_int) -> c_int {
        Self::no_pal("killpg")
    }

    fn raise(sig: c_int) -> c_int {
        Self::no_pal("raise")
    }

    unsafe fn sigaction(sig: c_int, act: *const sigaction, oact: *mut sigaction) -> c_int {
        Self::no_pal("sigaction")
    }

    fn sigprocmask(how: c_int, set: *const sigset_t, oset: *mut sigset_t) -> c_int {
        Self::no_pal("sigprocmask")
    }
}
