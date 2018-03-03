pub struct aiocb {
    pub aio_fildes: libc::c_int,
    pub aio_lio_opcode: libc::c_int,
    pub aio_reqprio: libc::c_int,
    pub aio_buf: *mut libc::c_void,
    pub aio_nbytes: usize,
    pub aio_sigevent: sigevent,
}

pub extern "C" fn aio_read(__aiocbp: *mut aiocb) -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn aio_write(__aiocbp: *mut aiocb) -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn lio_listio(__mode: libc::c_int,
                      __list: *const *const aiocb,
                      __nent: libc::c_int, __sig: *mut sigevent)
     -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn aio_error(__aiocbp: *const aiocb) -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn aio_return(__aiocbp: *mut aiocb) -> __ssize_t {
    unimplemented!();
}

pub extern "C" fn aio_cancel(__fildes: libc::c_int, __aiocbp: *mut aiocb)
     -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn aio_suspend(__list: *const *const aiocb,
                       __nent: libc::c_int,
                       __timeout: *const timespec) -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn aio_fsync(__operation: libc::c_int, __aiocbp: *mut aiocb)
     -> libc::c_int {
    unimplemented!();
}

