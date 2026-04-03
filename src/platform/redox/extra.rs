use core::slice;

use crate::{
    error::{Errno, ResultExt},
    platform::types::*,
};
use syscall::{F_SETFD, F_SETFL, O_RDONLY, O_WRONLY, error::*};

pub use redox_rt::proc::FdGuard;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_fpath(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    syscall::fpath(fd as usize, unsafe {
        slice::from_raw_parts_mut(buf as *mut u8, count)
    })
    .map_err(Errno::from)
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
