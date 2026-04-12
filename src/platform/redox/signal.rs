use super::{
    super::{Pal, PalSignal, types::*},
    Sys,
};
use crate::{
    error::{Errno, Result},
    header::{
        bits_timespec::timespec,
        errno::{EINVAL, ENOSYS},
        signal::{
            SIG_BLOCK, SIG_DFL, SIG_IGN, SIG_SETMASK, SIG_UNBLOCK, SIGALRM, SIGEV_SIGNAL,
            SS_DISABLE, SS_ONSTACK, sigaction, sigevent, siginfo_t, sigset_t, sigval, stack_t,
            ucontext_t,
        },
        sys_time::{ITIMER_REAL, itimerval},
        time::{itimerspec, timer_internal_t},
    },
    out::Out,
    sync::Mutex,
};
use core::mem::offset_of;
use redox_protocols::protocol::ProcKillTarget;
use redox_rt::signal::{
    PosixStackt, SigStack, Sigaction, SigactionFlags, SigactionKind, Sigaltstack, SignalHandler,
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

const _: () = {
    #[track_caller]
    const fn assert_eq(a: usize, b: usize) {
        if a != b {
            panic!("compile-time struct verification failed");
        }
    }
    assert_eq(offset_of!(ucontext_t, uc_link), offset_of!(SigStack, link));
    assert_eq(
        offset_of!(ucontext_t, uc_stack),
        offset_of!(SigStack, old_stack),
    );
    assert_eq(
        offset_of!(ucontext_t, uc_sigmask),
        offset_of!(SigStack, old_mask),
    );
    assert_eq(
        offset_of!(ucontext_t, uc_mcontext),
        offset_of!(SigStack, regs),
    );
};

impl PalSignal for Sys {
    fn getitimer(which: c_int, out: &mut itimerval) -> Result<()> {
        let path = match which {
            ITIMER_REAL => "/scheme/itimer/1",
            _ => return Err(Errno(EINVAL)),
        };
        // TODO: implement setitimer
        // let fd = FdGuard::new(redox_rt::sys::open(path, syscall::O_RDONLY | syscall::O_CLOEXEC)?);
        // let count = syscall::read(*fd, &mut spec)?;

        let spec = syscall::ITimerSpec::default();
        out.it_interval.tv_sec = spec.it_interval.tv_sec as time_t;
        out.it_interval.tv_usec = spec.it_interval.tv_nsec / 1000;
        out.it_value.tv_sec = spec.it_value.tv_sec as time_t;
        out.it_value.tv_usec = spec.it_value.tv_nsec / 1000;

        Ok(())
    }

    fn kill(pid: pid_t, sig: c_int) -> Result<()> {
        redox_rt::sys::posix_kill(ProcKillTarget::from_raw(pid as usize), sig as usize)?;
        Ok(())
    }
    fn sigqueue(pid: pid_t, sig: c_int, val: sigval) -> Result<()> {
        Ok(redox_rt::sys::posix_sigqueue(
            pid as usize,
            sig as usize,
            unsafe { val.sival_ptr } as usize,
        )?)
    }

    fn killpg(pgrp: pid_t, sig: c_int) -> Result<()> {
        if pgrp == 1 {
            return Err(Errno(EINVAL));
        }
        Self::kill(-pgrp, sig)
    }

    fn raise(sig: c_int) -> Result<()> {
        // TODO: Bypass kernel?
        unsafe { Self::rlct_kill(Self::current_os_tid(), sig as _) }
    }

    fn setitimer(which: c_int, _new: &itimerval, old: Option<&mut itimerval>) -> Result<()> {
        // TODO: setitimer is no longer part of POSIX and should not be implemented in Redox
        // Change the platform-independent implementation to use POSIX timers.
        // For Redox, the timer should probably use "/scheme/time"
        todo_skip!(0, "setitimer not implemented");
        Err(Errno(ENOSYS))
    }

    fn sigaction(
        sig: c_int,
        c_act: Option<&sigaction>,
        c_oact: Option<&mut sigaction>,
    ) -> Result<(), Errno> {
        let sig = u8::try_from(sig).map_err(|_| syscall::Error::new(syscall::EINVAL))?;

        let new_action = c_act.map(|c_act| {
            let handler = c_act.sa_handler.map_or(0, |f| f as usize);

            let kind = if handler == SIG_DFL {
                SigactionKind::Default
            } else if handler == SIG_IGN {
                SigactionKind::Ignore
            } else {
                SigactionKind::Handled {
                    handler: if c_act.sa_flags & crate::header::signal::SA_SIGINFO as c_int != 0 {
                        SignalHandler {
                            sigaction: unsafe { core::mem::transmute(c_act.sa_handler) },
                        }
                    } else {
                        SignalHandler {
                            handler: c_act.sa_handler,
                        }
                    },
                }
            };

            Sigaction {
                kind,
                mask: c_act.sa_mask,
                flags: SigactionFlags::from_bits_retain(c_act.sa_flags as u32),
            }
        });
        let mut old_action = c_oact.as_ref().map(|_| Sigaction::default());

        redox_rt::signal::sigaction(sig, new_action.as_ref(), old_action.as_mut())?;

        if let (Some(c_oact), Some(old_action)) = (c_oact, old_action) {
            *c_oact = match old_action.kind {
                SigactionKind::Ignore => sigaction {
                    sa_handler: unsafe { core::mem::transmute(SIG_IGN) },
                    sa_flags: 0,
                    sa_restorer: None,
                    sa_mask: 0,
                },
                SigactionKind::Default => sigaction {
                    sa_handler: unsafe { core::mem::transmute(SIG_DFL) },
                    sa_flags: 0,
                    sa_restorer: None,
                    sa_mask: 0,
                },
                SigactionKind::Handled { handler } => sigaction {
                    sa_handler: if old_action.flags.contains(SigactionFlags::SIGINFO) {
                        unsafe { core::mem::transmute(handler.sigaction) }
                    } else {
                        unsafe { handler.handler }
                    },
                    sa_restorer: None,
                    sa_flags: old_action.flags.bits() as c_int,
                    sa_mask: old_action.mask,
                },
            };
        }
        Ok(())
    }

    unsafe fn sigaltstack(
        new_c: Option<&stack_t>,
        old_c: Option<&mut stack_t>,
    ) -> Result<(), Errno> {
        let new = new_c
            .map(|c_stack| {
                let flags = usize::try_from(c_stack.ss_flags).map_err(|_| Errno(EINVAL))?;
                if flags != flags & (SS_DISABLE | SS_ONSTACK) {
                    return Err(Errno(EINVAL));
                }

                Ok(if flags & SS_DISABLE == SS_DISABLE {
                    Sigaltstack::Disabled
                } else {
                    Sigaltstack::Enabled {
                        onstack: false,
                        base: c_stack.ss_sp.cast(),
                        size: c_stack.ss_size,
                    }
                })
            })
            .transpose()?;

        let mut old = old_c.as_ref().map(|_| Sigaltstack::default());
        (unsafe { redox_rt::signal::sigaltstack(new.as_ref(), old.as_mut()) })?;

        if let (Some(old_c_stack), Some(old)) = (old_c, old) {
            let c_stack = PosixStackt::from(old);
            *old_c_stack = stack_t {
                ss_sp: c_stack.sp.cast(),
                ss_size: c_stack.size,
                ss_flags: c_stack.flags,
            };
        }
        Ok(())
    }

    fn sigpending(set: &mut sigset_t) -> Result<(), Errno> {
        *set = redox_rt::signal::currently_pending_blocked();
        Ok(())
    }

    fn sigprocmask(
        how: c_int,
        set: Option<&sigset_t>,
        oset: Option<&mut sigset_t>,
    ) -> Result<(), Errno> {
        match how {
            _ if set.is_none() => {
                if let Some(oset) = oset {
                    *oset = redox_rt::signal::get_sigmask()?;
                }
            }
            SIG_SETMASK => redox_rt::signal::set_sigmask(set.copied(), oset)?,
            SIG_BLOCK => redox_rt::signal::or_sigmask(set.copied(), oset)?,
            SIG_UNBLOCK => redox_rt::signal::andn_sigmask(set.copied(), oset)?,

            _ => return Err(Errno(EINVAL)),
        }
        Ok(())
    }

    fn sigsuspend(mask: &sigset_t) -> Errno {
        match redox_rt::signal::await_signal_async(!*mask) {
            Ok(_) => unreachable!(),
            Err(err) => err.into(),
        }
    }

    fn sigtimedwait(
        set: &sigset_t,
        info_out: Option<&mut siginfo_t>,
        timeout: Option<&timespec>,
    ) -> Result<c_int, Errno> {
        // TODO: deadline-based API
        let timeout = timeout.map(|timeout| syscall::TimeSpec {
            tv_sec: timeout.tv_sec,
            tv_nsec: timeout.tv_nsec as _,
        });
        let info = redox_rt::signal::await_signal_sync(*set, timeout.as_ref())?.into();
        if let Some(out) = info_out {
            *out = info;
        }
        Ok(info.si_signo)
    }

    /// Thread-based alarm() that Recycles the existing POSIX timer machinery
    /// (timerfd + eventfd + pthread) with process-level signal delivery.
    ///
    /// Internally works with a timespec to allow sub-second timers in the
    /// future (e.g. ualarm) as i've been asked, though the public API only exposes whole seconds.
    fn alarm(seconds: c_uint) -> c_uint {
        alarm_timespec(timespec {
            tv_sec: seconds as time_t,
            tv_nsec: 0,
        })
    }
}

/// Internal helper that arms/disarms the process-global alarm timer.
/// Accepts a full timespec so sub-second timers (ualarm) can reuse this later.
/// Returns the number of seconds remaining on the previous alarm (rounded up),
/// or 0 if there was no previous alarm.
///
/// TODO: This implementation does not survive `exec()`. POSIX requires that a
/// pending alarm be preserved across exec (the timer continues counting down
/// in the new process image as i understand).
fn alarm_timespec(duration: timespec) -> c_uint {
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
        let evp = sigevent {
            sigev_value: sigval {
                sival_ptr: core::ptr::null_mut(),
            },
            sigev_signo: SIGALRM as c_int,
            sigev_notify: SIGEV_SIGNAL,
            sigev_notify_function: None,
            sigev_notify_attributes: core::ptr::null_mut(),
        };

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
        let timer_st = unsafe { &mut *(timer_id as *mut timer_internal_t) };
        timer_st.process_pid = Sys::getpid();

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
