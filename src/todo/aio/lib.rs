pub struct aiocb {
    pub aio_fildes: libc::c_int,
    pub aio_lio_opcode: libc::c_int,
    pub aio_reqprio: libc::c_int,
    pub aio_buf: *mut libc::c_void,
    pub aio_nbytes: usize,
    pub aio_sigevent: sigevent,
}

#[no_mangle]
pub extern "C" fn aio_read(aiocbp: *mut aiocb) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn aio_write(aiocbp: *mut aiocb) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn lio_listio(
    mode: libc::c_int,
    list: *const *const aiocb,
    nent: libc::c_int,
    sig: *mut sigevent,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn aio_error(aiocbp: *const aiocb) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn aio_return(aiocbp: *mut aiocb) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn aio_cancel(fildes: libc::c_int, aiocbp: *mut aiocb) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn aio_suspend(
    list: *const *const aiocb,
    nent: libc::c_int,
    timeout: *const timespec,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn aio_fsync(operation: libc::c_int, aiocbp: *mut aiocb) -> libc::c_int {
    unimplemented!();
}
