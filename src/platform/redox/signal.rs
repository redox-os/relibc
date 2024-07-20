use core::mem;
use redox_rt::signal::{Sigaction, SigactionFlags, SigactionKind, Sigaltstack, SignalHandler};
use syscall::{self, Result};

use super::{
    super::{types::*, Pal, PalSignal},
    e, Sys,
};
use crate::{
    header::{
        errno::{EINVAL, ENOSYS},
        signal::{
            sigaction, siginfo_t, sigset_t, stack_t, SA_SIGINFO, SIG_BLOCK, SIG_DFL, SIG_IGN,
            SIG_SETMASK, SIG_UNBLOCK, SS_DISABLE, SS_ONSTACK,
        },
        sys_time::{itimerval, ITIMER_REAL},
        time::timespec,
    },
    platform::ERRNO,
    pthread::Errno,
};

impl PalSignal for Sys {
    unsafe fn getitimer(which: c_int, out: *mut itimerval) -> c_int {
        let path = match which {
            ITIMER_REAL => "itimer:1",
            _ => {
                ERRNO.set(EINVAL);
                return -1;
            }
        };

        let fd = e(syscall::open(path, syscall::O_RDONLY | syscall::O_CLOEXEC));
        if fd == !0 {
            return -1;
        }

        let mut spec = syscall::ITimerSpec::default();
        let count = e(syscall::read(fd, &mut spec));

        let _ = syscall::close(fd);

        if count == !0 {
            return -1;
        }

        (*out).it_interval.tv_sec = spec.it_interval.tv_sec as time_t;
        (*out).it_interval.tv_usec = spec.it_interval.tv_nsec / 1000;
        (*out).it_value.tv_sec = spec.it_value.tv_sec as time_t;
        (*out).it_value.tv_usec = spec.it_value.tv_nsec / 1000;

        0
    }

    fn kill(pid: pid_t, sig: c_int) -> c_int {
        e(redox_rt::sys::posix_kill(pid as usize, sig as usize).map(|()| 0)) as c_int
    }

    fn killpg(pgrp: pid_t, sig: c_int) -> c_int {
        e(redox_rt::sys::posix_killpg(pgrp as usize, sig as usize).map(|()| 0)) as c_int
    }

    fn raise(sig: c_int) -> Result<(), Errno> {
        // TODO: Bypass kernel?
        unsafe { Self::rlct_kill(Self::current_os_tid(), sig as _) }
    }

    unsafe fn setitimer(which: c_int, new: *const itimerval, old: *mut itimerval) -> c_int {
        let path = match which {
            ITIMER_REAL => "itimer:1",
            _ => {
                ERRNO.set(EINVAL);
                return -1;
            }
        };

        let fd = e(syscall::open(path, syscall::O_RDWR | syscall::O_CLOEXEC));
        if fd == !0 {
            return -1;
        }

        let mut spec = syscall::ITimerSpec::default();

        let mut count = e(syscall::read(fd, &mut spec));

        if count != !0 {
            unsafe {
                if !old.is_null() {
                    (*old).it_interval.tv_sec = spec.it_interval.tv_sec as time_t;
                    (*old).it_interval.tv_usec = spec.it_interval.tv_nsec / 1000;
                    (*old).it_value.tv_sec = spec.it_value.tv_sec as time_t;
                    (*old).it_value.tv_usec = spec.it_value.tv_nsec / 1000;
                }

                spec.it_interval.tv_sec = (*new).it_interval.tv_sec as i64;
                spec.it_interval.tv_nsec = (*new).it_interval.tv_usec * 1000;
                spec.it_value.tv_sec = (*new).it_value.tv_sec as i64;
                spec.it_value.tv_nsec = (*new).it_value.tv_usec * 1000;
            }

            count = e(syscall::write(fd, &spec));
        }

        let _ = syscall::close(fd);

        if count == !0 {
            return -1;
        }

        0
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
                    handler: if c_act.sa_flags & crate::header::signal::SA_SIGINFO as c_ulong != 0 {
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
                    sa_flags: old_action.flags.bits().into(),
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
        redox_rt::signal::sigaltstack(new.as_ref(), old.as_mut())?;

        if let (Some(old_c_stack), Some(old)) = (old_c, old) {
            *old_c_stack = match old {
                Sigaltstack::Disabled => stack_t {
                    ss_sp: core::ptr::null_mut(),
                    ss_size: 0,
                    ss_flags: SS_DISABLE.try_into().unwrap(),
                },
                Sigaltstack::Enabled {
                    onstack,
                    base,
                    size,
                } => stack_t {
                    ss_sp: base.cast(),
                    ss_size: size,
                    ss_flags: SS_ONSTACK.try_into().unwrap(),
                },
            };
        }
        Ok(())
    }

    fn sigpending(set: &mut sigset_t) -> Result<(), Errno> {
        *set = redox_rt::signal::currently_pending();
        Ok(())
    }

    fn sigprocmask(
        how: c_int,
        set: Option<&sigset_t>,
        oset: Option<&mut sigset_t>,
    ) -> Result<(), Errno> {
        Ok(match how {
            SIG_SETMASK => redox_rt::signal::set_sigmask(set.copied(), oset)?,
            SIG_BLOCK => redox_rt::signal::or_sigmask(set.copied(), oset)?,
            SIG_UNBLOCK => redox_rt::signal::andn_sigmask(set.copied(), oset)?,

            _ => return Err(Errno(EINVAL)),
        })
    }

    unsafe fn sigsuspend(set: *const sigset_t) -> c_int {
        ERRNO.set(ENOSYS);
        -1
    }

    unsafe fn sigtimedwait(
        set: *const sigset_t,
        sig: *mut siginfo_t,
        tp: *const timespec,
    ) -> c_int {
        ERRNO.set(ENOSYS);
        -1
    }
}
