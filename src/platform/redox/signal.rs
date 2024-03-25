use core::mem;
use syscall::{self, number::SYS_SIGRETURN, Result, SetSighandlerData, SignalStack};

use super::{
    super::{types::*, Pal, PalSignal},
    e, Sys,
};
use crate::{
    header::{
        errno::{EINVAL, ENOSYS},
        signal::{sigaction, siginfo_t, sigset_t, stack_t},
        sys_time::{itimerval, ITIMER_REAL},
        time::timespec,
    },
    platform::ERRNO,
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
        e(syscall::kill(pid as usize, sig as usize)) as c_int
    }

    fn killpg(pgrp: pid_t, sig: c_int) -> c_int {
        e(syscall::kill(-(pgrp as isize) as usize, sig as usize)) as c_int
    }

    fn raise(sig: c_int) -> c_int {
        Self::kill(Self::getpid(), sig)
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

    fn sigaction(sig: c_int, act: Option<&sigaction>, oact: Option<&mut sigaction>) -> c_int {
        e(sigaction_impl(sig, act, oact).map(|()| 0)) as c_int
    }

    unsafe fn sigaltstack(ss: *const stack_t, old_ss: *mut stack_t) -> c_int {
        unimplemented!()
    }

    unsafe fn sigpending(set: *mut sigset_t) -> c_int {
        ERRNO.set(ENOSYS);
        -1
    }

    unsafe fn sigprocmask(how: c_int, set: *const sigset_t, oset: *mut sigset_t) -> c_int {
        e(sigprocmask_impl(how, set, oset).map(|()| 0)) as c_int
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

pub(crate) fn sigaction_impl(
    sig: i32,
    act: Option<&sigaction>,
    oact: Option<&mut sigaction>,
) -> Result<()> {
    let new_opt = act.map(|act| {
        let sa_handler = unsafe { mem::transmute(act.sa_handler) };
        syscall::SigAction {
            sa_handler,
            sa_mask: act.sa_mask as u64,
            sa_flags: syscall::SigActionFlags::from_bits(act.sa_flags as usize)
                .expect("sigaction: invalid bit pattern"),
        }
    });
    let mut old_opt = oact.as_ref().map(|_| syscall::SigAction::default());
    syscall::sigaction(sig as usize, new_opt.as_ref(), old_opt.as_mut())?;
    if let (Some(old), Some(oact)) = (old_opt, oact) {
        oact.sa_handler = unsafe { mem::transmute(old.sa_handler) };
        oact.sa_mask = old.sa_mask as sigset_t;
        oact.sa_flags = old.sa_flags.bits() as c_ulong;
    }
    Ok(())
}
pub(crate) unsafe fn sigprocmask_impl(
    how: i32,
    set: *const sigset_t,
    oset: *mut sigset_t,
) -> Result<()> {
    syscall::sigprocmask(how as usize, set.as_ref(), oset.as_mut())?;
    Ok(())
}
#[cfg(target_arch = "x86_64")]
static CPUID_EAX1_ECX: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

pub fn sighandler_function() -> usize {
    #[cfg(target_arch = "x86_64")]
    // Check OSXSAVE bit
    // TODO: HWCAP?
    if CPUID_EAX1_ECX.load(core::sync::atomic::Ordering::Relaxed) & (1 << 27) != 0 {
        __relibc_internal_sigentry_xsave as usize
    } else {
        __relibc_internal_sigentry_fxsave as usize
    }

    #[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
    {
        __relibc_internal_sigentry as usize
    }
}

pub fn setup_sighandler() {
    use core::mem::size_of;

    // TODO
    let altstack_base = 0_usize;
    let altstack_len = 0_usize;

    #[cfg(target_arch = "x86_64")]
    {
        let cpuid_eax1_ecx = unsafe { core::arch::x86_64::__cpuid(1) }.ecx;
        CPUID_EAX1_ECX.store(cpuid_eax1_ecx, core::sync::atomic::Ordering::Relaxed);
    }

    let data = SetSighandlerData {
        entry: sighandler_function(),
        altstack_base,
        altstack_len,
    };

    let fd = syscall::open(
        "thisproc:current/sighandler",
        syscall::O_WRONLY | syscall::O_CLOEXEC,
    )
    .expect("failed to open thisproc:current/sighandler");
    syscall::write(fd, &data).expect("failed to write to thisproc:current/sighandler");
    let _ = syscall::close(fd);
}

#[repr(C)]
pub struct SigStack {
    #[cfg(target_arch = "x86_64")]
    fx: [u8; 4096],

    #[cfg(target_arch = "x86")]
    fx: [u8; 512],

    kernel_pushed: SignalStack,
}

#[inline(always)]
unsafe fn inner(stack: &mut SigStack) {
    let handler: extern "C" fn(c_int) = core::mem::transmute(stack.kernel_pushed.sa_handler);
    handler(stack.kernel_pushed.sig_num as c_int)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn inner_c(stack: usize) {
    inner(&mut *(stack as *mut SigStack))
}
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn inner_fastcall(stack: usize) {
    inner(&mut *(stack as *mut SigStack))
}

// TODO: is the memset necessary?
#[cfg(target_arch = "x86_64")]
asmfunction!(__relibc_internal_sigentry_xsave: ["
    sub rsp, 4096

    cld
    mov rdi, rsp
    xor eax, eax
    mov ecx, 4096
    rep stosb

    mov eax, 0xffffffff
    mov edx, eax
    xsave [rsp]

    mov rdi, rsp
    call {inner}

    mov eax, 0xffffffff
    mov edx, eax
    xrstor [rsp]
    add rsp, 4096

    mov eax, {SYS_SIGRETURN}
    syscall
"] <= [inner = sym inner_c, SYS_SIGRETURN = const SYS_SIGRETURN]);

#[cfg(target_arch = "x86_64")]
asmfunction!(__relibc_internal_sigentry_fxsave: ["
    sub rsp, 4096

    fxsave64 [rsp]

    mov rdi, rsp
    call {inner}

    fxrstor64 [rsp]
    add rsp, 4096

    mov eax, {SYS_SIGRETURN}
    syscall
"] <= [inner = sym inner_c, SYS_SIGRETURN = const SYS_SIGRETURN]);

#[cfg(target_arch = "x86")]
asmfunction!(__relibc_internal_sigentry: ["
    sub esp, 512
    fxsave [esp]

    mov ecx, esp
    call {inner}

    add esp, 512
    fxrstor [esp]

    mov eax, {SYS_SIGRETURN}
    int 0x80
"] <= [inner = sym inner_fastcall, SYS_SIGRETURN = const SYS_SIGRETURN]);

#[cfg(target_arch = "aarch64")]
asmfunction!(__relibc_internal_sigentry: ["
    mov x0, sp
    bl {inner}

    mov x8, {SYS_SIGRETURN}
    svc 0
"] <= [inner = sym inner_c, SYS_SIGRETURN = const SYS_SIGRETURN]);
