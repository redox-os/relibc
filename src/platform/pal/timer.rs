use crate::{
    error::Result,
    header::{
        signal::sigevent,
        sys_socket::{msghdr, sockaddr, socklen_t},
        time::itimerspec,
    },
    out::Out,
    platform::{Pal, types::*},
};

pub trait PalTimer: Pal {
    type InternalTimer;

    fn timer_create(
        clock_id: clockid_t,
        evp: &sigevent,
        timerid: Out<Self::InternalTimer>,
    ) -> Result<()>;

    fn timer_delete(timerid: Self::InternalTimer) -> Result<()>;

    fn timer_getoverrun(timerid: Self::InternalTimer) -> Result<c_int>;

    fn timer_gettime(timerid: Self::InternalTimer, value: Out<itimerspec>) -> Result<()>;

    fn timer_settime(
        timerid: Self::InternalTimer,
        flags: c_int,
        value: &itimerspec,
        ovalue: Option<Out<itimerspec>>,
    ) -> Result<()>;
}
