use super::super::{types::*, Pal};
use crate::errno::Errno;

pub trait PalPtrace: Pal {
    fn ptrace(
        request: c_int,
        pid: pid_t,
        addr: *mut c_void,
        data: *mut c_void,
    ) -> Result<c_int, Errno>;
}
