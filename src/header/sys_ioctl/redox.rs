use core::{mem, slice};
use syscall;

use crate::{
    header::{errno, fcntl, termios},
    platform::{self, e, types::*},
};

use super::winsize;

pub const FIONBIO: c_ulong = 0x5421;

pub const TCGETS: c_ulong = 0x5401;
pub const TCSETS: c_ulong = 0x5402;
pub const TCSETSW: c_ulong = 0x5403;
pub const TCSETSF: c_ulong = 0x5404;
pub const TCSBRK: c_ulong = 0x5409;

pub const TCXONC: c_ulong = 0x540A;

pub const TCFLSH: c_ulong = 0x540B;

pub const TIOCGPGRP: c_ulong = 0x540F;
pub const TIOCSPGRP: c_ulong = 0x5410;

pub const TIOCGWINSZ: c_ulong = 0x5413;
pub const TIOCSWINSZ: c_ulong = 0x5414;

// TODO: some of the structs passed as T have padding bytes, so casting to a byte slice is UB

fn dup_read<T>(fd: c_int, name: &str, t: &mut T) -> syscall::Result<usize> {
    let dup = syscall::dup(fd as usize, name.as_bytes())?;

    let size = mem::size_of::<T>();

    let res = syscall::read(dup, unsafe {
        slice::from_raw_parts_mut(t as *mut T as *mut u8, size)
    });

    let _ = syscall::close(dup);

    res.map(|bytes| bytes / size)
}

fn dup_write<T>(fd: c_int, name: &str, t: &T) -> syscall::Result<usize> {
    let dup = syscall::dup(fd as usize, name.as_bytes())?;

    let size = mem::size_of::<T>();

    let res = syscall::write(dup, unsafe {
        slice::from_raw_parts(t as *const T as *const u8, size)
    });

    let _ = syscall::close(dup);

    res.map(|bytes| bytes / size)
}

#[no_mangle]
pub unsafe extern "C" fn ioctl(fd: c_int, request: c_ulong, out: *mut c_void) -> c_int {
    match request {
        FIONBIO => {
            let mut flags = fcntl::sys_fcntl(fd, fcntl::F_GETFL, 0);
            if flags < 0 {
                return -1;
            }
            flags = if *(out as *mut c_int) == 0 {
                flags & !fcntl::O_NONBLOCK
            } else {
                flags | fcntl::O_NONBLOCK
            };
            if fcntl::sys_fcntl(fd, fcntl::F_SETFL, flags) < 0 {
                -1
            } else {
                0
            }
        }
        TCGETS => {
            let termios = &mut *(out as *mut termios::termios);
            if e(dup_read(fd, "termios", termios)) == !0 {
                -1
            } else {
                0
            }
        }
        // TODO: give these different behaviors
        TCSETS | TCSETSW | TCSETSF => {
            let termios = &*(out as *const termios::termios);
            if e(dup_write(fd, "termios", termios)) == !0 {
                -1
            } else {
                0
            }
        }
        TCFLSH => {
            let queue = out as c_int;
            if e(dup_write(fd, "flush", &queue)) == !0 {
                -1
            } else {
                0
            }
        }
        TIOCGPGRP => {
            let pgrp = &mut *(out as *mut pid_t);
            if e(dup_read(fd, "pgrp", pgrp)) == !0 {
                -1
            } else {
                0
            }
        }
        TIOCSPGRP => {
            let pgrp = &*(out as *const pid_t);
            if e(dup_write(fd, "pgrp", pgrp)) == !0 {
                -1
            } else {
                0
            }
        }
        TIOCGWINSZ => {
            let winsize = &mut *(out as *mut winsize);
            if e(dup_read(fd, "winsize", winsize)) == !0 {
                -1
            } else {
                0
            }
        }
        TIOCSWINSZ => {
            let winsize = &*(out as *const winsize);
            if e(dup_write(fd, "winsize", winsize)) == !0 {
                -1
            } else {
                0
            }
        }
        TCSBRK => {
            // TODO
            0
        }
        TCXONC => {
            // TODO
            0
        }
        _ => {
            platform::errno = errno::EINVAL;
            -1
        }
    }
}
