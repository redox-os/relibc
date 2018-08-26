#![allow(non_camel_case_types)]

use libc::{self, c_int};
use redox_termios::{tcflag_t, Termios};
use syscall::{self, EINVAL, Error};
use ::types::pid_t;

type speed_t = libc::c_uint;

// tcsetattr args
pub const TCSANOW:   tcflag_t = 0x1;
pub const TCSADRAIN: tcflag_t = 0x2;
pub const TCSAFLUSH: tcflag_t = 0x4;

// tcflush args
pub const TCIFLUSH:  tcflag_t = 0x1;
pub const TCIOFLUSH: tcflag_t = 0x3;
pub const TCOFLUSH:  tcflag_t = 0x2;

// tcflow args
pub const TCIOFF: tcflag_t = 0x1;
pub const TCION:  tcflag_t = 0x2;
pub const TCOOFF: tcflag_t = 0x4;
pub const TCOON:  tcflag_t = 0x8;

libc_fn!(unsafe tcgetattr(fd: c_int, tio: *mut Termios) -> Result<c_int> {
    let fd = syscall::dup(fd as usize, b"termios")?;
    let res = syscall::read(fd, &mut *tio);
    let _ = syscall::close(fd);

    if res? == (&*tio).len() {
        Ok(0)
    } else {
        Err(Error::new(EINVAL))
    }
});

libc_fn!(unsafe tcsetattr(fd: c_int, _arg: c_int, tio: *mut Termios) -> Result<c_int> {
    let fd = syscall::dup(fd as usize, b"termios")?;
    let res = syscall::write(fd, &*tio);
    let _ = syscall::close(fd);

    if res? == (&*tio).len() {
        Ok(0)
    } else {
        Err(Error::new(EINVAL))
    }
});

libc_fn!(cfgetispeed(_tio: *mut Termios) -> speed_t {
    // TODO
    0 as speed_t
});

libc_fn!(cfgetospeed(_tio: *mut Termios) -> speed_t {
    // TODO
    0 as speed_t
});

libc_fn!(cfsetispeed(_tio: *mut Termios, _speed: speed_t) -> Result<c_int> {
    // TODO
    Ok(0)
});

libc_fn!(cfsetospeed(_tio: *mut Termios, _speed: speed_t) -> Result<c_int> {
    // TODO
    Ok(0)
});

libc_fn!(tcdrain(_i: c_int) -> Result<c_int> {
    // TODO
    Ok(0)
});

libc_fn!(tcflow(_fd: c_int, _arg: c_int) -> Result<c_int> {
    // TODO
    Ok(0)
});

libc_fn!(tcflush(_fd: c_int, _arg: c_int) -> Result<c_int> {
    // TODO
    Ok(0)
});

libc_fn!(unsafe tcgetsid(_fd: c_int) -> pid_t {
    // TODO
    ::process::_getpid()
});

libc_fn!(tcsendbreak(_fd: c_int, _arg: c_int) -> Result<c_int> {
    // TODO
    Ok(0)
});
