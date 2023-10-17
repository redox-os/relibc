use core::mem;
use syscall::{self, Result};

use super::{
    super::{types::*, Pal, PalSignal},
    e_raw, Sys,
};
use crate::{
    errno::Errno,
    header::{
        errno::{EINVAL, ENOSYS},
        signal::{sigaction, siginfo_t, sigset_t, stack_t},
        sys_time::{itimerval, ITIMER_REAL},
        time::timespec,
    },
    platform::errno,
};

impl PalSignal for Sys {
    fn getitimer(which: c_int, out: *mut itimerval) -> Result<(), Errno> {
        let path = match which {
            ITIMER_REAL => "itimer:1",
            _ => return Err(Errno(EINVAL)),
        };

        let fd = e_raw(syscall::open(path, syscall::O_RDONLY | syscall::O_CLOEXEC))?;

        let mut spec = syscall::ITimerSpec::default();
        let count = e_raw(syscall::read(fd, &mut spec));

        let _ = syscall::close(fd);

        count?;

        unsafe {
            (*out).it_interval.tv_sec = spec.it_interval.tv_sec as time_t;
            (*out).it_interval.tv_usec = spec.it_interval.tv_nsec / 1000;
            (*out).it_value.tv_sec = spec.it_value.tv_sec as time_t;
            (*out).it_value.tv_usec = spec.it_value.tv_nsec / 1000;
        }

        Ok(())
    }

    fn kill(pid: pid_t, sig: c_int) -> Result<(), Errno> {
        e_raw(syscall::kill(pid as usize, sig as usize))?;
        Ok(())
    }

    fn killpg(pgrp: pid_t, sig: c_int) -> Result<(), Errno> {
        e_raw(syscall::kill(-(pgrp as isize) as usize, sig as usize))?;
        Ok(())
    }

    fn raise(sig: c_int) -> Result<(), Errno> {
        Self::kill(Self::getpid(), sig)
    }

    fn setitimer(which: c_int, new: *const itimerval, old: *mut itimerval) -> Result<(), Errno> {
        let path = match which {
            ITIMER_REAL => "itimer:1",
            _ => return Err(Errno(EINVAL)),
        };

        let fd = e_raw(syscall::open(path, syscall::O_RDWR | syscall::O_CLOEXEC))?;

        let mut spec = syscall::ITimerSpec::default();

        let mut count = e_raw(syscall::read(fd, &mut spec));

        if let Ok(_) = count {
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

            count = e_raw(syscall::write(fd, &spec));
        }

        let _ = syscall::close(fd);

        count?;

        Ok(())
    }

    fn sigaction(
        sig: c_int,
        act: Option<&sigaction>,
        oact: Option<&mut sigaction>,
    ) -> Result<(), Errno> {
        e_raw(sigaction_impl(sig, act, oact).map(|()| 0))?;
        Ok(())
    }

    fn sigaltstack(ss: *const stack_t, old_ss: *mut stack_t) -> Result<(), Errno> {
        unimplemented!()
    }

    fn sigpending(set: *mut sigset_t) -> Result<(), Errno> {
        Err(Errno(ENOSYS))
    }

    fn sigprocmask(how: c_int, set: *const sigset_t, oset: *mut sigset_t) -> Result<(), Errno> {
        e_raw(sigprocmask_impl(how, set, oset).map(|()| 0))?;
        Ok(())
    }

    fn sigsuspend(set: *const sigset_t) -> Errno {
        Errno(ENOSYS)
    }

    fn sigtimedwait(
        set: *const sigset_t,
        sig: *mut siginfo_t,
        tp: *const timespec,
    ) -> Result<c_int, Errno> {
        Err(Errno(ENOSYS))
    }
}

pub(crate) fn sigaction_impl(
    sig: i32,
    act: Option<&sigaction>,
    oact: Option<&mut sigaction>,
) -> Result<()> {
    let new_opt = act.map(|act| {
        let m = act.sa_mask;
        let sa_handler = unsafe { mem::transmute(act.sa_handler) };
        syscall::SigAction {
            sa_handler,
            sa_mask: [m as u64, 0],
            sa_flags: syscall::SigActionFlags::from_bits(act.sa_flags as usize)
                .expect("sigaction: invalid bit pattern"),
        }
    });
    let mut old_opt = oact.as_ref().map(|_| syscall::SigAction::default());
    syscall::sigaction(sig as usize, new_opt.as_ref(), old_opt.as_mut())?;
    if let (Some(old), Some(oact)) = (old_opt, oact) {
        oact.sa_handler = unsafe { mem::transmute(old.sa_handler) };
        let m = old.sa_mask;
        oact.sa_mask = m[0] as sigset_t;
        oact.sa_flags = old.sa_flags.bits() as c_ulong;
    }
    Ok(())
}
pub(crate) fn sigprocmask_impl(how: i32, set: *const sigset_t, oset: *mut sigset_t) -> Result<()> {
    let new_opt = if set.is_null() {
        None
    } else {
        Some([unsafe { *set as u64 }, 0])
    };
    let mut old_opt = if oset.is_null() { None } else { Some([0, 0]) };
    syscall::sigprocmask(how as usize, new_opt.as_ref(), old_opt.as_mut())?;
    if let Some(old) = old_opt {
        unsafe { *oset = old[0] as sigset_t };
    }
    Ok(())
}
