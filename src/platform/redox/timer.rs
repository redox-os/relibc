use syscall::Error;

use crate::{
    error::{Errno, Result},
    header::{
        errno::EIO,
        signal::{SIGEV_SIGNAL, SIGEV_THREAD},
        time::{timer_internal_t, timespec},
    },
    out::Out,
    platform::{Pal, Sys, sys::event, types::c_void},
};
use core::{
    mem::{MaybeUninit, size_of},
    ops::DerefMut,
    ptr,
};

pub extern "C" fn timer_routine(arg: *mut c_void) -> *mut c_void {
    let timer_ptr = unsafe { timer_internal_t::from_raw(arg) };
    let (mut timer_version, eventfd) = {
        let timer_st = timer_ptr.lock();
        (timer_st.next_wake_version, timer_st.eventfd)
    };
    loop {
        let mut buf = MaybeUninit::uninit();
        let res = Error::demux(unsafe {
            // this blocks the thread
            event::redox_event_queue_get_events_v1(
                eventfd,
                buf.as_mut_ptr(),
                1,
                0,
                core::ptr::null(),
                core::ptr::null(),
            )
        });
        if let Ok(res) = res {
            assert_eq!(res, 1, "EOF is not yet well defined for event queues");
        } else {
            break;
        }

        let mut timer_st = timer_ptr.lock();
        if timer_version == timer_st.next_wake_version {
            if timer_st.evp.sigev_notify == SIGEV_THREAD {
                if let Some(fun) = timer_st.evp.sigev_notify_function {
                    fun(timer_st.evp.sigev_value);
                }
            } else if timer_st.evp.sigev_notify == SIGEV_SIGNAL {
                // TODO: This will deliver signal to process, which is required for alarm()
                //       Until it can bypass the exec() boundary, do not uncomment this code
                // if timer_st.process_pid != 0 && Sys::kill(timer_st.process_pid, timer_st.evp.sigev_signo).is_err() { break; } else
                if unsafe { Sys::rlct_kill(timer_st.caller_thread, timer_st.evp.sigev_signo as _) }
                    .is_err()
                {
                    break;
                }
            }
        }

        if timer_next_event(timer_st.deref_mut()).is_err() {
            break;
        }
        timer_version = timer_st.next_wake_version;
    }
    ptr::null_mut()
}

// Internal function only valid for inside timer_routine
fn timer_next_event(timer_st: &mut timer_internal_t) -> Result<()> {
    if let Err(e) = timer_update_wake_time(timer_st) {
        timer_st.thread = ptr::null_mut();
        return Err(e);
    }
    let buf_to_write = unsafe {
        Error::demux(event::redox_event_queue_ctl_v1(
            timer_st.eventfd,
            timer_st.timerfd,
            1,
            0,
        ))?;

        syscall::TimeSpec::from(&timer_st.next_wake_time.it_value)
    };
    let bytes_written = redox_rt::sys::posix_write(timer_st.timerfd, &*buf_to_write)?;
    if bytes_written < size_of::<timespec>() {
        return Err(Errno(EIO));
    }
    Ok(())
}

/// Update next_wake_time.it_value from next_wake_time.it_interval
pub(crate) fn timer_update_wake_time(timer_st: &mut timer_internal_t) -> Result<()> {
    let interval = &timer_st.next_wake_time.it_interval;
    timer_st.next_wake_time.it_value = if interval.is_zero() {
        timespec::default()
    } else {
        let mut now = timespec::default();
        Sys::clock_gettime(timer_st.clockid, Out::from_mut(&mut now))?;
        timespec::add(&now, interval).unwrap_or_default()
    };
    if timer_st.next_wake_time.it_value.is_zero() {
        return Err(Errno(0));
    }
    timer_st.next_wake_version += 1;
    Ok(())
}
