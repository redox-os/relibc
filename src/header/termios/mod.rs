//! termios implementation, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/termios.h.html

use crate::{
    header::{
        errno,
        sys_ioctl::{self, winsize},
    },
    platform::{self, types::*},
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

#[cfg(target_os = "linux")]
#[repr(C)]
#[derive(Default)]
pub struct termios {
    c_iflag: tcflag_t,
    c_oflag: tcflag_t,
    c_cflag: tcflag_t,
    c_lflag: tcflag_t,
    c_line: cc_t,
    c_cc: [cc_t; NCCS],
    __c_ispeed: speed_t,
    __c_ospeed: speed_t,
}

// Must match structure in redox_termios
#[cfg(target_os = "redox")]
#[repr(C)]
#[derive(Default)]
pub struct termios {
    c_iflag: tcflag_t,
    c_oflag: tcflag_t,
    c_cflag: tcflag_t,
    c_lflag: tcflag_t,
    c_cc: [cc_t; NCCS],
}

#[no_mangle]
pub unsafe extern "C" fn tcgetattr(fd: c_int, out: *mut termios) -> c_int {
    sys_ioctl::ioctl(fd, sys_ioctl::TCGETS, out as *mut c_void)
}

#[no_mangle]
pub unsafe extern "C" fn tcsetattr(fd: c_int, act: c_int, value: *const termios) -> c_int {
    if act < 0 || act > 2 {
        platform::errno.set(errno::EINVAL);
        return -1;
    }
    // This is safe because ioctl shouldn't modify the value
    sys_ioctl::ioctl(fd, sys_ioctl::TCSETS + act as c_ulong, value as *mut c_void)
}

#[cfg(target_os = "linux")]
#[no_mangle]
pub unsafe extern "C" fn cfgetispeed(termios_p: *const termios) -> speed_t {
    (*termios_p).__c_ispeed
}

#[cfg(target_os = "redox")]
#[no_mangle]
pub unsafe extern "C" fn cfgetispeed(termios_p: *const termios) -> speed_t {
    //TODO
    0
}

#[cfg(target_os = "linux")]
#[no_mangle]
pub unsafe extern "C" fn cfgetospeed(termios_p: *const termios) -> speed_t {
    (*termios_p).__c_ospeed
}

#[cfg(target_os = "redox")]
#[no_mangle]
pub unsafe extern "C" fn cfgetospeed(termios_p: *const termios) -> speed_t {
    //TODO
    0
}

#[cfg(target_os = "linux")]
#[no_mangle]
pub unsafe extern "C" fn cfsetispeed(termios_p: *mut termios, speed: speed_t) -> c_int {
    match speed as usize {
        B0..=B38400 | B57600..=B4000000 => {
            (*termios_p).__c_ispeed = speed;
            0
        }
        _ => {
            platform::errno.set(errno::EINVAL);
            -1
        }
    }
}

#[cfg(target_os = "redox")]
#[no_mangle]
pub unsafe extern "C" fn cfsetispeed(termios_p: *mut termios, speed: speed_t) -> c_int {
    //TODO
    platform::errno.set(errno::EINVAL);
    -1
}

#[cfg(target_os = "linux")]
#[no_mangle]
pub unsafe extern "C" fn cfsetospeed(termios_p: *mut termios, speed: speed_t) -> c_int {
    match speed as usize {
        B0..=B38400 | B57600..=B4000000 => {
            (*termios_p).__c_ospeed = speed;
            0
        }
        _ => {
            platform::errno.set(errno::EINVAL);
            -1
        }
    }
}

#[cfg(target_os = "redox")]
#[no_mangle]
pub unsafe extern "C" fn cfsetospeed(termios_p: *mut termios, speed: speed_t) -> c_int {
    //TODO
    platform::errno.set(errno::EINVAL);
    -1
}

#[no_mangle]
pub unsafe extern "C" fn cfsetspeed(termios_p: *mut termios, speed: speed_t) -> c_int {
    let r = cfsetispeed(termios_p, speed);
    if r < 0 {
        return r;
    }
    cfsetospeed(termios_p, speed)
}

#[no_mangle]
pub unsafe extern "C" fn tcflush(fd: c_int, queue: c_int) -> c_int {
    sys_ioctl::ioctl(fd, sys_ioctl::TCFLSH, queue as *mut c_void)
}

#[no_mangle]
pub unsafe extern "C" fn tcdrain(fd: c_int) -> c_int {
    sys_ioctl::ioctl(fd, sys_ioctl::TCSBRK, 1 as *mut _)
}

#[no_mangle]
pub unsafe extern "C" fn tcsendbreak(fd: c_int, _dur: c_int) -> c_int {
    // non-zero duration is ignored by musl due to it being
    // implementation-defined. we do the same.
    sys_ioctl::ioctl(fd, sys_ioctl::TCSBRK, 0 as *mut _)
}

#[no_mangle]
pub unsafe extern "C" fn tcsetwinsize(fd: c_int, mut sws: winsize) -> c_int {
    sys_ioctl::ioctl(fd, sys_ioctl::TIOCSWINSZ, &mut sws as *mut _ as *mut c_void)
}

#[no_mangle]
pub unsafe extern "C" fn tcflow(fd: c_int, action: c_int) -> c_int {
    // non-zero duration is ignored by musl due to it being
    // implementation-defined. we do the same.
    sys_ioctl::ioctl(fd, sys_ioctl::TCXONC, action as *mut _)
}

#[no_mangle]
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
