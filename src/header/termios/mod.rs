//! `termios.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/termios.h.html>.

use crate::{
    header::{errno, sys_ioctl},
    platform::{
        self,
        types::{c_int, c_uchar, c_uint, c_ulong, c_void, pid_t},
    },
};

pub use crate::header::bits_winsize::winsize;

pub use self::sys::*;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

/// Used for terminal special characters.
pub type cc_t = c_uchar;
/// Used for terminal baud rates.
pub type speed_t = c_uint;
/// Used for terminal modes.
pub type tcflag_t = c_uint;

/// Suspend output.
pub const TCOOFF: c_int = 0;
/// Restart output.
pub const TCOON: c_int = 1;
/// Transmit a `STOP` character, intended to suspend input data.
pub const TCIOFF: c_int = 2;
/// Transmit a `START` character, intended to restart input data.
pub const TCION: c_int = 3;

/// Flush pending input.
pub const TCIFLUSH: c_int = 0;
/// Flush untransmitted output.
pub const TCOFLUSH: c_int = 1;
/// Flush bot pending input and untransmitted output.
pub const TCIOFLUSH: c_int = 2;

/// Change attributes immediately.
pub const TCSANOW: c_int = 0;
/// Change attributes when output has drained.
pub const TCSADRAIN: c_int = 1;
/// Change attributes when output has drained; also flush pending input.
pub const TCSAFLUSH: c_int = 2;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/termios.h.html>.
#[cfg(target_os = "linux")]
#[repr(C)]
#[derive(Default, Clone)]
pub struct termios {
    /// Input modes.
    pub c_iflag: tcflag_t,
    /// Output modes.
    pub c_oflag: tcflag_t,
    /// Control modes.
    pub c_cflag: tcflag_t,
    /// Local modes.
    pub c_lflag: tcflag_t,
    pub c_line: cc_t,
    /// Control characters.
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
    /// Input modes.
    pub c_iflag: tcflag_t,
    /// Output modes.
    pub c_oflag: tcflag_t,
    /// Control modes.
    pub c_cflag: tcflag_t,
    /// Local modes.
    pub c_lflag: tcflag_t,
    /// Control characters.
    pub c_cc: [cc_t; NCCS],
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcgetattr.html>.
///
/// Gets the parameters associated with the terminal referred to by `fildes`
/// and stores them in the termios structure referenced by `termios_p`.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcgetattr(fildes: c_int, termios_p: *mut termios) -> c_int {
    unsafe { sys_ioctl::ioctl(fildes, sys_ioctl::TCGETS, termios_p.cast::<c_void>()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcsetattr.html>.
///
/// Sets the parameters associated with the terminal referred to by `fildes`
/// from the termios structure referred to by `termios_p`.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcsetattr(
    fildes: c_int,
    optional_actions: c_int,
    termios_p: *const termios,
) -> c_int {
    match optional_actions {
        TCSANOW | TCSADRAIN | TCSAFLUSH => {
            // This is safe because ioctl shouldn't modify the value
            unsafe {
                sys_ioctl::ioctl(
                    fildes,
                    sys_ioctl::TCSETS + optional_actions as c_ulong,
                    termios_p as *mut c_void,
                )
            }
        }
        _ => {
            platform::ERRNO.set(errno::EINVAL);
            -1
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcgetsid.html>.
///
/// Obtains the process group ID of the session for which the terminal
/// specified by `fildes` is the controlling terminal.
///
/// Upon success, returns the process group ID of the session associated with
/// the terminal. Upon failure, returns `-1` and sets errno to indicate the
/// error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcgetsid(fildes: c_int) -> pid_t {
    let mut sid = 0;
    if unsafe { sys_ioctl::ioctl(fildes, sys_ioctl::TIOCGSID, (&raw mut sid).cast::<c_void>()) } < 0
    {
        return -1;
    }
    sid
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cfgetispeed.html>.
///
/// Extracts the input baud rate from the termios structure reference to by
/// `termios_p`.
///
/// Returns a value representing the input baud rate.
///
/// # Implementation
/// Infallible. Always returns a value.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cfgetispeed(termios_p: *const termios) -> speed_t {
    #[cfg(target_os = "linux")]
    {
        unsafe { (*termios_p).__c_ispeed }
    }
    #[cfg(target_os = "redox")]
    {
        //TODO
        0
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cfgetospeed.html>.
///
/// Extracts the output baud rate from the termios structure reference to by
/// `termios_p`.
///
/// Returns a value representing the output baud rate.
///
/// # Implementation
/// Infallible. Always returns a value.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cfgetospeed(termios_p: *const termios) -> speed_t {
    #[cfg(target_os = "linux")]
    {
        unsafe { (*termios_p).__c_ospeed }
    }
    #[cfg(target_os = "redox")]
    {
        //TODO
        0
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cfsetispeed.html>.
///
/// Sets the input baud rate stored in the structure pointed to by `termios_p`
/// to `speed`.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cfsetispeed(termios_p: *mut termios, speed: speed_t) -> c_int {
    match speed as usize {
        // TODO redox impl
        #[cfg(target_os = "linux")]
        B0..=B38400 | B57600..=B4000000 => {
            unsafe { (*termios_p).__c_ispeed = speed };
            0
        }
        _ => {
            platform::ERRNO.set(errno::EINVAL);
            -1
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cfsetospeed.html>.
///
/// Sets the output baud rate stored in the structure pointed to by `termios_p`
/// to `speed`.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cfsetospeed(termios_p: *mut termios, speed: speed_t) -> c_int {
    match speed as usize {
        // TODO redox impl
        #[cfg(target_os = "linux")]
        B0..=B38400 | B57600..=B4000000 => {
            unsafe { (*termios_p).__c_ospeed = speed };
            0
        }
        _ => {
            platform::ERRNO.set(errno::EINVAL);
            -1
        }
    }
}

// TODO should be guarded by _BSD_SOURCE or _DEFAULT_SOURCE
/// Non-POSIX, 4.4 BSD extension, see
/// <https://www.man7.org/linux/man-pages/man3/cfsetispeed.3.html>.
///
/// Sets both the input and output baud rate stored in the structure pointed to
/// by `termios_p` to `speed`.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
///
/// # Implementation
/// Calls `cfsetispeed()` and if successful then calls `cfsetospeed()`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cfsetspeed(termios_p: *mut termios, speed: speed_t) -> c_int {
    let r = unsafe { cfsetispeed(termios_p, speed) };
    if r < 0 {
        return r;
    }
    unsafe { cfsetospeed(termios_p, speed) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcflush.html>.
///
/// Discards data written to the object referred to by `fildes` but not
/// transmitted, or data received but not read, depending on the value of
/// `queue_selector`.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcflush(fildes: c_int, queue_selector: c_int) -> c_int {
    match queue_selector {
        TCIFLUSH | TCOFLUSH | TCIOFLUSH => unsafe {
            sys_ioctl::ioctl(fildes, sys_ioctl::TCFLSH, queue_selector as *mut c_void)
        },
        _ => {
            platform::ERRNO.set(errno::EINVAL);
            -1
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcdrain.html>.
///
/// Blocks until all output written to the object referred to by `fildes` is
/// transmitted.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcdrain(fildes: c_int) -> c_int {
    unsafe { sys_ioctl::ioctl(fildes, sys_ioctl::TCSBRK, core::ptr::dangling_mut()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcsendbreak.html>.
///
/// Causes transmission of a continuous stream of zero-valued bits for at least
/// 0.25 seconds, and not more than 0.5 seconds.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
///
/// # Implementation
/// This function does not accept a non-zero duration. `_duration` is ignored
/// and assumed to be `0`. Matches behaviour of musl.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcsendbreak(fildes: c_int, _duration: c_int) -> c_int {
    unsafe { sys_ioctl::ioctl(fildes, sys_ioctl::TCSBRK, core::ptr::null_mut()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcgetwinsize.html>.
///
/// Gets the terminal window size associated with the terminal referred to by
/// `fildes` and stores it in the winsize structure pointed to by `winsize_p`.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcgetwinsize(fildes: c_int, winsize_p: *mut winsize) -> c_int {
    unsafe { sys_ioctl::ioctl(fildes, sys_ioctl::TIOCGWINSZ, winsize_p.cast()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcsetwinsize.html>.
///
/// Sets the terminal window size associated with the terminal referred to by
/// `fildes` and stores it in the winsize structure pointed to by `winsize_p`.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
///
/// # Safety
/// It is undefined behaviour if the value of the winsize structure pointed to
/// by `winsize_p` was not derived from the result of a call to
/// `tcsgetwinsize()` on `fildes`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcsetwinsize(fildes: c_int, winsize_p: *const winsize) -> c_int {
    unsafe { sys_ioctl::ioctl(fildes, sys_ioctl::TIOCSWINSZ, winsize_p.cast_mut().cast()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcflow.html>.
///
/// Suspends or restarts transmission or reception of data on the object
/// referred to by `fildes`, depending on the value of `action`.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcflow(fildes: c_int, action: c_int) -> c_int {
    match action {
        TCIOFF | TCION | TCOOFF | TCOON => unsafe {
            sys_ioctl::ioctl(fildes, sys_ioctl::TCXONC, action as *mut _)
        },
        _ => {
            platform::ERRNO.set(errno::EINVAL);
            -1
        }
    }
}

// TODO should be guarded by _BSD_SOURCE or _DEFAULT_SOURCE
/// Non-POSIX, BSD extension, see <https://www.man7.org/linux/man-pages/man3/cfmakeraw.3.html>.
///
/// Sets the terminal to something similar to "raw" mode: input is available
/// character by character, echoing is disabled, and all special processing of
/// terminal input and output characters is disabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cfmakeraw(termios_p: *mut termios) {
    unsafe {
        (*termios_p).c_iflag &=
            !(IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON) as u32;
        (*termios_p).c_oflag &= !OPOST as u32;
        (*termios_p).c_lflag &= !(ECHO | ECHONL | ICANON | ISIG | IEXTEN) as u32;
        (*termios_p).c_cflag &= !(CSIZE | PARENB) as u32;
        (*termios_p).c_cflag |= CS8 as u32;
        (*termios_p).c_cc[VMIN] = 1;
        (*termios_p).c_cc[VTIME] = 0;
    }
}
