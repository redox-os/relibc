use crate::{
    error::Result,
    platform::{types::*, Pal},
};

pub trait PalPtrace: Pal {
    unsafe fn ptrace(
        request: c_int,
        pid: pid_t,
        addr: *mut c_void,
        data: *mut c_void,
    ) -> Result<c_int>;
}
