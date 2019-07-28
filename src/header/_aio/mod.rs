use crate::{
    header::time::{sigevent, timespec},
    platform::types::*,
};

pub struct aiocb {
    pub aio_fildes: c_int,
    pub aio_lio_opcode: c_int,
    pub aio_reqprio: c_int,
    pub aio_buf: *mut c_void,
    pub aio_nbytes: usize,
    pub aio_sigevent: sigevent,
}

// #[no_mangle]
pub extern "C" fn aio_read(aiocbp: *mut aiocb) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn aio_write(aiocbp: *mut aiocb) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn lio_listio(
    mode: c_int,
    list: *const *const aiocb,
    nent: c_int,
    sig: *mut sigevent,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn aio_error(aiocbp: *const aiocb) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn aio_return(aiocbp: *mut aiocb) -> usize {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn aio_cancel(fildes: c_int, aiocbp: *mut aiocb) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn aio_suspend(
    list: *const *const aiocb,
    nent: c_int,
    timeout: *const timespec,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn aio_fsync(operation: c_int, aiocbp: *mut aiocb) -> c_int {
    unimplemented!();
}
