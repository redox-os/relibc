//! `termios.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/termios.h.html>.

use crate::{
    header::{
        errno,
        sys_ioctl::{self, winsize},
    },
    platform::{
        self,
        types::{c_int, c_ulong, c_void, pid_t},
    },
};

pub use self::sys::*;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

pub type cc_t = u8;
pub type speed_t = u32;
pub type tcflag_t = u32;

pub const TCOOFF: c_int = 0;
pub const TCOON: c_int = 1;
pub const TCIOFF: c_int = 2;
pub const TCION: c_int = 3;

pub const TCIFLUSH: c_int = 0;
pub const TCOFLUSH: c_int = 1;
pub const TCIOFLUSH: c_int = 2;

pub const TCSANOW: c_int = 0;
pub const TCSADRAIN: c_int = 1;
pub const TCSAFLUSH: c_int = 2;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/termios.h.html>.
#[cfg(target_os = "linux")]
#[repr(C)]
#[derive(Default, Clone)]
pub struct termios {
    pub c_iflag: tcflag_t,
    pub c_oflag: tcflag_t,
    pub c_cflag: tcflag_t,
    pub c_lflag: tcflag_t,
    pub c_line: cc_t,
    pub c_cc: [cc_t; NCCS],
    pub __c_ispeed: speed_t,
    pub __c_ospeed: speed_t,
}

// Must match structure in redox_termios
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/termios.h.html>.
#[cfg(target_os = "redox")]
#[repr(C)]
#[derive(Default, Clone)]
pub struct termios {
    pub c_iflag: tcflag_t,
    pub c_oflag: tcflag_t,
    pub c_cflag: tcflag_t,
    pub c_lflag: tcflag_t,
    pub c_cc: [cc_t; NCCS],
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcgetattr.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcgetattr(fd: c_int, out: *mut termios) -> c_int {
    sys_ioctl::ioctl(fd, sys_ioctl::TCGETS, out as *mut c_void)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcsetattr.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcsetattr(fd: c_int, act: c_int, value: *const termios) -> c_int {
    if act < 0 || act > 2 {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }
    // This is safe because ioctl shouldn't modify the value
    sys_ioctl::ioctl(fd, sys_ioctl::TCSETS + act as c_ulong, value as *mut c_void)
}

/// See <https://pubs.opengroup.org/onlinepubs/009695299/functions/tcgetsid.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcgetsid(fd: c_int) -> pid_t {
    let mut sid = 0;
    if sys_ioctl::ioctl(fd, sys_ioctl::TIOCGSID, (&raw mut sid) as *mut c_void) < 0 {
        return -1;
    }
    sid
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cfgetispeed.html>.
#[cfg(target_os = "linux")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cfgetispeed(termios_p: *const termios) -> speed_t {
    (*termios_p).__c_ispeed
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cfgetispeed.html>.
#[cfg(target_os = "redox")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cfgetispeed(termios_p: *const termios) -> speed_t {
    //TODO
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cfgetospeed.html>.
#[cfg(target_os = "linux")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cfgetospeed(termios_p: *const termios) -> speed_t {
    (*termios_p).__c_ospeed
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cfgetospeed.html>.
#[cfg(target_os = "redox")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cfgetospeed(termios_p: *const termios) -> speed_t {
    //TODO
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cfsetispeed.html>.
#[cfg(target_os = "linux")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cfsetispeed(termios_p: *mut termios, speed: speed_t) -> c_int {
    match speed as usize {
        B0..=B38400 | B57600..=B4000000 => {
            (*termios_p).__c_ispeed = speed;
            0
        }
        _ => {
            platform::ERRNO.set(errno::EINVAL);
            -1
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cfsetispeed.html>.
#[cfg(target_os = "redox")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cfsetispeed(termios_p: *mut termios, speed: speed_t) -> c_int {
    //TODO
    platform::ERRNO.set(errno::EINVAL);
    -1
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cfsetospeed.html>.
#[cfg(target_os = "linux")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cfsetospeed(termios_p: *mut termios, speed: speed_t) -> c_int {
    match speed as usize {
        B0..=B38400 | B57600..=B4000000 => {
            (*termios_p).__c_ospeed = speed;
            0
        }
        _ => {
            platform::ERRNO.set(errno::EINVAL);
            -1
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cfsetospeed.html>.
#[cfg(target_os = "redox")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cfsetospeed(termios_p: *mut termios, speed: speed_t) -> c_int {
    //TODO
    platform::ERRNO.set(errno::EINVAL);
    -1
}

/// Non-POSIX, 4.4 BSD extension
///
/// See <https://www.man7.org/linux/man-pages/man3/cfsetispeed.3.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cfsetspeed(termios_p: *mut termios, speed: speed_t) -> c_int {
    let r = cfsetispeed(termios_p, speed);
    if r < 0 {
        return r;
    }
    cfsetospeed(termios_p, speed)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcflush.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcflush(fd: c_int, queue: c_int) -> c_int {
    sys_ioctl::ioctl(fd, sys_ioctl::TCFLSH, queue as *mut c_void)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcdrain.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcdrain(fd: c_int) -> c_int {
    sys_ioctl::ioctl(fd, sys_ioctl::TCSBRK, 1 as *mut _)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcsendbreak.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcsendbreak(fd: c_int, _dur: c_int) -> c_int {
    // non-zero duration is ignored by musl due to it being
    // implementation-defined. we do the same.
    sys_ioctl::ioctl(fd, sys_ioctl::TCSBRK, 0 as *mut _)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcgetwinsize.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcgetwinsize(fd: c_int, sws: *mut winsize) -> c_int {
    sys_ioctl::ioctl(fd, sys_ioctl::TIOCGWINSZ, sws.cast())
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcsetwinsize.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcsetwinsize(fd: c_int, sws: *const winsize) -> c_int {
    sys_ioctl::ioctl(fd, sys_ioctl::TIOCSWINSZ, (sws as *mut winsize).cast())
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcflow.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcflow(fd: c_int, action: c_int) -> c_int {
    // non-zero duration is ignored by musl due to it being
    // implementation-defined. we do the same.
    sys_ioctl::ioctl(fd, sys_ioctl::TCXONC, action as *mut _)
}

/// Non-POSIX, BSD extension
///
/// See <https://www.man7.org/linux/man-pages/man3/cfmakeraw.3.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cfmakeraw(termios_p: *mut termios) {
    (*termios_p).c_iflag &=
        !(IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON) as u32;
    (*termios_p).c_oflag &= !OPOST as u32;
    (*termios_p).c_lflag &= !(ECHO | ECHONL | ICANON | ISIG | IEXTEN) as u32;
    (*termios_p).c_cflag &= !(CSIZE | PARENB) as u32;
    (*termios_p).c_cflag |= CS8 as u32;
    (*termios_p).c_cc[VMIN] = 1;
    (*termios_p).c_cc[VTIME] = 0;
}
