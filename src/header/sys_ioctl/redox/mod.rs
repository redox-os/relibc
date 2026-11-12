use core::{mem, ptr, slice};
use redox_protocols::protocol::{ProcCall, TtyCall};
use syscall::{self, flag::CallFlags};

use crate::{
    error::{Errno, Result},
    header::{
        bits_winsize::winsize,
        errno::{EINVAL, ENOTTY, EPERM},
        fcntl,
        sys_ioctl::constants::{
            FIONBIO, SIOCATMARK, TCFLSH, TCGETS, TCSBRK, TCSETS, TCSETSF, TCSETSW, TCXONC,
            TIOCGPGRP, TIOCGPTLCK, TIOCGPTN, TIOCGSID, TIOCGWINSZ, TIOCSCTTY, TIOCSPGRP,
            TIOCSPTLCK, TIOCSWINSZ,
        },
        sys_wait, termios, unistd,
    },
    platform::{
        Pal, Sys,
        types::{c_int, c_ulong, c_ulonglong, c_void, pid_t},
    },
};

// TODO: Find a better way to represent whether each call is read or write
const REQ_READ: u64 = 0;
const REQ_WRITE: u64 = 1;

mod drm;

// TODO: some of the structs passed as T have padding bytes, so casting to a byte slice is UB

fn sys_call_read<T>(fd: c_int, verb: TtyCall, t: &mut T) -> syscall::Result<usize> {
    let size = mem::size_of::<T>();

    let payload =
        unsafe { slice::from_raw_parts_mut(core::ptr::from_mut::<T>(t).cast::<u8>(), size) };

    let bytes = redox_rt::sys::sys_call_ro(
        fd as usize,
        payload,
        CallFlags::READ,
        &[verb as u64, REQ_READ],
    )?;

    Ok(bytes / size)
}

// FIXME: unsound
fn sys_call_write<T>(fd: c_int, verb: TtyCall, t: &T) -> Result<usize> {
    let size = mem::size_of::<T>();

    let payload = unsafe { slice::from_raw_parts(core::ptr::from_ref::<T>(t).cast::<u8>(), size) };

    let bytes = redox_rt::sys::sys_call_wo(
        fd as usize,
        payload,
        CallFlags::WRITE,
        &[verb as u64, REQ_WRITE],
    )?;

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
            sys_call_read(fd, TtyCall::Termios, termios)?;
        }
        // TODO: give these different behaviors
        TCSETS | TCSETSW | TCSETSF => {
            let termios = unsafe { &*(out as *const termios::termios) };
            sys_call_write(fd, TtyCall::Termios, termios)?;
        }
        // tcflush()
        TCFLSH => {
            let queue = out as c_int;
            sys_call_write(fd, TtyCall::Flush, &queue)?;
        }
        // tcsendbreak() and tcdrain()
        TCSBRK => {
            // tcsendbreak == ioctl(TCSBRK, 0)
            // tcdrain == ioctl(TCSBRK, <nonzero>)
            let duration = out as c_int;
            sys_call_write(fd, TtyCall::SendBreak, &duration)?;
        }
        // tcflow()
        TCXONC => {
            let arg = out as c_int;
            sys_call_write(fd, TtyCall::Flow, &arg)?;
        }
        TIOCSCTTY => {
            // fd, TIOCSCTTY, 0 (set cont term for caller session)
            // fd, TIOCSCTTY, 1 (steal cont term from another session for caller session)
            // both cases require caller to be session leader and caller session to have no cont term
            let arg = out as c_int;
            // only 0 and 1 are valid values for arg
            let should_steal = match arg {
                0 => 0, // don't steal
                1 => 1, // steal
                _ => return Err(Errno(EINVAL)),
            };
            let pid = unistd::getpid();
            let sid = unistd::getsid(0);
            // check if caller is session leader
            if pid != sid {
                return Err(Errno(EPERM));
            }
            // fd should refer to a terminal
            // TODO: remove this check? seems to be assumed that caller should ensure this beforehand
            if unistd::isatty(fd) == 0 {
                return Err(Errno(ENOTTY));
            }
            let procfd_usize = redox_rt::current_proc_fd().as_raw_fd();
            let target_group = usize::try_from(pid).expect("positive and within bounds");
            match redox_rt::sys::openat(procfd_usize, format!("pgrp/{target_group}"), 0, 0) {
                // caller session does not have a controlling terminal
                Err(e) if Errno(ENOTTY) == e.into() => {
                    // create a new process to assign the controlling terminal from
                    let child = unsafe { unistd::fork() };
                    // only executes in child process
                    if child == 0 {
                        let procfd_usize = redox_rt::current_proc_fd().as_raw_fd();
                        let _ = unistd::setsid();
                        // send pty to procmgr to become controlling terminal for the session
                        redox_rt::sys::sys_call_wo(
                            redox_rt::sys::dup(procfd_usize, b"proc-obj").expect("process exists"),
                            &redox_rt::sys::dup(fd as usize, &[])
                                .expect("exists")
                                .to_ne_bytes(),
                            CallFlags::FD | CallFlags::FD_CLONE, // on_sendfd in procmgr
                            &[ProcCall::ControlTerm as u64, should_steal],
                        )?;
                        // send process group handle to ptyd
                        redox_rt::sys::sys_call_wo(
                            redox_rt::sys::dup(fd as usize, &[])
                                .expect("positive and within bounds"), // the pty fd
                            &redox_rt::sys::dup(procfd_usize, b"pgrp-obj")
                                .expect("process exists")
                                .to_ne_bytes(),
                            CallFlags::FD | CallFlags::FD_CLONE, // on_sendfd in ptyd
                            &[],
                        )?;
                        unistd::_exit(0);
                    }
                    // wait for child process to finish
                    let _ = unsafe { sys_wait::waitpid(child, ptr::null_mut(), 0) };
                }
                // caller session already has a controlling terminal
                Ok(pgrpobj_handle) => {
                    let _ = unistd::close(pgrpobj_handle as c_int);
                    return Err(Errno(EPERM));
                }
                Err(e) => return Err(Errno::from(e)),
            }
        }
        // tcgetpgrp() and tcgetsid()
        TIOCGPGRP | TIOCGSID => {
            let pgrp = unsafe { &mut *out.cast::<pid_t>() };
            sys_call_read(fd, TtyCall::Pgrp, pgrp)?;
            if request == TIOCGSID {
                *pgrp = unistd::getsid(*pgrp);
            }
        }
        // tcsetpgrp()
        TIOCSPGRP => {
            let pgrp = unsafe { *(out as *const pid_t) };
            sys_call_write(fd, TtyCall::Pgrp, &pgrp)?;
            // TODO restore below after TIOCSCTTY
            /*
            if unistd::isatty(fd) == 0 {
                return Err(Errno(ENOTTY));
            }
            let pgrp = unsafe { *(out as *const pid_t) };
            let procfd_usize = redox_rt::current_proc_fd().as_raw_fd();
            let target_group = pgrp as usize;
            let pgrpobj_handle =
                redox_rt::sys::openat(procfd_usize, format!("pgrp/{target_group}"), 0, 0)?;
            redox_rt::sys::sys_call_wo(
                fd as usize,
                &pgrpobj_handle.to_ne_bytes(),
                CallFlags::FD, // on_sendfd in ptyd
                &[],
            )?;
            */
        }
        // tcgetwinsize()
        TIOCGWINSZ => {
            let winsize = unsafe { &mut *out.cast::<winsize>() };
            sys_call_read(fd, TtyCall::Winsize, winsize)?;
        }
        // tcsetwinsize()
        TIOCSWINSZ => {
            let winsize = unsafe { &*(out as *const winsize) };
            sys_call_write(fd, TtyCall::Winsize, winsize)?;
        }
        TIOCGPTLCK => {
            let lock = unsafe { &mut *out.cast::<c_int>() };
            sys_call_read(fd, TtyCall::PtLock, lock)?;
        }
        TIOCSPTLCK => {
            let lock = unsafe { *(out as *const c_int) };
            sys_call_write(fd, TtyCall::PtLock, &lock)?;
        }
        TIOCGPTN => {
            let name = unsafe { &mut *out.cast::<c_int>() };
            sys_call_read(fd, TtyCall::PtsName, name)?;
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
