//! sys/select.h implementation

use core::mem;
use header::sys_time::timeval;
use platform::types::*;
use platform::{Pal, Sys};

// fd_set is also defined in C because cbindgen is incompatible with mem::size_of booo

pub const FD_SETSIZE: usize = 1024;

#[repr(C)]
pub struct fd_set {
    pub fds_bits: [c_ulong; FD_SETSIZE / (8 * mem::size_of::<c_ulong>())],
}

#[no_mangle]
pub extern "C" fn select(
    nfds: c_int,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    exceptfds: *mut fd_set,
    timeout: *mut timeval,
) -> c_int {
    trace_expr!(
        Sys::select(nfds, readfds, writefds, exceptfds, timeout),
        "select({}, {:p}, {:p}, {:p}, {:p})",
        nfds,
        readfds,
        writefds,
        exceptfds,
        timeout
    )
}
