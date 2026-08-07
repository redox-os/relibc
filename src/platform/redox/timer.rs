use redox_rt::proc::FdGuard;
use syscall::Error;

use crate::{
    error::{Errno, Result},
    header::{
        errno::EIO,
        signal::{SIGEV_SIGNAL, SIGEV_THREAD, sigevent},
        time::{itimerspec, timespec},
    },
    out::Out,
    platform::{
        Pal, PalSignal, Sys,
        sys::event,
        types::{c_void, clockid_t, pid_t, pthread_t, timer_t},
    },
    sync::Mutex,
};
use alloc::collections::BTreeMap;
use core::{
    mem::{MaybeUninit, size_of},
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};

pub(crate) static TIMERS: Mutex<BTreeMap<usize, RlctTimer>> = Mutex::new(BTreeMap::new());

static TIMER_ID: AtomicUsize = AtomicUsize::new(1);

/// timer_t internal data, ABI unstable
pub(crate) struct RlctTimer {
    /// The key to this struct. This is NOT to be dereferenced
    pub key: timer_t,
    pub clockid: clockid_t,
    pub timerfd: FdGuard<true>,
    pub eventfd: FdGuard<true>,
    pub evp: sigevent,
    pub thread: pthread_t,
    /// relibc handles it_interval, not the kernel
    pub next_wake_time: itimerspec,
    /// kernel does not support unregistering timer
    pub next_wake_version: usize,
    // When non-zero, timer_routine delivers SIGALRM via kill(process_pid, sig)
    // instead of rlct_kill (thread-specific). Used by alarm().
    pub process_pid: pid_t,
}

unsafe impl Send for RlctTimer {}
unsafe impl Sync for RlctTimer {}

impl RlctTimer {
    /// Get the new instance of [`RlctTimer`], get the key
    pub fn create(
        clockid: i32,
        evp: &sigevent,
        timerfd: FdGuard<true>,
        eventfd: FdGuard<true>,
    ) -> timer_t {
        let id = TIMER_ID.fetch_add(1, Ordering::SeqCst);
        // TODO: how to avoid cast?
        let key: timer_t = id as _;
        let timer_st = RlctTimer {
            key,
            clockid,
            timerfd,
            eventfd,
            evp: evp.clone(),
            thread: ptr::null_mut(),
            next_wake_time: itimerspec::default(),
            next_wake_version: 0,
            process_pid: Sys::getpid(),
        };
        assert!(TIMERS.lock().insert(id, timer_st).is_none());
        key
    }
}

pub extern "C" fn timer_routine(arg: *mut c_void) -> *mut c_void {
    let (mut timer_version, eventfd) = {
        let timers = TIMERS.lock();
        let Some(timer_st) = timers.get(&arg.addr()) else {
            return ptr::null_mut();
        };
        (timer_st.next_wake_version, timer_st.eventfd.as_raw_fd())
    };
    loop {
        let mut buf = MaybeUninit::uninit();
        let res = Error::demux(unsafe {
            // this blocks the thread
            event::redox_event_queue_get_events_v1(
                // TODO: should safe to pass even closed, but not sure.
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

        let mut timers = TIMERS.lock();
        let Some(timer_st) = timers.get_mut(&arg.addr()) else {
            return ptr::null_mut();
        };
        if timer_version == timer_st.next_wake_version {
            if timer_st.evp.sigev_notify == SIGEV_THREAD {
                if let Some(fun) = timer_st.evp.sigev_notify_function {
                    fun(timer_st.evp.sigev_value);
                }
            } else if timer_st.evp.sigev_notify == SIGEV_SIGNAL
                && Sys::sigqueue(
                    timer_st.process_pid,
                    timer_st.evp.sigev_signo as _,
                    timer_st.evp.sigev_value,
                )
                .is_err()
            {
                break;
            }
        }

        if timer_next_event(timer_st).is_err() {
            break;
        }
        timer_version = timer_st.next_wake_version;
    }
    ptr::null_mut()
}

// Internal function only valid for inside timer_routine
fn timer_next_event(timer_st: &mut RlctTimer) -> Result<()> {
    if let Err(e) = timer_update_wake_time(timer_st) {
        timer_st.thread = ptr::null_mut();
        return Err(e);
    }
    let buf_to_write = unsafe {
        Error::demux(event::redox_event_queue_ctl_v1(
            timer_st.eventfd.as_raw_fd(),
            timer_st.timerfd.as_raw_fd(),
            1,
            0,
        ))?;

        syscall::TimeSpec::from(&timer_st.next_wake_time.it_value)
    };
    let bytes_written = timer_st.timerfd.write(&buf_to_write)?;
    if bytes_written < size_of::<timespec>() {
        return Err(Errno(EIO));
    }
    Ok(())
}

/// Update next_wake_time.it_value from next_wake_time.it_interval
pub(crate) fn timer_update_wake_time(timer_st: &mut RlctTimer) -> Result<()> {
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
