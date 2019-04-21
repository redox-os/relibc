//! sgtty implementation that won't work on redox because no ioctl

use header::sys_ioctl::*;
use platform::types::*;

#[no_mangle]
pub extern "C" fn gtty(fd: c_int, out: *mut sgttyb) -> c_int {
    eprintln!(
        "unimplemented: gtty({}, {:p})",
        fd,
        out
    );
    -1
}
