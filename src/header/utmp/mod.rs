//! `utmp.h` implementation.
//!
//! Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/openpty.3.html>.

// TODO: set this for entire crate when possible
#![deny(unsafe_op_in_unsafe_fn)]

use crate::{
    header::{sys_ioctl, unistd},
    platform::types::{c_int, c_void},
};

/// See <https://www.man7.org/linux/man-pages/man3/openpty.3.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn login_tty(fd: c_int) -> c_int {
    // Create a new session
    unistd::setsid();

    // Set controlling terminal
    let mut arg: c_int = 0;
    if unsafe {
        sys_ioctl::ioctl(
            fd,
            sys_ioctl::TIOCSCTTY,
            &mut arg as *mut c_int as *mut c_void,
        )
    } != 0
    {
        return -1;
    }

    // Overwrite stdio
    unistd::dup2(fd, 0);
    unistd::dup2(fd, 1);
    unistd::dup2(fd, 2);

    // Close if needed
    if fd > 2 {
        unistd::close(fd);
    }

    0
}
