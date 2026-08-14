use core::{mem, ptr, slice};
use syscall::{self, flag::CallFlags};

use crate::{
    error::{Errno, Result},
    header::{bits_winsize::winsize, errno::EINVAL, fcntl, termios},
    platform::{
        Pal, Sys,
        types::{c_int, c_ulong, c_ulonglong, c_void, pid_t},
    },
};

use super::constants::*;

mod drm;

// TODO: some of the structs passed as T have padding bytes, so casting to a byte slice is UB

fn sys_call_read<T>(fd: c_int, t: &mut T) -> syscall::Result<usize> {
    let size = mem::size_of::<T>();

    let payload =
        unsafe { slice::from_raw_parts_mut(core::ptr::from_mut::<T>(t).cast::<u8>(), size) };

    let bytes = redox_rt::sys::sys_call_ro(fd as usize, payload, CallFlags::READ, &[])?;

    Ok(bytes / size)
}

// FIXME: unsound
fn sys_call_write<T>(fd: c_int, t: &T) -> Result<usize> {
    let size = mem::size_of::<T>();

    let payload = unsafe { slice::from_raw_parts(core::ptr::from_ref::<T>(t).cast::<u8>(), size) };

    let bytes = redox_rt::sys::sys_call_wo(fd as usize, payload, CallFlags::WRITE, &[])?;

    Ok(bytes / size)
}

#[derive(Debug)]
enum IoctlBuffer {
    None,
    Read(*mut c_void, usize),    // read (write to userspace)
    Write(*const c_void, usize), // write (read from userspace)
    ReadWrite(*mut c_void, usize),
}

impl IoctlBuffer {
    unsafe fn read<T>(&self) -> Result<T> {
        let (ptr, size) = match *self {
            Self::Write(ptr, size) => (ptr, size),
            Self::ReadWrite(ptr, size) => (ptr.cast_const(), size),
            _ => {
                return Err(Errno(EINVAL));
            }
        };
        if size == mem::size_of::<T>() {
            let value = unsafe { ptr::read(ptr.cast::<T>()) };
            Ok(value)
        } else {
            Err(Errno(EINVAL))
        }
    }

    unsafe fn write<T>(&mut self, value: T) -> Result<()> {
        let (Self::Read(ptr, size) | Self::ReadWrite(ptr, size)) = *self else {
            return Err(Errno(EINVAL));
        };
        if size == mem::size_of::<T>() {
            unsafe { ptr::write(ptr.cast::<T>(), value) };
            Ok(())
        } else {
            Err(Errno(EINVAL))
        }
    }
}

pub unsafe fn ioctl_inner(fd: c_int, request: c_ulong, out: *mut c_void) -> Result<c_int> {
    match request {
        FIONBIO => {
            let mut flags = Sys::fcntl(fd, fcntl::F_GETFL, 0)?;
            flags = if unsafe { *out.cast::<c_int>() } == 0 {
                flags & !fcntl::O_NONBLOCK
            } else {
                flags | fcntl::O_NONBLOCK
            };
            Sys::fcntl(fd, fcntl::F_SETFL, flags as c_ulonglong)?;
        }
        // tcgetattr()
        TCGETS => {
            let termios = unsafe { &mut *out.cast::<termios::termios>() };
            sys_call_read(fd, termios)?;
        }
        // TODO: give these different behaviors
        TCSETS | TCSETSW | TCSETSF => {
            let termios = unsafe { &*(out as *const termios::termios) };
            sys_call_write(fd, termios)?;
        }
        // tcflush()
        TCFLSH => {
            let queue = out as c_int;
            sys_call_write(fd, &queue)?;
        }
        // tcsendbreak() and tcdrain()
        TCSBRK => {
            // tcsendbreak == ioctl(TCSBRK, 0)
            // tcdrain == ioctl(TCSBRK, <nonzero>)
            let duration = out as c_int;
            sys_call_write(fd, &duration)?;
        }
        // tcflow()
        TCXONC => {
            let arg = out as c_int;
            sys_call_write(fd, &arg)?;
        }
        TIOCSCTTY => {
            todo_skip!(0, "ioctl TIOCSCTTY");
        }
        // tcgetpgrp()
        TIOCGPGRP => {
            let pgrp = unsafe { &mut *out.cast::<pid_t>() };
            sys_call_read(fd, pgrp)?;
        }
        // tcsetpgrp()
        TIOCSPGRP => {
            let pgrp = unsafe { *(out as *const pid_t) };
            sys_call_write(fd, &pgrp)?;
        }
        // tcgetwinsize()
        TIOCGWINSZ => {
            let winsize = unsafe { &mut *out.cast::<winsize>() };
            sys_call_read(fd, winsize)?;
        }
        // tcsetwinsize()
        TIOCSWINSZ => {
            let winsize = unsafe { &*(out as *const winsize) };
            sys_call_write(fd, winsize)?;
        }
        TIOCGPTLCK => {
            let lock = unsafe { &mut *out.cast::<c_int>() };
            sys_call_read(fd, lock)?;
        }
        TIOCSPTLCK => {
            let lock = unsafe { *(out as *const c_int) };
            sys_call_write(fd, &lock)?;
        }
        TIOCGPTN => {
            let name = unsafe { &mut *out.cast::<c_int>() };
            sys_call_read(fd, name)?;
        }
        SIOCATMARK => {
            todo_skip!(0, "ioctl SIOCATMARK");
        }
        _ => {
            // See https://docs.kernel.org/userspace-api/ioctl/ioctl-decoding.html for details
            let dir = (request >> 30) & 0b11;
            let size = ((request >> 16) & 0x3FFF) as usize;
            let name = (((request >> 8) & 0xFF) as u8) as char;
            let func = (request & 0xFF) as u8;
            match name {
                'd' => {
                    let buf = match dir {
                        0b10 => IoctlBuffer::Read(out, size),
                        0b01 => IoctlBuffer::Write(out, size),
                        0b11 => IoctlBuffer::ReadWrite(out, size),
                        _ => IoctlBuffer::None,
                    };
                    return unsafe { drm::ioctl(fd, func, buf) };
                }
                _ => {
                    return Err(Errno(EINVAL));
                }
            }
        }
    }
    Ok(0)
}
