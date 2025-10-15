use ::event::raw::RawEventV1;
use syscall::Error;

use crate::{
    error::{Errno, Result},
    header::{
        signal::{sigevent, sigval},
        time::{timer_internal_t, timespec},
    },
    out::Out,
    platform::{
        Pal, Sys,
        sys::{event, libredox},
        types::c_void,
    },
};
use core::{
    mem::{MaybeUninit, transmute},
    ops::ControlFlow,
    ptr,
};

pub extern "C" fn timer_routine(arg: *mut c_void) -> *mut c_void {
    unsafe {
        let timer = &mut *(arg as *mut timer_internal_t);

        loop {
            let mut buf = MaybeUninit::uninit();

            unsafe {
                let res = Error::demux(event::redox_event_queue_get_events_v1(
                    timer.eventfd,
                    buf.as_mut_ptr(),
                    1,
                    0,
                    core::ptr::null(),
                    core::ptr::null(),
                ));
                if let Ok(res) = res {
                    assert_eq!(res, 1, "EOF is not yet well defined for event queues");
                } else {
                    break;
                }

                if let Some(fun) = timer.evp.sigev_notify_function {
                    fun(timer.evp.sigev_value);
                }

                if timer_next_event(timer).is_err() {
                    break;
                }
            }
        }
    }

    ptr::null_mut()
}

fn timer_next_event(timer: &mut timer_internal_t) -> Result<()> {
    if timer.next_wake_time.it_interval.tv_nsec == 0 && timer.next_wake_time.it_interval.tv_sec == 0
    {
        timer.next_wake_time.it_value = timespec::default();
        return Err(Errno(0));
    } else {
        let mut now = timespec::default();
        Sys::clock_gettime(timer.clockid, Out::from_mut(&mut now))?;
        let next_time = match timespec::add(now, timer.next_wake_time.it_interval) {
            Some(a) => a,
            None => return Err(Errno(0)),
        };

        timer.next_wake_time.it_value = next_time;

        Error::demux(unsafe {
            event::redox_event_queue_ctl_v1(timer.eventfd, timer.timerfd, 1, 0)
        })?;
    }
    Ok(())
}
