//! pty.h implementation, not POSIX specified

use core::slice;

use crate::{
    header::{limits, sys_ioctl, termios, unistd},
    platform::types::*,
};

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
mod imp;

#[no_mangle]
pub unsafe extern "C" fn openpty(
    amaster: *mut c_int,
    aslave: *mut c_int,
    namep: *mut c_char,
    termp: *const termios::termios,
    winp: *const sys_ioctl::winsize,
) -> c_int {
    let mut tmp_name = [0; limits::PATH_MAX];
    let mut name = if !namep.is_null() {
        slice::from_raw_parts_mut(namep as *mut u8, limits::PATH_MAX)
    } else {
        &mut tmp_name
    };

    let (master, slave) = match imp::openpty(name) {
        Ok(ok) => ok,
        Err(()) => return -1,
    };

    if !termp.is_null() {
        termios::tcsetattr(slave, termios::TCSANOW, termp);
    }

    if !winp.is_null() {
        sys_ioctl::ioctl(slave, sys_ioctl::TIOCSWINSZ, winp as *mut c_void);
    }

    *amaster = master;
    *aslave = slave;

    return 0;
}
