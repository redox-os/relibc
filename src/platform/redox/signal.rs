use core::{
    cell::Cell,
    mem,
    sync::atomic::{AtomicUsize, Ordering},
};
use libc::{SIG_DFL, SIG_IGN};
use redox_exec::FdGuard;

use syscall::{
    data::{SigAction, Sighandler, SignalStack},
    error::{Error, Result, ENOMEM},
    flag::{SigActionFlags, O_CLOEXEC, O_WRONLY},
};

use super::{
    super::{types::*, Pal, PalSignal},
    e,
    path::SignalMask,
    Sys,
};
use crate::{
    header::{
        errno::{EINVAL, ENOSYS},
        signal::{
            sigaction, siginfo_t, sigset_t, sigval, stack_t, MINSIGSTKSZ, SS_DISABLE, SS_ONSTACK,
        },
        sys_time::{itimerval, ITIMER_REAL},
        time::timespec,
    },
    platform::errno,
    pthread::OsTid,
    sync::Mutex,
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

        if sigdebug() {
            dbg!((new_abi, old_abi));
        }

        let ret = e(syscall::sigaction(
            sig as usize,
            new_abi.as_ref(),
            old_abi.as_mut(),
        )) as c_int;

        if sigdebug() {
            dbg!((new_abi, old_abi));
        }

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
        e(sigaltstack_impl(unsafe { ss.as_ref() }, unsafe { old_ss.as_mut() }).map(|()| 0)) as c_int
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

    // TODO: TIDs and PIDs are not the same thing!

    fn sigqueue(pid: pid_t, sig: c_int, val: crate::header::signal::sigval) -> c_int {
        e(syscall::sigqueue(
            pid as usize,
            sig as usize,
            unsafe { val.sigval_ptr } as usize,
        )) as c_int
    }
    fn rlct_sigqueue(tid: OsTid, sig: c_int, val: crate::header::signal::sigval) -> c_int {
        e(syscall::sigqueue(
            tid.context_id as usize,
            sig as usize,
            unsafe { val.sigval_ptr } as usize,
        )) as c_int
    }
}

fn sigaltstack_impl(new: Option<&stack_t>, old: Option<&mut stack_t>) -> Result<()> {
    let old_altstack = ALTSTACK.get();

    if let Some(new) = new {
        if new.ss_flags & !(SS_ONSTACK as c_int | SS_DISABLE as c_int) != 0 {
            return Err(Error::new(EINVAL));
        }

        if new.ss_size < MINSIGSTKSZ {
            return Err(Error::new(ENOMEM));
        }

        let handler = Sighandler {
            altstack_base: new.ss_sp as usize,
            altstack_size: new.ss_size,
            handler: __relibc_internal_sighandler as usize,
        };

        let _ = syscall::write(
            *FdGuard::new(syscall::open(
                "thisproc:current/sighandler",
                O_CLOEXEC | O_WRONLY,
            )?),
            &handler,
        )?;
        ALTSTACK.set(Altstack {
            base: new.ss_sp as usize,
            size: new.ss_size,
        });
    }
    if let Some(old) = old {
        let range = old_altstack.base..old_altstack.base + old_altstack.size;

        old.ss_flags = if range.contains(&get_current_sp()) {
            SS_ONSTACK as c_int
        } else if old_altstack.size == 0 {
            SS_DISABLE as c_int
        } else {
            0
        };
        old.ss_size = old_altstack.size;
        old.ss_sp = old_altstack.base as *mut c_void;
    }
    Ok(())
}

fn get_current_sp() -> usize {
    unsafe {
        let sp: usize;

        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov {}, rsp", out(reg) sp);

        #[cfg(target_arch = "x86")]
        core::arch::asm!("mov {}, esp", out(reg) sp);

        #[cfg(target_arch = "aarch64")]
        core::arch::asm!("mov {}, sp", out(reg) sp);

        sp
    }
}

extern "C" {
    pub fn __relibc_internal_sighandler();
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

//static SIGDEBUG: AtomicBool = AtomicBool::new(false);

fn sigdebug() -> bool {
    false
    //SIGDEBUG.load(Ordering::SeqCst)
}

/*
#[no_mangle]
pub fn __relibc_internal_enable_sigdebug() {
    SIGDEBUG.store(true, Ordering::SeqCst);
}
*/

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
unsafe extern "C" fn sighandler_arch(stack: &mut Stack) {
    sighandler_inner(stack)
}
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn sighandler_arch(stack: &mut Stack) {
    sighandler_inner(stack)
}

#[inline(always)]
unsafe fn sighandler_inner(stack: &mut Stack) {
    let signal = u8::try_from(stack.inner.signal).expect("signal must be less than 64");
    let flags = stack.inner.sa_flags;

    //let mut handler = HANDLERS[usize::from(signal)].load(Ordering::Acquire);
    let handler: Handler = mem::transmute(stack.inner.sa_handler);

    if flags.contains(SigActionFlags::SA_SIGINFO) {
        let siginfo = siginfo_t {
            si_signo: c_int::from(signal),
            si_errno: 0,
            si_code: 0,
            si_value: sigval {
                sigval_ptr: stack.inner.sigval as *mut c_void,
            },
            ..Default::default()
        };
        if sigdebug() {
            dbg!(&siginfo.si_value.sigval_ptr);
        }
        if let Some(sa_sigaction) = handler.sa_sigaction {
            if sigdebug() {
                dbg!();
            }
            sa_sigaction(c_int::from(signal), &siginfo, (stack as *mut Stack).cast());
        }
    } else {
        if sigdebug() {
            dbg!();
        }
        if let Some(sa_handler) = handler.sa_handler {
            if sigdebug() {
                dbg!();
            }
            sa_handler(c_int::from(signal));
        }
    }
}

#[cfg(target_arch = "x86_64")]
core::arch::global_asm!("
    .globl __relibc_internal_sighandler
    .type __relibc_internal_sighandler, @function
    .p2align 6
__relibc_internal_sighandler:
    sub rsp, 512
    fxsave64 [rsp]

    mov rdi, rsp
    call {inner}

    fxrstor64 [rsp]
    add rsp, 512

    mov rax, {SYS_SIGRETURN}
    syscall

    ud2

    .size __relibc_internal_sighandler, . - __relibc_internal_sighandler
", inner = sym sighandler_arch, SYS_SIGRETURN = const syscall::SYS_SIGRETURN);

#[cfg(target_arch = "x86")]
core::arch::global_asm!("
    .globl __relibc_internal_sighandler
    .type __relibc_internal_sighandler, @function
    .p2align 6
__relibc_internal_sighandler:
    sub esp, 512
    fxsave [esp]

    mov ecx, esp
    call {inner}

    fxrstor [esp]
    add esp, 512

    mov eax, {SYS_SIGRETURN}
    int 0x80

    ud2

    .size __relibc_internal_sighandler, . - __relibc_internal_sighandler
", inner = sym sighandler_arch, SYS_SIGRETURN = const syscall::SYS_SIGRETURN);

#[thread_local]
static ALTSTACK: Cell<Altstack> = Cell::new(Altstack { base: 0, size: 0 });

#[derive(Clone, Copy)]
struct Altstack {
    base: usize,
    size: usize,
}

pub fn current_sighandler() -> Sighandler {
    Sighandler {
        altstack_base: ALTSTACK.get().base,
        altstack_size: ALTSTACK.get().size,
        handler: __relibc_internal_sighandler as usize,
    }
}

pub unsafe fn init() {
    let fd = FdGuard::new(
        syscall::open("thisproc:current/sighandler", O_CLOEXEC)
            .expect("failed to open sighandler fd"),
    );
    let _ = syscall::write(*fd, &current_sighandler()).expect("failed to write sighandler struct");
}
