use core::ffi::c_int;

use crate::arch::*;

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
    // TODO
    let altstack_base = 0_usize;
    let altstack_len = 0_usize;

    #[cfg(target_arch = "x86_64")]
    {
        let cpuid_eax1_ecx = unsafe { core::arch::x86_64::__cpuid(1) }.ecx;
        CPUID_EAX1_ECX.store(cpuid_eax1_ecx, core::sync::atomic::Ordering::Relaxed);
    }

    let data = syscall::SetSighandlerData {
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

    kernel_pushed: syscall::SignalStack,
}

#[inline(always)]
unsafe fn inner(stack: &mut SigStack) {
    let handler: extern "C" fn(c_int) = core::mem::transmute(stack.kernel_pushed.sa_handler);
    handler(stack.kernel_pushed.sig_num as c_int)
}
#[cfg(not(target_arch = "x86"))]
pub(crate) unsafe extern "C" fn inner_c(stack: usize) {
    inner(&mut *(stack as *mut SigStack))
}
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn inner_fastcall(stack: usize) {
    inner(&mut *(stack as *mut SigStack))
}
