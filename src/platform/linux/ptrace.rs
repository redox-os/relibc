use super::{
    super::{types::*, PalPtrace},
    e_raw, Sys,
};
use crate::errno::Errno;

impl PalPtrace for Sys {
    fn ptrace(
        request: c_int,
        pid: pid_t,
        addr: *mut c_void,
        data: *mut c_void,
    ) -> Result<c_int, Errno> {
        e_raw(unsafe { syscall!(PTRACE, request, pid, addr, data) }).map(|res| res as c_int)
    }
}
