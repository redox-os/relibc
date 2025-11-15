use super::{Sys, e_raw};
use crate::{
    error::Result,
    header::{
        signal::sigevent,
        sys_socket::{msghdr, sockaddr, socklen_t},
        time::itimerspec,
    },
    out::Out,
    platform::{PalTimer, types::*},
};

impl PalTimer for Sys {
    type InternalTimer = timer_t;
    fn timer_create(clock_id: clockid_t, evp: &sigevent, mut timerid: Out<timer_t>) -> Result<()> {
        e_raw(unsafe {
            syscall!(
                TIMER_CREATE,
                clock_id,
                core::ptr::addr_of!(evp),
                timerid.as_mut_ptr()
            )
        })
        .map(|_| ())
    }

    fn timer_delete(timerid: timer_t) -> Result<()> {
        e_raw(unsafe { syscall!(TIMER_DELETE, timerid) }).map(|_| ())
    }

    fn timer_getoverrun(timerid: Self::InternalTimer) -> Result<c_int> {
        e_raw(unsafe { syscall!(TIMER_GETOVERRUN, timerid) }).map(|r| r as c_int)
    }

    fn timer_gettime(timerid: timer_t, mut value: Out<itimerspec>) -> Result<()> {
        e_raw(unsafe { syscall!(TIMER_GETTIME, timerid, value.as_mut_ptr()) }).map(|_| ())
    }

    fn timer_settime(
        timerid: timer_t,
        flags: c_int,
        value: &itimerspec,
        mut ovalue: Option<Out<itimerspec>>,
    ) -> Result<()> {
        e_raw(unsafe {
            syscall!(
                TIMER_SETTIME,
                timerid,
                flags,
                core::ptr::addr_of!(value),
                match ovalue {
                    None => core::ptr::null_mut(),
                    Some(mut o) => o.as_mut_ptr(),
                }
            )
        })
        .map(|_| ())
    }
}
