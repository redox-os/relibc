use core::cell::Cell;
use core::ffi::c_int;

use syscall::{Result, Sigcontrol};

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

pub fn setup_sighandler(control: &Sigcontrol) {
    #[cfg(target_arch = "x86_64")]
    {
        let cpuid_eax1_ecx = unsafe { core::arch::x86_64::__cpuid(1) }.ecx;
        CPUID_EAX1_ECX.store(cpuid_eax1_ecx, core::sync::atomic::Ordering::Relaxed);
    }

    let data = syscall::SetSighandlerData {
        user_handler: sighandler_function(),
        excp_handler: 0, // TODO
        word_addr: control as *const Sigcontrol as usize,
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

    sa_handler: usize,
    sig_num: usize,
}

#[inline(always)]
unsafe fn inner(stack: &mut SigStack) {
    let handler: extern "C" fn(c_int) = core::mem::transmute(stack.sa_handler);
    handler(stack.sig_num as c_int)
}
#[cfg(not(target_arch = "x86"))]
pub(crate) unsafe extern "C" fn inner_c(stack: usize) {
    inner(&mut *(stack as *mut SigStack))
}
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn inner_fastcall(stack: usize) {
    inner(&mut *(stack as *mut SigStack))
}

pub fn set_sigmask(new: Option<u64>, old: Option<&mut u64>) -> Result<()> {
    todo!()
}
pub fn or_sigmask(new: Option<u64>, old: Option<&mut u64>) -> Result<()> {
    todo!()
}
pub fn andn_sigmask(new: Option<u64>, old: Option<&mut u64>) -> Result<()> {
    todo!()
}

extern "C" {
    pub fn __relibc_internal_get_sigcontrol_addr() -> &'static Sigcontrol;
}

pub struct TmpDisableSignalsGuard { _inner: () }

#[thread_local]
static TMP_DISABLE_SIGNALS_DEPTH: Cell<u64> = Cell::new(0);

pub fn tmp_disable_signals() -> TmpDisableSignalsGuard {
    unsafe {
        let ctl = __relibc_internal_get_sigcontrol_addr().control_flags.get();
        ctl.write_volatile(ctl.read_volatile() | syscall::flag::INHIBIT_DELIVERY);
        // TODO: fence?
        TMP_DISABLE_SIGNALS_DEPTH.set(TMP_DISABLE_SIGNALS_DEPTH.get() + 1);
    }

    TmpDisableSignalsGuard { _inner: () }
}
impl Drop for TmpDisableSignalsGuard {
    fn drop(&mut self) {
        unsafe {
            let old = TMP_DISABLE_SIGNALS_DEPTH.get();
            TMP_DISABLE_SIGNALS_DEPTH.set(old - 1);

            if old == 1 {
                let ctl = __relibc_internal_get_sigcontrol_addr().control_flags.get();
                ctl.write_volatile(ctl.read_volatile() & !syscall::flag::INHIBIT_DELIVERY);
            }
        }
    }
}
