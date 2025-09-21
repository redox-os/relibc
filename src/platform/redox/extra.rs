use core::{ptr, slice};

use crate::{
    error::{Errno, ResultExt},
    platform::types::*,
};
use syscall::{error::*, F_SETFD, F_SETFL};

pub use redox_rt::proc::FdGuard;

#[no_mangle]
pub unsafe extern "C" fn redox_fpath(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    syscall::fpath(
        fd as usize,
        slice::from_raw_parts_mut(buf as *mut u8, count),
    )
    .map_err(Errno::from)
    .map(|l| l as ssize_t)
    .or_minus_one_errno()
}

pub fn pipe2(flags: usize) -> syscall::error::Result<[c_int; 2]> {
    let mut read_fd = FdGuard::new(syscall::open("/scheme/pipe", flags)?);
    let mut write_fd = FdGuard::new(syscall::dup(*read_fd, b"write")?);
    syscall::fcntl(*write_fd, F_SETFL, flags)?;
    syscall::fcntl(*write_fd, F_SETFD, flags)?;

    let fds = [
        c_int::try_from(*read_fd).map_err(|_| Error::new(EMFILE))?,
        c_int::try_from(*write_fd).map_err(|_| Error::new(EMFILE))?,
    ];

    read_fd.take();
    write_fd.take();

    Ok(fds)
}
