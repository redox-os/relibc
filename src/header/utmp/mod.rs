//! utmp.h implementation, not POSIX specified

use crate::{
    header::{sys_ioctl, unistd},
    platform::types::*,
};

#[no_mangle]
pub unsafe extern "C" fn login_tty(fd: c_int) -> c_int {
    // Create a new session
    unistd::setsid();

    // Set controlling terminal
    let mut arg: c_int = 0;
    if sys_ioctl::ioctl(
        fd,
        sys_ioctl::TIOCSCTTY,
        &mut arg as *mut c_int as *mut c_void,
    ) != 0
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
