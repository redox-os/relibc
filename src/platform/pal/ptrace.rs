use super::super::{types::*, Pal};

pub trait PalPtrace: Pal {
    fn ptrace(request: c_int, pid: pid_t, addr: *mut c_void, data: *mut c_void) -> c_int;
}
