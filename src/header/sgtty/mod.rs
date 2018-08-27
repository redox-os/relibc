//! sgtty implementation that won't work on redox because no ioctl

use core::fmt::Write;

use header::sys_ioctl::*;
use platform;
use platform::types::*;

#[no_mangle]
pub extern "C" fn gtty(fd: c_int, out: *mut sgttyb) -> c_int {
    writeln!(
        platform::FileWriter(2),
        "unimplemented: gtty({}, {:p})",
        fd,
        out
    );
    -1
}
