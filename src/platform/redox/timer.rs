use ::event::raw::RawEventV1;
use syscall::Error;

use crate::{
    error::{Errno, Result},
    header::{
        errno::EIO,
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
    mem::{MaybeUninit, size_of, transmute},
    ops::ControlFlow,
    ptr, slice,
};

pub extern "C" fn timer_routine(arg: *mut c_void) -> *mut c_void {
    unsafe {
        let timer_st = &mut *(arg as *mut timer_internal_t);

        loop {
            let mut buf = MaybeUninit::uninit();

            unsafe {
                let res = Error::demux(event::redox_event_queue_get_events_v1(
                    timer_st.eventfd,
                    buf.as_mut_ptr(),
                    1,
                    0,
                    core::ptr::null(),
                    core::ptr::null(),
                ));
                if let Ok(res) = res {
                    assert_eq!(res, 1, "EOF is not yet well defined for event queues");
                } else {
                    timer_st.thread = ptr::null_mut();
                    break;
                }

                if let Some(fun) = timer_st.evp.sigev_notify_function {
                    fun(timer_st.evp.sigev_value);
                }

                if timer_next_event(timer_st).is_err() {
                    timer_st.thread = ptr::null_mut();
                    break;
                }
            }
        }
    }

    ptr::null_mut()
}

fn timer_next_event(timer_st: &mut timer_internal_t) -> Result<()> {
    timer_update_wake_time(timer_st)?;

    Error::demux(unsafe {
        event::redox_event_queue_ctl_v1(timer_st.eventfd, timer_st.timerfd, 1, 0)
    })?;

    let buf_to_write = unsafe {
        slice::from_raw_parts(
            &timer_st.next_wake_time.it_value as *const _ as *const u8,
            size_of::<timespec>(),
        )
    };

    let bytes_written = redox_rt::sys::posix_write(timer_st.timerfd, buf_to_write)?;
    if bytes_written < size_of::<timespec>() {
        return Err(Errno(EIO));
    }
    Ok(())
}

pub(crate) fn timer_update_wake_time(timer_st: &mut timer_internal_t) -> Result<()> {
    timer_st.next_wake_time.it_value = if timer_st.next_wake_time.it_interval.is_default() {
        timespec::default()
    } else {
        let mut now = timespec::default();
        Sys::clock_gettime(timer_st.clockid, Out::from_mut(&mut now))?;
        let next_time = match timespec::add(now, timer_st.next_wake_time.it_interval) {
            Some(a) => a,
            None => timespec::default(),
        };

        next_time
    };
    if timer_st.next_wake_time.it_value.is_default() {
        return Err(Errno(0));
    }
    Ok(())
}
