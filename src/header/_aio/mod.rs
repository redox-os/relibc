//! `aio.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/aio.h.html>.

use crate::{
    header::{signal::sigevent, time::timespec},
    platform::types::{c_int, c_void},
};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/aio.h.html>.
pub struct aiocb {
    pub aio_fildes: c_int,
    pub aio_lio_opcode: c_int,
    pub aio_reqprio: c_int,
    pub aio_buf: *mut c_void,
    pub aio_nbytes: usize,
    pub aio_sigevent: sigevent,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/aio_read.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn aio_read(aiocbp: *mut aiocb) -> c_int {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/aio_write.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn aio_write(aiocbp: *mut aiocb) -> c_int {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/lio_listio.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn lio_listio(
    mode: c_int,
    list: *const *const aiocb,
    nent: c_int,
    sig: *mut sigevent,
) -> c_int {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/aio_error.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn aio_error(aiocbp: *const aiocb) -> c_int {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/aio_return.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn aio_return(aiocbp: *mut aiocb) -> usize {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/aio_cancel.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn aio_cancel(fildes: c_int, aiocbp: *mut aiocb) -> c_int {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/aio_suspend.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn aio_suspend(
    list: *const *const aiocb,
    nent: c_int,
    timeout: *const timespec,
) -> c_int {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/aio_fsync.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn aio_fsync(operation: c_int, aiocbp: *mut aiocb) -> c_int {
    unimplemented!();
}
