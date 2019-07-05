use super::super::types::*;
use super::super::PalPtrace;
use super::{e, Sys};

impl PalPtrace for Sys {
    fn ptrace(request: c_int, pid: pid_t, addr: *mut c_void, data: *mut c_void) -> c_int {
        // Oh boy, this is not gonna be fun.........
        unimplemented!()
    }
}
