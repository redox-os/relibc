use redox_rt::proc::FdGuard;
use syscall::Error;

use crate::{
    error::{Errno, Result},
    header::{
        errno::{EINVAL, EIO, EOPNOTSUPP},
        fcntl::O_RDWR,
        pthread::{pthread_cancel, pthread_create},
        signal::{NSIG, SIGEV_NONE, SIGEV_SIGNAL, SIGEV_THREAD, SIGRTMIN, sigevent},
        sys_mman::{MAP_ANONYMOUS, PROT_READ, PROT_WRITE},
        time::{TIMER_ABSTIME, itimerspec, timespec},
    },
    out::Out,
    platform::{
        Pal, Sys,
        pal::PalTimer,
        sys::{event, libredox},
        types::{c_int, c_void, clockid_t},
    },
};
use core::{
    mem,
    mem::{MaybeUninit, size_of},
    ptr, slice,
};

/// timer_t internal data, ABI unstable
#[repr(C)]
#[derive(Clone)]
pub struct timer_internal_t {
    pub clockid: clockid_t,
    pub timerfd: usize,
    pub eventfd: usize,
    pub evp: sigevent,
    pub thread: *mut c_void,
    pub caller_thread: crate::pthread::OsTid,
    // relibc handles it_interval, not the kernel
    pub next_wake_time: itimerspec,
}

impl PalTimer for Sys {
    type InternalTimer = *mut timer_internal_t;
    fn timer_create(
        clock_id: clockid_t,
        evp: &sigevent,
        mut timerid: Out<Self::InternalTimer>,
    ) -> Result<()> {
        if evp.sigev_notify == SIGEV_THREAD {
            if evp.sigev_notify_function.is_none() {
                return Err(Errno(EINVAL));
            }
        } else if evp.sigev_notify == SIGEV_SIGNAL {
            const n_sig: i32 = NSIG as i32;
            const rt_min: i32 = SIGRTMIN as i32;
            const rt_max: i32 = SIGRTMIN as i32;
            match evp.sigev_signo {
                0..n_sig => {}
                rt_min..=rt_max => {}
                _ => {
                    return Err(Errno(EINVAL));
                }
            }
        }

        let path = format!("/scheme/time/{clock_id}");
        let timerfd = FdGuard::new(libredox::open(&path, O_RDWR, 0)?);
        let eventfd = FdGuard::new(Error::demux(unsafe {
            event::redox_event_queue_create_v1(0)
        })?);
        let caller_thread = Self::current_os_tid();

        let timer_buf = unsafe {
            let timer_buf = Self::mmap(
                ptr::null_mut(),
                size_of::<timer_internal_t>(),
                PROT_READ | PROT_WRITE,
                MAP_ANONYMOUS,
                0,
                0,
            )?;

            let timer_ptr = timer_buf as *mut timer_internal_t;
            let timer_st = &mut *timer_ptr;

            timer_st.clockid = clock_id;
            timer_st.timerfd = timerfd.take();
            timer_st.eventfd = eventfd.take();
            timer_st.evp = (*evp).clone();
            timer_st.next_wake_time = itimerspec::default();
            timer_st.thread = ptr::null_mut();
            timer_st.caller_thread = caller_thread;
            timer_buf
        };

        timerid.write(timer_buf.cast());

        Ok(())
    }

    fn timer_delete(timerid: Self::InternalTimer) -> Result<()> {
        unsafe {
            let timer_st = &mut *timerid;
            let _ = syscall::close(timer_st.timerfd);
            let _ = syscall::close(timer_st.eventfd);
            if !timer_st.thread.is_null() {
                let _ = pthread_cancel(timer_st.thread);
            }
            Self::munmap(timerid.cast(), size_of::<timer_internal_t>())?;
        }

        Ok(())
    }

    fn timer_gettime(timerid: Self::InternalTimer, mut value: Out<itimerspec>) -> Result<()> {
        let timer_st = unsafe { &mut *timerid };
        let mut now = timespec::default();
        Self::clock_gettime(timer_st.clockid, Out::from_mut(&mut now))?;

        if timer_st.evp.sigev_notify == SIGEV_NONE {
            if timespec::subtract(timer_st.next_wake_time.it_value, now).is_none() {
                // error here means the timer is disarmed
                let _ = timer_update_wake_time(timer_st);
            }
        }

        value.write(if timer_st.next_wake_time.it_value.is_default() {
            // disarmed
            itimerspec::default()
        } else {
            itimerspec {
                it_interval: timer_st.next_wake_time.it_interval,
                it_value: timespec::subtract(timer_st.next_wake_time.it_value, now)
                    .unwrap_or_default(),
            }
        });

        Ok(())
    }

    fn timer_settime(
        timerid: Self::InternalTimer,
        flags: c_int,
        value: &itimerspec,
        ovalue: Option<Out<itimerspec>>,
    ) -> Result<()> {
        let timer_st = unsafe { &mut *timerid };

        if let Some(ovalue) = ovalue {
            Self::timer_gettime(timerid, ovalue)?;
        }

        let mut now = timespec::default();
        Self::clock_gettime(timer_st.clockid, Out::from_mut(&mut now))?;

        //FIXME: make these atomic?
        timer_st.next_wake_time = {
            let mut val = value.clone();
            if flags & TIMER_ABSTIME == 0 {
                val.it_value = timespec::add(now, val.it_value).ok_or((Errno(EINVAL)))?;
            }
            val
        };

        Error::demux(unsafe {
            event::redox_event_queue_ctl_v1(timer_st.eventfd, timer_st.timerfd, 1, 0)
        })?;

        let buf_to_write = unsafe {
            slice::from_raw_parts(
                &timer_st.next_wake_time.it_value as *const _ as *const u8,
                mem::size_of::<timespec>(),
            )
        };

        let bytes_written = redox_rt::sys::posix_write(timer_st.timerfd, buf_to_write)?;

        if bytes_written < mem::size_of::<timespec>() {
            return Err(Errno(EIO));
        }

        if timer_st.thread.is_null() {
            timer_st.thread = match timer_st.evp.sigev_notify {
                SIGEV_THREAD | SIGEV_SIGNAL => {
                    let mut tid = ptr::null_mut();
                    let result = unsafe {
                        pthread_create(
                            &mut tid as *mut _,
                            ptr::null(),
                            timer_routine,
                            timerid as *mut c_void,
                        )
                    };
                    if result != 0 {
                        return Err(Errno(result));
                    }
                    tid
                }
                SIGEV_NONE => ptr::null_mut(),
                _ => {
                    return Err(Errno(EINVAL));
                }
            };
        }

        Ok(())
    }

    fn timer_getoverrun(_timerid: Self::InternalTimer) -> Result<c_int> {
        Err(Errno(EOPNOTSUPP))
    }
}

pub extern "C" fn timer_routine(arg: *mut c_void) -> *mut c_void {
    let timer_st = unsafe { &mut *(arg as *mut timer_internal_t) };

    loop {
        let mut buf = MaybeUninit::uninit();
        let res = Error::demux(unsafe {
            event::redox_event_queue_get_events_v1(
                timer_st.eventfd,
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

        if timer_st.evp.sigev_notify == SIGEV_THREAD {
            if let Some(fun) = timer_st.evp.sigev_notify_function {
                fun(timer_st.evp.sigev_value);
            }
        } else if timer_st.evp.sigev_notify == SIGEV_SIGNAL {
            if unsafe { Sys::rlct_kill(timer_st.caller_thread, timer_st.evp.sigev_signo as _) }
                .is_err()
            {
                break;
            }
        }

        if timer_next_event(timer_st).is_err() {
            break;
        }
    }

    timer_st.thread = ptr::null_mut();
    ptr::null_mut()
}

fn timer_next_event(timer_st: &mut timer_internal_t) -> Result<()> {
    timer_update_wake_time(timer_st)?;
    let buf_to_write = unsafe {
        Error::demux(event::redox_event_queue_ctl_v1(
            timer_st.eventfd,
            timer_st.timerfd,
            1,
            0,
        ))?;

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
