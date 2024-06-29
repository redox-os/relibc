use syscall::error::{Result, Error, EINTR};

use crate::arch::manually_enter_trampoline;
use crate::signal::tmp_disable_signals;

// TODO: uninitialized memory?
#[inline]
pub fn posix_read(fd: usize, buf: &mut [u8]) -> Result<usize> {
    loop {
        let res = syscall::read(fd, buf);

        if res == Err(Error::new(EINTR)) {
            unsafe {
                manually_enter_trampoline();
            }
        }

        return res;
    }
}
#[inline]
pub fn posix_write(fd: usize, buf: &[u8]) -> Result<usize> {
    loop {
        let res = syscall::write(fd, buf);

        if res == Err(Error::new(EINTR)) {
            unsafe {
                manually_enter_trampoline();
            }
        }

        return res;

    }
}
