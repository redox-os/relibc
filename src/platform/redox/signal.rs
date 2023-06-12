use core::{mem, cell::Cell, sync::atomic::{AtomicUsize, Ordering}};
use libc::{SIG_DFL, SIG_IGN};
use redox_exec::FdGuard;
use syscall::{self, SignalStack, SigAction, SigActionFlags, O_CLOEXEC};

use super::{
    super::{types::*, Pal, PalSignal},
    e, Sys, path::SignalMask,
};
use crate::{
    header::{
        errno::{EINVAL, ENOSYS},
        signal::{sigaction, siginfo_t, sigset_t, stack_t, sival},
        sys_time::{itimerval, ITIMER_REAL},
        time::timespec,
    },
    platform::errno, sync::Mutex,
};

impl PalSignal for Sys {
    fn getitimer(which: c_int, out: *mut itimerval) -> c_int {
        let path = match which {
            ITIMER_REAL => "itimer:1",
            _ => unsafe {
                errno = EINVAL;
                return -1;
            },
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

        unsafe {
            (*out).it_interval.tv_sec = spec.it_interval.tv_sec as time_t;
            (*out).it_interval.tv_usec = spec.it_interval.tv_nsec / 1000;
            (*out).it_value.tv_sec = spec.it_value.tv_sec as time_t;
            (*out).it_value.tv_usec = spec.it_value.tv_nsec / 1000;
        }

        0
    }

    fn kill(pid: pid_t, sig: c_int) -> c_int {
        e(syscall::kill(pid as usize, sig as usize)) as c_int
    }

    fn killpg(pgrp: pid_t, sig: c_int) -> c_int {
        e(syscall::kill(-(pgrp as isize) as usize, sig as usize)) as c_int
    }

    fn raise(sig: c_int) -> c_int {
        Self::kill(Self::getpid(), sig)
    }

    fn setitimer(which: c_int, new: *const itimerval, old: *mut itimerval) -> c_int {
        let path = match which {
            ITIMER_REAL => "itimer:1",
            _ => unsafe {
                errno = EINVAL;
                return -1;
            },
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

    fn sigaction(sig: c_int, act: Option<&sigaction>, oact: Option<&mut sigaction>) -> c_int {
        let mut is_sa_siginfo = false;

        let new_abi = act.map(|act| {
            let mut sa_flags = SigActionFlags::from_bits_truncate(act.sa_flags as usize);

            if act.sa_handler == unsafe { mem::transmute(SIG_DFL) } {
                // DFL is the default
            } else if act.sa_handler == unsafe { mem::transmute(SIG_IGN) } {
                sa_flags |= SigActionFlags::SA_TY_IGN;
            } else {
                sa_flags |= SigActionFlags::SA_TY_USR;
            }

            is_sa_siginfo = sa_flags.contains(SigActionFlags::SA_SIGINFO);

            SigAction {
                sa_mask: act.sa_mask,
                sa_handler: act.sa_handler.map_or(0, |h| h as usize),
                sa_flags,
            }
        });
        let mut old_abi = Some(SigAction::default()).filter(|_| oact.is_some());

        let ret = e(syscall::sigaction(
            sig as usize,
            new_abi.as_ref(),
            old_abi.as_mut(),
        )) as c_int;

        if ret == 0 {
            /*if let Some(act) = act {
                HANDLERS[sig as usize].store(if is_sa_siginfo {
                    Handler { sa_sigaction: unsafe { mem::transmute(act.sa_handler) } }
                } else {
                    Handler { sa_handler: act.sa_handler }
                });
            }*/

            if let (Some(old_abi), Some(oact)) = (old_abi, oact) {
                oact.sa_handler = unsafe { mem::transmute(old_abi.sa_handler) };
                /*
                if old_abi.sa_flags.contains(SigActionFlags::SA_TY_IGN) {
                    oact.sa_handler = unsafe { mem::transmute(SIG_IGN) };
                } else if old_abi.sa_flags.contains(SigActionFlags::SA_TY_USR) {
                    oact.sa_handler = unsafe { mem::transmute(HANDLERS[sig as usize].get()) };
                } else {
                    oact.sa_handler = unsafe { mem::transmute(SIG_DFL) };
                }*/
                oact.sa_mask = old_abi.sa_mask;
                oact.sa_flags = old_abi.sa_flags.bits() as c_ulong;
            }
        }

        ret
    }

    fn sigaltstack(ss: *const stack_t, old_ss: *mut stack_t) -> c_int {
        unimplemented!()
    }

    fn sigpending(set: *mut sigset_t) -> c_int {
        unsafe {
            errno = ENOSYS;
        }
        -1
    }

    fn sigprocmask(how: c_int, set: *const sigset_t, oset: *mut sigset_t) -> c_int {
        let new_opt = unsafe { set.as_ref() };
        let old_opt = unsafe { oset.as_mut() };

        e(syscall::sigprocmask(how as usize, new_opt, old_opt)) as c_int
    }

    fn sigsuspend(set: *const sigset_t) -> c_int {
        unsafe {
            errno = ENOSYS;
        }
        -1
    }

    fn sigtimedwait(set: *const sigset_t, sig: *mut siginfo_t, tp: *const timespec) -> c_int {
        unsafe {
            errno = ENOSYS;
        }
        -1
    }
}

extern "C" {
    fn __relibc_internal_sighandler();
}

struct Stack {
    float_regs: [u8; 512],
    inner: SignalStack,
}

#[derive(Clone, Copy)]
union Handler {
    sa_sigaction: Option<extern "C" fn(c_int, *const siginfo_t, *mut c_void)>,
    sa_handler: Option<extern "C" fn(c_int)>,
}

unsafe extern "C" fn sighandler_inner(stack: &mut Stack) {
    let signal = u8::try_from(stack.inner.signal).expect("signal must be less than 64");
    let flags = stack.inner.sa_flags;

    //let mut handler = HANDLERS[usize::from(signal)].load(Ordering::Acquire);
    let handler: Handler = mem::transmute(stack.inner.sa_handler);

    if flags.contains(SigActionFlags::SA_SIGINFO) {
        let siginfo = siginfo_t {
            si_signo: c_int::from(signal),
            si_errno: 0,
            si_code: 0,
            si_sival: sival { sigval_ptr: stack.inner.sigval as *mut c_void },
            ..Default::default()
        };
        if let Some(sa_sigaction) = handler.sa_sigaction {
            sa_sigaction(c_int::from(signal), &siginfo, (stack as *mut Stack).cast());
        }
    } else {
        if let Some(sa_handler) = handler.sa_handler {
            sa_handler(c_int::from(signal));
        }
    }
}

core::arch::global_asm!("
    .globl __relibc_internal_sighandler
    .type __relibc_internal_sighandler, @function
    .p2align 6
__relibc_internal_sighandler:
    sub rsp, 512
    fxsave [rsp]

    mov rdi, rsp
    call {inner}

    fxrstor [rsp]
    add rsp, 512

    mov rax, {SYS_SIGRETURN}
    syscall

    ud2

    .size __relibc_internal_sighandler, . - __relibc_internal_sighandler
", inner = sym sighandler_inner, SYS_SIGRETURN = const syscall::SYS_SIGRETURN);

pub fn current_altstack() -> usize {
    // TODO
    0
}
pub fn sighandler() -> usize {
    __relibc_internal_sighandler as usize
}

pub unsafe fn init() {
    let mut buf = [0_u8; 2 * core::mem::size_of::<usize>()];
    let (altstack, handler) = buf.split_at_mut(core::mem::size_of::<usize>());
    altstack.copy_from_slice(&current_altstack().to_ne_bytes());
    handler.copy_from_slice(&sighandler().to_ne_bytes());

    let fd = FdGuard::new(syscall::open("thisproc:current/sighandler", O_CLOEXEC).expect("failed to open sighandler fd"));
    let _ = syscall::write(*fd, &buf).expect("failed to write sighandler struct");
}
