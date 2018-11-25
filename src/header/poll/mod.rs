//! poll implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/poll.h.html

use platform::types::*;
use platform::{Pal, Sys};

pub const POLLIN: c_short = 0x001;
pub const POLLPRI: c_short = 0x002;
pub const POLLOUT: c_short = 0x004;
pub const POLLERR: c_short = 0x008;
pub const POLLHUP: c_short = 0x010;
pub const POLLNVAL: c_short = 0x020;


pub type nfds_t = c_ulong;

#[repr(C)]
pub struct pollfd {
    pub fd: c_int,
    pub events: c_short,
    pub revents: c_short,
}

#[no_mangle]
pub extern "C" fn poll(fds: *mut pollfd, nfds: nfds_t, timeout: c_int) -> c_int {
    Sys::poll(fds, nfds, timeout)
}
