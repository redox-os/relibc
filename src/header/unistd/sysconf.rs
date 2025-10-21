/// sysconf.h: <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sysconf.html>.

#[cfg(target_os = "redox")]
#[path = "sysconf/redox.rs"]
mod sys;

#[cfg(target_os = "linux")]
#[path = "sysconf/linux.rs"]
mod sys;

pub use sys::*;

use core::ffi::{c_int, c_long};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sysconf(name: c_int) -> c_long {
    sysconf_impl(name)
}
