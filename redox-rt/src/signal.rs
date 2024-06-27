use core::cell::{Cell, UnsafeCell};
use core::ffi::c_int;
use core::sync::atomic::Ordering;

use syscall::{Error, Result, SetSighandlerData, SigProcControl, Sigcontrol, SigcontrolFlags, EINVAL, SIGCHLD, SIGCONT, SIGKILL, SIGSTOP, SIGTSTP, SIGTTIN, SIGTTOU, SIGURG, SIGW0_NOCLDSTOP_BIT, SIGW0_TSTP_IS_STOP_BIT, SIGW0_TTIN_IS_STOP_BIT, SIGW0_TTOU_IS_STOP_BIT, SIGWINCH, data::AtomicU64};

use crate::{arch::*, Tcb};
use crate::sync::Mutex;

#[cfg(target_arch = "x86_64")]
static CPUID_EAX1_ECX: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

pub fn sighandler_function() -> usize {
    //#[cfg(target_arch = "x86_64")]
    // Check OSXSAVE bit
    // TODO: HWCAP?
    /*if CPUID_EAX1_ECX.load(core::sync::atomic::Ordering::Relaxed) & (1 << 27) != 0 {
        __relibc_internal_sigentry_xsave as usize
    } else {
        __relibc_internal_sigentry_fxsave as usize
    }*/

    //#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
    {
        __relibc_internal_sigentry as usize
    }
}

#[repr(C)]
pub struct SigStack {
    #[cfg(target_arch = "x86_64")]
    fx: [u8; 4096], // 64 byte aligned

    #[cfg(target_arch = "x86")]
    fx: [u8; 512], // 16 byte aligned

    #[cfg(target_arch = "x86_64")]
    _pad: [usize; 3], // pad to 192 = 3 * 64 = 168 + 24 bytes

    #[cfg(target_arch = "x86")]
    _pad: [usize; 3], // pad to 64 = 4 * 16 = 52 + 12 bytes

    sig_num: usize,

    // x86_64: 160 bytes
    // i686: 48 bytes
    pub regs: ArchIntRegs,
}

#[inline(always)]
unsafe fn inner(stack: &mut SigStack) {
    let os = &Tcb::current().unwrap().os_specific;

    // asm counts from 0
    stack.sig_num += 1;

    arch_pre(stack, &mut *os.arch.get());

    let sigaction = {
        let mut guard = SIGACTIONS.lock();
        let action = guard[stack.sig_num];
        if action.flags.contains(SigactionFlags::RESETHAND) {
            // TODO: other things that must be set
            guard[stack.sig_num].kind = SigactionKind::Default;
        }
        action
    };

    let handler = match sigaction.kind {
        SigactionKind::Ignore => unreachable!(),
        SigactionKind::Default => {
            syscall::exit(stack.sig_num << 8);
            unreachable!();
        }
        SigactionKind::Handled { handler } => handler,
    };

    let mut sigallow_inside = !sigaction.mask;
    if !sigaction.flags.contains(SigactionFlags::NODEFER) {
        sigallow_inside &= !sig_bit(stack.sig_num);
    }
    let sigallow_inside_lo = sigallow_inside & 0xffff_ffff;
    let sigallow_inside_hi = sigallow_inside >> 32;

    // Set sigmask to sa_mask and unmark the signal as pending.
    let prev_sigallow_lo = os.control.word[0].load(Ordering::Relaxed) >> 32;
    let prev_sigallow_hi = os.control.word[1].load(Ordering::Relaxed) >> 32;
    let prev_sigallow = prev_sigallow_lo | (prev_sigallow_hi << 32);

    let sig_group = stack.sig_num / 32;

    let prev_w0 = os.control.word[0].fetch_add(sigallow_inside_lo.wrapping_sub(prev_sigallow_lo), Ordering::Relaxed);
    let prev_w1 = os.control.word[1].fetch_add(sigallow_inside_hi.wrapping_sub(prev_sigallow_hi), Ordering::Relaxed);

    // TODO: If sa_mask caused signals to be unblocked, deliver one or all of those first?

    // Re-enable signals again.
    let control_flags = &os.control.control_flags;
    control_flags.store(control_flags.load(Ordering::Relaxed) & !SigcontrolFlags::INHIBIT_DELIVERY.bits(), Ordering::Release);
    core::sync::atomic::compiler_fence(Ordering::Acquire);

    // Call handler, either sa_handler or sa_siginfo depending on flag.
    if sigaction.flags.contains(SigactionFlags::SIGINFO) && let Some(sigaction) = handler.sigaction {
        sigaction(stack.sig_num as c_int, core::ptr::null_mut(), core::ptr::null_mut());
    } else if let Some(handler) = handler.handler {
        handler(stack.sig_num as c_int);
    }

    // Disable signals while we modify the sigmask again
    control_flags.store(control_flags.load(Ordering::Relaxed) | SigcontrolFlags::INHIBIT_DELIVERY.bits(), Ordering::Release);
    core::sync::atomic::compiler_fence(Ordering::Acquire);

    // Update allowset again.
    let prev_w0 = os.control.word[0].fetch_add(prev_sigallow_lo.wrapping_sub(sigallow_inside_lo), Ordering::Relaxed);
    let prev_w1 = os.control.word[1].fetch_add(prev_sigallow_hi.wrapping_sub(sigallow_inside_hi), Ordering::Relaxed);

    // TODO: If resetting the sigmask caused signals to be unblocked, then should they be delivered
    // here? And would it be possible to tail-call-optimize that?

    // And re-enable them again
    control_flags.store(control_flags.load(Ordering::Relaxed) & !SigcontrolFlags::INHIBIT_DELIVERY.bits(), Ordering::Release);
    core::sync::atomic::compiler_fence(Ordering::Acquire);
}
#[cfg(not(target_arch = "x86"))]
pub(crate) unsafe extern "C" fn inner_c(stack: usize) {
    inner(&mut *(stack as *mut SigStack))
}
#[cfg(target_arch = "x86")]
pub(crate) unsafe extern "fastcall" fn inner_fastcall(stack: usize) {
    inner(&mut *(stack as *mut SigStack))
}

pub fn get_sigmask() -> Result<u64> {
    let mut mask = 0;
    modify_sigmask(Some(&mut mask), Option::<fn(u32, bool) -> u32>::None)?;
    Ok(mask)
}
pub fn set_sigmask(new: Option<u64>, old: Option<&mut u64>) -> Result<()> {
    modify_sigmask(old, new.map(move |newmask| move |_, upper| if upper { newmask >> 32 } else { newmask } as u32))
}
pub fn or_sigmask(new: Option<u64>, old: Option<&mut u64>) -> Result<()> {
    // Parsing nightmare... :)
    modify_sigmask(old, new.map(move |newmask| move |oldmask, upper| oldmask | if upper { newmask >> 32 } else { newmask } as u32))
}
pub fn andn_sigmask(new: Option<u64>, old: Option<&mut u64>) -> Result<()> {
    modify_sigmask(old, new.map(move |newmask| move |oldmask, upper| oldmask & !if upper { newmask >> 32 } else { newmask } as u32))
}
fn modify_sigmask(old: Option<&mut u64>, op: Option<impl FnMut(u32, bool) -> u32>) -> Result<()> {
    let _guard = tmp_disable_signals();
    let ctl = current_sigctl();

    let mut words = ctl.word.each_ref().map(|w| w.load(Ordering::Relaxed));

    if let Some(old) = old {
        *old = !combine_allowset(words);
    }
    let Some(mut op) = op else {
        return Ok(());
    };

    let mut can_raise = 0;
    let mut cant_raise = 0;

    for i in 0..2 {
        let pending_bits = words[i] & 0xffff_ffff;
        let old_allow_bits = words[i] & 0xffff_ffff_0000_0000;
        let new_allow_bits = u64::from(!op(!((old_allow_bits >> 32) as u32), i == 1)) << 32;

        ctl.word[i].fetch_add(new_allow_bits.wrapping_sub(old_allow_bits), Ordering::Relaxed);
    }

    // TODO: Prioritize cant_raise realtime signals?

    Ok(())
}

#[derive(Clone, Copy, Default)]
pub enum SigactionKind {
    #[default]
    Default,

    Ignore,
    Handled {
        handler: SignalHandler,
    },
}

#[derive(Clone, Copy, Default)]
pub struct Sigaction {
    pub kind: SigactionKind,
    pub mask: u64,
    pub flags: SigactionFlags,
}

pub fn sigaction(signal: u8, new: Option<&Sigaction>, old: Option<&mut Sigaction>) -> Result<()> {
    if matches!(usize::from(signal), 0 | 32 | SIGKILL | SIGSTOP | 65..) {
        return Err(Error::new(EINVAL));
    }

    let _sigguard = tmp_disable_signals();
    let ctl = current_sigctl();
    let mut guard = SIGACTIONS.lock();
    let old_ignmask = IGNMASK.load(Ordering::Relaxed);

    if let Some(old) = old {
        *old = guard[usize::from(signal)];
    }

    let Some(new) = new else {
        return Ok(());
    };
    guard[usize::from(signal)] = *new;

    let sig_group = usize::from(signal) / 32;
    let sig_bit32 = 1 << ((signal - 1) % 32);

    match (usize::from(signal), new.kind) {
        (_, SigactionKind::Ignore) | (SIGURG | SIGWINCH, SigactionKind::Default) => {
            IGNMASK.store(old_ignmask | sig_bit(signal.into()), Ordering::Relaxed);

            // mark the signal as masked
            ctl.word[sig_group].fetch_or(sig_bit32, Ordering::Relaxed);

            // POSIX specifies that pending signals shall be discarded if set to SIG_IGN by
            // sigaction.
            // TODO: handle tmp_disable_signals
        }
        // TODO: Handle pending signals before these flags are set.
        (SIGTSTP, SigactionKind::Default) => {
            PROC_CONTROL_STRUCT.word[0].fetch_or(SIGW0_TSTP_IS_STOP_BIT, Ordering::SeqCst);
        }
        (SIGTTIN, SigactionKind::Default) => {
            PROC_CONTROL_STRUCT.word[0].fetch_or(SIGW0_TTIN_IS_STOP_BIT, Ordering::SeqCst);
        }
        (SIGTTOU, SigactionKind::Default) => {
            PROC_CONTROL_STRUCT.word[0].fetch_or(SIGW0_TTOU_IS_STOP_BIT, Ordering::SeqCst);
        }
        (SIGCHLD, SigactionKind::Default) => {
            if new.flags.contains(SigactionFlags::NOCLDSTOP) {
                PROC_CONTROL_STRUCT.word[0].fetch_or(SIGW0_NOCLDSTOP_BIT, Ordering::SeqCst);
            } else {
                PROC_CONTROL_STRUCT.word[0].fetch_and(!SIGW0_NOCLDSTOP_BIT, Ordering::SeqCst);
            }
            IGNMASK.store(old_ignmask | sig_bit(signal.into()), Ordering::Relaxed);

            // mark the signal as masked
            ctl.word[sig_group].fetch_or(sig_bit32, Ordering::Relaxed);
        }

        (_, SigactionKind::Default) => {
            IGNMASK.store(old_ignmask & !sig_bit(signal.into()), Ordering::Relaxed);

            // TODO: update mask
            //ctl.word[usize::from(signal)].fetch_or();
        },
        (_, SigactionKind::Handled { .. }) => (),
    }

    Ok(())
}

fn current_sigctl() -> &'static Sigcontrol {
    &unsafe { Tcb::current() }.unwrap().os_specific.control
}

pub struct TmpDisableSignalsGuard { _inner: () }

pub fn tmp_disable_signals() -> TmpDisableSignalsGuard {
    unsafe {
        let ctl = &current_sigctl().control_flags;
        ctl.store(ctl.load(Ordering::Relaxed) | syscall::flag::INHIBIT_DELIVERY.bits(), Ordering::Release);
        core::sync::atomic::compiler_fence(Ordering::Acquire);

        // TODO: fence?
        (*Tcb::current().unwrap().os_specific.arch.get()).disable_signals_depth += 1;
    }

    TmpDisableSignalsGuard { _inner: () }
}
impl Drop for TmpDisableSignalsGuard {
    fn drop(&mut self) {
        unsafe {
            let depth = &mut (*Tcb::current().unwrap().os_specific.arch.get()).disable_signals_depth;
            *depth -= 1;

            if *depth == 0 {
                let ctl = &current_sigctl().control_flags;
                ctl.store(ctl.load(Ordering::Relaxed) & !syscall::flag::INHIBIT_DELIVERY.bits(), Ordering::Release);
                core::sync::atomic::compiler_fence(Ordering::Acquire);
            }
        }
    }
}

bitflags::bitflags! {
    // Some flags are ignored by the rt, but they match relibc's 1:1 to simplify conversion.
    #[derive(Clone, Copy, Default)]
    pub struct SigactionFlags: u32 {
        const NOCLDSTOP = 1;
        const NOCLDWAIT = 2;
        const SIGINFO = 4;
        const RESTORER = 0x0400_0000;
        const ONSTACK = 0x0800_0000;
        const RESTART = 0x1000_0000;
        const NODEFER = 0x4000_0000;
        const RESETHAND = 0x8000_0000;
    }
}
fn default_term_handler(sig: c_int) {
    syscall::exit((sig as usize) << 8);
}
fn default_core_handler(sig: c_int) {
    syscall::exit((sig as usize) << 8);
}
fn default_ign_handler(_: c_int) {
}
fn stop_handler_sentinel(_: c_int) {
}

#[derive(Clone, Copy)]
pub union SignalHandler {
    pub handler: Option<extern "C" fn(c_int)>,
    pub sigaction: Option<unsafe extern "C" fn(c_int, *const (), *mut ())>,
}

struct TheDefault {
    actions: [Sigaction; 64],
    ignmask: u64,
}

// indexed directly by signal number
static SIGACTIONS: Mutex<[Sigaction; 64]> = Mutex::new([Sigaction { flags: SigactionFlags::empty(), mask: 0, kind: SigactionKind::Default }; 64]);

static IGNMASK: AtomicU64 = AtomicU64::new(sig_bit(SIGCHLD) | sig_bit(SIGURG) | sig_bit(SIGWINCH));

static PROC_CONTROL_STRUCT: SigProcControl = SigProcControl {
    word: [
        //AtomicU64::new(SIGW0_TSTP_IS_STOP_BIT | SIGW0_TTIN_IS_STOP_BIT | SIGW0_TTOU_IS_STOP_BIT | 0xffff_ffff_0000_0000),
        AtomicU64::new(0xffff_ffff_0000_0000), // "allow all, no pending"
        AtomicU64::new(0xffff_ffff_0000_0000), // "allow all, no pending"
    ],
};

fn combine_allowset([lo, hi]: [u64; 2]) -> u64 {
    (lo >> 32) | ((hi >> 32) << 32)
}

const fn sig_bit(sig: usize) -> u64 {
    //assert_ne!(sig, 32);
    //assert_ne!(sig, 0);
    1 << (sig - 1)
}

pub fn setup_sighandler(area: &RtSigarea) {
    {
        let mut sigactions = SIGACTIONS.lock();
    }
    let arch = unsafe { &mut *area.arch.get() };
    {
        // The asm decides whether to use the altstack, based on whether the saved stack pointer
        // was already on that stack. Thus, setting the altstack to the entire address space, is
        // equivalent to not using any altstack at all (the default).
        arch.altstack_top = usize::MAX;
        arch.altstack_bottom = 0;
        arch.onstack = 0;
    }

    #[cfg(target_arch = "x86_64")]
    {
        let cpuid_eax1_ecx = unsafe { core::arch::x86_64::__cpuid(1) }.ecx;
        CPUID_EAX1_ECX.store(cpuid_eax1_ecx, core::sync::atomic::Ordering::Relaxed);
    }

    let data = current_setsighandler_struct();

    let fd = syscall::open(
        "thisproc:current/sighandler",
        syscall::O_WRONLY | syscall::O_CLOEXEC,
    )
    .expect("failed to open thisproc:current/sighandler");
    syscall::write(fd, &data).expect("failed to write to thisproc:current/sighandler");
    let _ = syscall::close(fd);

    // TODO: Inherited set of ignored signals
    set_sigmask(Some(0), None);
}
#[derive(Debug, Default)]
pub struct RtSigarea {
    pub control: Sigcontrol,
    pub arch: UnsafeCell<crate::arch::SigArea>,
}
pub fn current_setsighandler_struct() -> SetSighandlerData {
    SetSighandlerData {
        user_handler: sighandler_function(),
        excp_handler: 0, // TODO
        thread_control_addr: core::ptr::addr_of!(unsafe { Tcb::current() }.unwrap().os_specific.control) as usize,
        proc_control_addr: &PROC_CONTROL_STRUCT as *const SigProcControl as usize,
    }
}
