use crate::{
    error::Result,
    header::signal::sigset_t,
    platform::{
        Pal,
        types::{c_int, c_uint, c_void, size_t, uint64_t},
    },
};

pub trait PalIOUring: Pal {
    unsafe fn io_uring_enter(
        fd: c_uint,
        to_submit: c_uint,
        min_complete: c_uint,
        flags: c_uint,
        sig: Option<&sigset_t>,
    ) -> Result<c_int>;
    unsafe fn io_uring_enter2(
        fd: c_uint,
        to_submit: c_uint,
        min_complete: c_uint,
        flags: c_uint,
        arg: *mut c_void,
        sz: size_t,
    ) -> Result<c_int>;
    unsafe fn io_uring_setup(entries: c_uint, p: &mut io_uring_params) -> Result<c_int>;
    unsafe fn io_uring_register(
        fd: c_uint,
        opcode: c_uint,
        arg: *mut c_void,
        nr_args: c_uint,
    ) -> Result<c_int>;
}
