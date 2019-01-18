use core::{mem, slice};
use syscall;

use header::errno;
use header::termios;
use platform;
use platform::e;
use platform::types::*;

use super::winsize;

#[no_mangle]
pub unsafe extern "C" fn ioctl(fd: c_int, request: c_ulong, out: *mut c_void) -> c_int {
    match request {
        TCGETS => {
            let dup = e(syscall::dup(fd as usize, b"termios"));
            if dup == !0 {
                return -1;
            }

            let count = e(syscall::read(dup, unsafe {
                slice::from_raw_parts_mut(out as *mut u8, mem::size_of::<termios::termios>())
            }));
            let _ = syscall::close(dup);

            if count == !0 {
                return -1;
            }
            0
        }

        TCSETS => {
            let dup = e(syscall::dup(fd as usize, b"termios"));
            if dup == !0 {
                return -1;
            }

            let count = e(syscall::write(dup, unsafe {
                slice::from_raw_parts(out as *const u8, mem::size_of::<termios::termios>())
            }));
            let _ = syscall::close(dup);

            if count == !0 {
                return -1;
            }
            0
        },
        TIOCGPGRP => {
            let dup = e(syscall::dup(fd as usize, b"pgrp"));
            if dup == !0 {
                return -1;
            }

            let count = e(syscall::read(
                dup,
                slice::from_raw_parts_mut(out as *mut u8, mem::size_of::<pid_t>())
            ));
            let _ = syscall::close(dup);

            if count == !0 {
                return -1;
            }
            0
        },
        TIOCSPGRP => {
            let dup = e(syscall::dup(fd as usize, b"pgrp"));
            if dup == !0 {
                return -1;
            }

            let count = e(syscall::write(
                dup,
                slice::from_raw_parts(out as *const u8, mem::size_of::<pid_t>())
            ));
            let _ = syscall::close(dup);

            if count == !0 {
                return -1;
            }
            0
        },
        TIOCGWINSZ => {
            let dup = e(syscall::dup(fd as usize, b"winsize"));
            if dup == !0 {
                return -1;
            }

            let count = e(syscall::read(
                dup,
                slice::from_raw_parts_mut(out as *mut u8, mem::size_of::<winsize>())
            ));
            let _ = syscall::close(dup);

            if count == !0 {
                return -1;
            }
            0
        },
        TIOCSWINSZ => {
            let dup = e(syscall::dup(fd as usize, b"winsize"));
            if dup == !0 {
                return -1;
            }

            let count = e(syscall::write(
                dup,
                slice::from_raw_parts(out as *const u8, mem::size_of::<winsize>())
            ));
            let _ = syscall::close(dup);

            if count == !0 {
                return -1;
            }
            0
        },
        _ => {
            platform::errno = errno::EINVAL;
            -1
        }
    }
}

pub const TCGETS: c_ulong = 0x5401;
pub const TCSETS: c_ulong = 0x5402;

pub const TIOCGPGRP: c_ulong = 0x540F;
pub const TIOCSPGRP: c_ulong = 0x5410;

pub const TIOCGWINSZ: c_ulong = 0x5413;
pub const TIOCSWINSZ: c_ulong = 0x5414;
