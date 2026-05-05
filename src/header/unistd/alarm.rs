#[cfg(target_os = "redox")]
use crate::header::time::timer_internal_t;
use crate::{
    header::{
        bits_timespec::timespec,
        signal::{SIGALRM, SIGEV_SIGNAL, sigevent},
        time::itimerspec,
    },
    out::Out,
    platform::{
        Pal, Sys,
        types::{c_int, c_uint, timer_t},
    },
    sync::Mutex,
};

/// Wrapper for timer_t that implements Send (the timer_t pointer is a process-
/// wide mmap'd allocation that outlives any single thread).
struct AlarmTimer(timer_t);
// SAFETY: The timer_t pointer refers to an mmap'd timer_internal_t that is
// only accessed under the ALARM_TIMER mutex lock.
unsafe impl Send for AlarmTimer {}

/// Process-global singleton timer used by alarm(). Protected by a mutex to
/// ensure only one alarm is active at a time (POSIX requirement).
static ALARM_TIMER: Mutex<Option<AlarmTimer>> = Mutex::new(None);

/// Internal helper that arms/disarms the process-global alarm timer.
/// Accepts a full timespec so sub-second timers (ualarm) can reuse this later.
/// Returns the number of seconds remaining on the previous alarm (rounded up),
/// or 0 if there was no previous alarm.
///
/// TODO: This implementation does not survive `exec()`. POSIX requires that a
/// pending alarm be preserved across exec (the timer continues counting down
/// in the new process image as i understand).
pub fn alarm_timespec(duration: timespec) -> c_uint {
    let mut guard = ALARM_TIMER.lock();

    // Determine remaining time on any existing alarm
    let remaining = if let Some(ref alarm) = *guard {
        let mut cur = itimerspec::default();
        if Sys::timer_gettime(alarm.0, Out::from_mut(&mut cur)).is_ok() {
            let secs = cur.it_value.tv_sec as c_uint;
            if cur.it_value.tv_nsec > 0 {
                secs + 1 // POSIX: round up
            } else {
                secs
            }
        } else {
            0
        }
    } else {
        0
    };

    let disarm = duration.tv_sec == 0 && duration.tv_nsec == 0;

    if disarm {
        // alarm(0): cancel any pending alarm
        if let Some(ref alarm) = *guard {
            let zero = itimerspec::default();
            let _ = Sys::timer_settime(alarm.0, 0, &zero, None);
        }
        return remaining;
    }

    // Lazily create the singleton timer if it doesn't exist yet
    if guard.is_none() {
        let mut evp = unsafe { core::mem::zeroed::<sigevent>() };
        evp.sigev_notify = SIGEV_SIGNAL;
        evp.sigev_signo = SIGALRM as c_int;
        let mut timer_id: timer_t = core::ptr::null_mut();
        if Sys::timer_create(
            crate::header::time::CLOCK_REALTIME,
            &evp,
            Out::from_mut(&mut timer_id),
        )
        .is_err()
        {
            return remaining;
        }

        // Enable process-wide signal delivery instead of thread-specific
        #[cfg(target_os = "redox")]
        {
            let timer_ptr = unsafe { timer_internal_t::from_raw(timer_id) };
            let mut timer_st = timer_ptr.lock();
            timer_st.process_pid = Sys::getpid();
            drop(timer_st);
        }

        *guard = Some(AlarmTimer(timer_id));
    }

    let timer_id = guard
        .as_ref()
        .expect("alarm timer must exist after lazy init")
        .0;

    // Arm the timer as a one-shot (no interval)
    let spec = itimerspec {
        it_value: duration,
        it_interval: timespec::default(),
    };
    let _ = Sys::timer_settime(timer_id, 0, &spec, None);

    remaining
}
