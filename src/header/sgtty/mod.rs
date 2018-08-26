//! sgtty implementation that won't work on redox because no ioctl

use core::fmt::Write;

use platform;
use platform::types::*;
use header::sys_ioctl::*;

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
