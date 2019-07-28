use super::{
    super::{types::*, PalPtrace},
    e, Sys,
};

impl PalPtrace for Sys {
    fn ptrace(request: c_int, pid: pid_t, addr: *mut c_void, data: *mut c_void) -> c_int {
        unsafe { e(syscall!(PTRACE, request, pid, addr, data)) as c_int }
    }
}
