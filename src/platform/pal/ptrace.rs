use crate::{
    error::Result,
    platform::{Pal, types::*},
};

/// Platform abstraction for `ptrace` functionality.
pub trait PalPtrace: Pal {
    /// Platform implementation of [`ptrace()`](crate::header::sys_ptrace::ptrace) from [`sys/ptrace.h`](crate::header::sys_ptrace).
    unsafe fn ptrace(
        request: c_int,
        pid: pid_t,
        addr: *mut c_void,
        data: *mut c_void,
    ) -> Result<c_int>;
}
