//! Note: This module is not going to be clean. We're not going to be
//! able to follow the specs 100%. Linux ptrace is very, very,
//! different to Redox. Many people agree that Linux ptrace is bad, so
//! we are NOT going to bend our API for the sake of
//! compatibility. So, this module will be a hellhole.

use super::super::types::*;
use super::super::PalPtrace;
use super::{e, Sys};
use crate::sync::{Mutex, Once};
use alloc::collections::BTreeMap;

#[derive(Default)]
struct State {

}

static STATE: Once<Mutex<State>> = Once::new();

fn state() -> &'static Mutex<State> {
    STATE.call_once(|| Mutex::new(State::default()))
}

impl PalPtrace for Sys {
    fn ptrace(request: c_int, pid: pid_t, addr: *mut c_void, data: *mut c_void) -> c_int {
        // Oh boy, this is not gonna be fun.........
        unimplemented!()
    }
}
