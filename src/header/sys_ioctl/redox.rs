use core::{mem, slice};
use redox_rt::proc::FdGuard;
use syscall;

use crate::{
    error::{Errno, Result, ResultExt},
    header::{
        errno::{self, EINVAL},
        fcntl, termios,
    },
    platform::{self, Pal, Sys, types::*},
};

use super::winsize;

pub const TCGETS: c_ulong = 0x5401;
pub const TCSETS: c_ulong = 0x5402;
pub const TCSETSW: c_ulong = 0x5403;
pub const TCSETSF: c_ulong = 0x5404;

pub const TCSBRK: c_ulong = 0x5409;
pub const TCXONC: c_ulong = 0x540A;
pub const TCFLSH: c_ulong = 0x540B;

pub const TIOCSCTTY: c_ulong = 0x540E;
pub const TIOCGPGRP: c_ulong = 0x540F;
pub const TIOCSPGRP: c_ulong = 0x5410;

pub const TIOCGWINSZ: c_ulong = 0x5413;
pub const TIOCSWINSZ: c_ulong = 0x5414;

pub const FIONREAD: c_ulong = 0x541B;

pub const FIONBIO: c_ulong = 0x5421;

pub const TIOCSPTLCK: c_ulong = 0x4004_5431;
pub const TIOCGPTLCK: c_ulong = 0x8004_5439;

pub const SIOCATMARK: c_ulong = 0x8905;

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

// FIXME: unsound
fn dup_write<T>(fd: c_int, name: &str, t: &T) -> Result<usize> {
    let dup = FdGuard::new(syscall::dup(fd as usize, name.as_bytes())?);

    let size = mem::size_of::<T>();

    let bytes_written = syscall::write(*dup, unsafe {
        slice::from_raw_parts(t as *const T as *const u8, size)
    })?;

    Ok(bytes_written / size)
}

unsafe fn ioctl_inner(fd: c_int, request: c_ulong, out: *mut c_void) -> Result<c_int> {
    match request {
        FIONBIO => {
            let mut flags = Sys::fcntl(fd, fcntl::F_GETFL, 0)?;
            flags = if *(out as *mut c_int) == 0 {
                flags & !fcntl::O_NONBLOCK
            } else {
                flags | fcntl::O_NONBLOCK
            };
            Sys::fcntl(fd, fcntl::F_SETFL, flags as c_ulonglong)?;
        }
        TCGETS => {
            let termios = &mut *(out as *mut termios::termios);
            dup_read(fd, "termios", termios)?;
        }
        // TODO: give these different behaviors
        TCSETS | TCSETSW | TCSETSF => {
            let termios = &*(out as *const termios::termios);
            dup_write(fd, "termios", termios)?;
        }
        TCFLSH => {
            let queue = out as c_int;
            dup_write(fd, "flush", &queue)?;
        }
        TIOCSCTTY => {
            eprintln!("TODO: ioctl TIOCSCTTY");
        }
        TIOCGPGRP => {
            let pgrp = &mut *(out as *mut pid_t);
            dup_read(fd, "pgrp", pgrp)?;
        }
        TIOCSPGRP => {
            let pgrp = &*(out as *const pid_t);
            dup_write(fd, "pgrp", pgrp)?;
        }
        TIOCGWINSZ => {
            let winsize = &mut *(out as *mut winsize);
            dup_read(fd, "winsize", winsize)?;
        }
        TIOCSWINSZ => {
            let winsize = &*(out as *const winsize);
            dup_write(fd, "winsize", winsize)?;
        }
        TIOCGPTLCK => {
            eprintln!("TODO: ioctl TIOCGPTLCK");
        }
        TIOCSPTLCK => {
            eprintln!("TODO: ioctl TIOCSPTLCK");
        }
        TCSBRK => {
            eprintln!("TODO: ioctl TCSBRK");
        }
        TCXONC => {
            eprintln!("TODO: ioctl TCXONC");
        }
        SIOCATMARK => {
            eprintln!("TODO: ioctl SIOCATMARK");
        }
        _ => {
            return Err(Errno(EINVAL));
        }
    }
    Ok(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ioctl(fd: c_int, request: c_ulong, out: *mut c_void) -> c_int {
    ioctl_inner(fd, request, out).or_minus_one_errno()
}
