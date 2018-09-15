use super::super::types::*;
use super::super::Pal;
use header::signal::{sigaction, sigset_t};

pub trait PalSignal: Pal {
    fn kill(pid: pid_t, sig: c_int) -> c_int;

    fn killpg(pgrp: pid_t, sig: c_int) -> c_int;

    fn raise(sig: c_int) -> c_int;

    unsafe fn sigaction(sig: c_int, act: *const sigaction, oact: *mut sigaction) -> c_int;
}
