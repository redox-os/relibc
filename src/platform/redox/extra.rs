use core::slice;

use crate::{
    error::{Errno, ResultExt}, platform::{Pal, sys, types::*}
};

use alloc::string::ToString;
use syscall::{F_SETFD, F_SETFL, O_RDONLY, O_WRONLY, error::*};
pub use redox_rt::proc::FdGuard;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_fpath(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {

    syscall::write(2,b"fpath\n").expect("test");

    syscall::write(2,&fd.to_string().as_bytes()).expect("test");

    syscall::write(2,b"<--FD\n").expect("test");
    syscall::fpath(fd as usize, unsafe {
        slice::from_raw_parts_mut(buf as *mut u8, count)
    })
    .map_err(Errno::from)
    .map(|l| l as ssize_t)
    .or_minus_one_errno()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rfpath(fd: c_int, out: *mut c_void, count: size_t) -> ssize_t{
    sys::Sys::fpath(fd,unsafe { slice::from_raw_parts_mut(out as *mut u8, count) })
    .map(|l| l as ssize_t)
    .or_minus_one_errno()
}

pub fn pipe2(flags: usize) -> syscall::error::Result<[c_int; 2]> {
    let read_flags = flags | O_RDONLY;
    let write_flags = flags | O_WRONLY;
    let read_fd = FdGuard::open("/scheme/pipe", read_flags)?;
    let write_fd = read_fd.dup(b"write")?;
    write_fd.fcntl(F_SETFL, write_flags)?;
    write_fd.fcntl(F_SETFD, write_flags)?;

    let fds = [
        c_int::try_from(read_fd.as_raw_fd()).map_err(|_| Error::new(EMFILE))?,
        c_int::try_from(write_fd.as_raw_fd()).map_err(|_| Error::new(EMFILE))?,
    ];

    read_fd.take();
    write_fd.take();

    Ok(fds)
}
