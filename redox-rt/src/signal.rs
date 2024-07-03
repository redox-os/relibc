use core::cell::{Cell, UnsafeCell};
use core::ffi::c_int;
use core::sync::atomic::{AtomicUsize, Ordering};

use syscall::{RawAction, ENOMEM, EPERM, SIGABRT, SIGBUS, SIGFPE, SIGILL, SIGQUIT, SIGSEGV, SIGSYS, SIGTRAP, SIGXCPU, SIGXFSZ};
use syscall::{Error, Result, SetSighandlerData, SigProcControl, Sigcontrol, SigcontrolFlags, EINVAL, SIGCHLD, SIGCONT, SIGKILL, SIGSTOP, SIGTSTP, SIGTTIN, SIGTTOU, SIGURG, SIGWINCH, data::AtomicU64};

use crate::{arch::*, Tcb};
use crate::sync::Mutex;

#[cfg(target_arch = "x86_64")]
static CPUID_EAX1_ECX: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

pub fn sighandler_function() -> usize {
    // TODO: HWCAP?

    __relibc_internal_sigentry as usize
}

#[repr(C)]
pub struct SigStack {
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    _pad: [usize; 1], // pad to 16 bytes alignment

    #[cfg(target_arch = "x86")]
    _pad: [usize; 3], // pad to 16 bytes alignment

    sig_num: usize,

    // x86_64: 864 bytes
    // i686: 512 bytes
    // aarch64: 272 bytes (SIMD TODO)
    pub regs: ArchIntRegs,
}

#[inline(always)]
unsafe fn inner(stack: &mut SigStack) {
    let os = &Tcb::current().unwrap().os_specific;

    let sig_idx = stack.sig_num;

    // Commenting out this line will stress test how the signal code responds to 'interrupt spraying'.
    os.control.word[sig_idx / 32].fetch_and(!(1 << (sig_idx % 32)), Ordering::Relaxed);

    // asm counts from 0
    stack.sig_num += 1;

    arch_pre(stack, &mut *os.arch.get());

    let sigaction = {
        let mut guard = SIGACTIONS_LOCK.lock();
        let action = convert_old(&PROC_CONTROL_STRUCT.actions[stack.sig_num - 1]);
        if action.flags.contains(SigactionFlags::RESETHAND) {
            // TODO: other things that must be set
            drop(guard);
            sigaction(stack.sig_num as u8, Some(&Sigaction {
                kind: SigactionKind::Default,
                mask: 0,
                flags: SigactionFlags::empty(),
            }), None);
        }
        action
    };

    let handler = match sigaction.kind {
        SigactionKind::Ignore => {
            panic!("ctl {:x?} signal {}", os.control, stack.sig_num)
        }
        SigactionKind::Default => {
            syscall::exit(stack.sig_num << 8);
            unreachable!();
        }
        SigactionKind::Handled { handler } => handler,
    };

    // Set sigmask to sa_mask and unmark the signal as pending.
    let prev_sigallow_lo = os.control.word[0].load(Ordering::Relaxed) >> 32;
    let prev_sigallow_hi = os.control.word[1].load(Ordering::Relaxed) >> 32;
    let prev_sigallow = prev_sigallow_lo | (prev_sigallow_hi << 32);

    let mut sigallow_inside = !sigaction.mask & prev_sigallow;
    if !sigaction.flags.contains(SigactionFlags::NODEFER) {
        sigallow_inside &= !sig_bit(stack.sig_num);
    }
    let sigallow_inside_lo = sigallow_inside & 0xffff_ffff;
    let sigallow_inside_hi = sigallow_inside >> 32;

    //let _ = syscall::write(1, &alloc::format!("WORD0 {:x?}\n", os.control.word).as_bytes());
    let prev_w0 = os.control.word[0].fetch_add((sigallow_inside_lo << 32).wrapping_sub(prev_sigallow_lo << 32), Ordering::Relaxed);
    let prev_w1 = os.control.word[1].fetch_add((sigallow_inside_hi << 32).wrapping_sub(prev_sigallow_hi << 32), Ordering::Relaxed);
    //let _ = syscall::write(1, &alloc::format!("WORD1 {:x?}\n", os.control.word).as_bytes());

    // TODO: If sa_mask caused signals to be unblocked, deliver one or all of those first?

    // Re-enable signals again.
    let control_flags = &os.control.control_flags;
    control_flags.store(control_flags.load(Ordering::Relaxed) & !SigcontrolFlags::INHIBIT_DELIVERY.bits(), Ordering::Release);
    core::sync::atomic::compiler_fence(Ordering::Acquire);

    // Call handler, either sa_handler or sa_siginfo depending on flag.
    if sigaction.flags.contains(SigactionFlags::SIGINFO) && let Some(sigaction) = handler.sigaction {
        //let _ = syscall::write(1, alloc::format!("SIGACTION {:p}\n", sigaction).as_bytes());
        sigaction(stack.sig_num as c_int, core::ptr::null_mut(), core::ptr::null_mut());
    } else if let Some(handler) = handler.handler {
        //let _ = syscall::write(1, alloc::format!("HANDLER {:p}\n", handler).as_bytes());
        handler(stack.sig_num as c_int);
    }
    //let _ = syscall::write(1, alloc::format!("RETURNED HANDLER\n").as_bytes());

    // Disable signals while we modify the sigmask again
    control_flags.store(control_flags.load(Ordering::Relaxed) | SigcontrolFlags::INHIBIT_DELIVERY.bits(), Ordering::Release);
    core::sync::atomic::compiler_fence(Ordering::Acquire);

    // Update allowset again.
    //let _ = syscall::write(1, &alloc::format!("WORD2 {:x?}\n", os.control.word).as_bytes());

    let prev_w0 = os.control.word[0].fetch_add((prev_sigallow_lo << 32).wrapping_sub(sigallow_inside_lo << 32), Ordering::Relaxed);
    let prev_w1 = os.control.word[1].fetch_add((prev_sigallow_hi << 32).wrapping_sub(sigallow_inside_hi << 32), Ordering::Relaxed);
    //let _ = syscall::write(1, &alloc::format!("WORD3 {:x?}\n", os.control.word).as_bytes());

    // TODO: If resetting the sigmask caused signals to be unblocked, then should they be delivered
    // here? And would it be possible to tail-call-optimize that?

    //let _ = syscall::write(1, alloc::format!("will return to {:x?}\n", stack.regs.eip).as_bytes());

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

    //let _ = syscall::write(1, &alloc::format!("OLDWORD {:x?}\n", ctl.word).as_bytes());
    for i in 0..2 {
        let old_allow_bits = words[i] & 0xffff_ffff_0000_0000;
        let new_allow_bits = u64::from(!op(!((old_allow_bits >> 32) as u32), i == 1)) << 32;

        ctl.word[i].fetch_add(new_allow_bits.wrapping_sub(old_allow_bits), Ordering::Relaxed);
    }
    //let _ = syscall::write(1, &alloc::format!("NEWWORD {:x?}\n", ctl.word).as_bytes());

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

impl Sigaction {
    fn ip(&self) -> usize {
        unsafe {
            match self.kind {
                SigactionKind::Handled { handler } => if self.flags.contains(SigactionFlags::SIGINFO) {
                    handler.sigaction.map_or(0, |a| a as usize)
                } else {
                    handler.handler.map_or(0, |a| a as usize)
                }
                _ => 0,
            }
        }
    }
}

const MASK_DONTCARE: u64 = !0;

fn convert_old(action: &RawAction) -> Sigaction {
    let old_first = action.first.load(Ordering::Relaxed);
    let old_mask = action.user_data.load(Ordering::Relaxed);

    let handler = (old_first & !(u64::from(STORED_FLAGS) << 32)) as usize;
    let flags = SigactionFlags::from_bits_retain(((old_first >> 32) as u32) & STORED_FLAGS);

    let kind = if handler == default_handler as usize {
        SigactionKind::Default
    } else if flags.contains(SigactionFlags::IGNORED) {
        SigactionKind::Ignore
    } else {
        SigactionKind::Handled { handler: unsafe { core::mem::transmute(handler as usize) } }
    };

    Sigaction {
        mask: old_mask,
        flags,
        kind,
    }
}

pub fn sigaction(signal: u8, new: Option<&Sigaction>, old: Option<&mut Sigaction>) -> Result<()> {
    if matches!(usize::from(signal), 0 | 32 | SIGKILL | SIGSTOP | 65..) {
        return Err(Error::new(EINVAL));
    }

    let _sigguard = tmp_disable_signals();
    let ctl = current_sigctl();

    let _guard = SIGACTIONS_LOCK.lock();

    let action = &PROC_CONTROL_STRUCT.actions[usize::from(signal) - 1];

    if let Some(old) = old {
        *old = convert_old(action);
    }

    let Some(new) = new else {
        return Ok(());
    };

    let explicit_handler = new.ip();

    let (mask, flags, handler) = match (usize::from(signal), new.kind) {
        (_, SigactionKind::Ignore) | (SIGURG | SIGWINCH, SigactionKind::Default) => {
            // TODO: POSIX specifies that pending signals shall be discarded if set to SIG_IGN by
            // sigaction.
            // TODO: handle tmp_disable_signals
            (MASK_DONTCARE, SigactionFlags::IGNORED, if matches!(new.kind, SigactionKind::Default) {
                default_handler as usize
            } else {
                0
            })
        }
        // TODO: Handle pending signals before these flags are set.
        (SIGTSTP | SIGTTOU | SIGTTIN, SigactionKind::Default) => (MASK_DONTCARE, SigactionFlags::SIG_SPECIFIC, default_handler as usize),
        (SIGCHLD, SigactionKind::Default) => {
            let nocldstop_bit = new.flags & SigactionFlags::SIG_SPECIFIC;
            (MASK_DONTCARE, SigactionFlags::IGNORED | nocldstop_bit, default_handler as usize)
        }

        (_, SigactionKind::Default) => {
            (new.mask, new.flags, default_handler as usize)
        },
        (_, SigactionKind::Handled { .. }) => {
            (new.mask, new.flags, explicit_handler)
        }
    };
    let new_first = (handler as u64) | (u64::from(flags.bits() & STORED_FLAGS) << 32);
    action.first.store(new_first, Ordering::Relaxed);
    action.user_data.store(mask, Ordering::Relaxed);

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
        const NOCLDWAIT = 2;
        const RESTORER = 4;
        const SIGINFO = 0x0200_0000;
        const ONSTACK = 0x0400_0000;
        const RESTART = 0x0800_0000;
        const NODEFER = 0x1000_0000;
        const RESETHAND = 0x2000_0000;
        const SIG_SPECIFIC = 0x4000_0000;
        const IGNORED = 0x8000_0000;
    }
}

const STORED_FLAGS: u32 = 0xfe00_0000;

fn default_handler(sig: c_int) {
    syscall::exit((sig as usize) << 8);
}

#[derive(Clone, Copy)]
pub union SignalHandler {
    pub handler: Option<extern "C" fn(c_int)>,
    pub sigaction: Option<unsafe extern "C" fn(c_int, *const (), *mut ())>,
}

static SIGACTIONS_LOCK: Mutex<()> = Mutex::new(());

pub(crate) static PROC_CONTROL_STRUCT: SigProcControl = SigProcControl {
    pending: AtomicU64::new(0),
    actions: [const {
        RawAction {
            first: AtomicU64::new(0),
            user_data: AtomicU64::new(0),
        }
    }; 64],
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
        let mut sigactions = SIGACTIONS_LOCK.lock();
        for (sig_idx, action) in PROC_CONTROL_STRUCT.actions.iter().enumerate() {
            let sig = sig_idx + 1;
            let bits = if matches!(sig, SIGTSTP | SIGTTIN | SIGTTOU) {
                SigactionFlags::SIG_SPECIFIC
            } else if matches!(sig, SIGCHLD | SIGURG | SIGWINCH) {
                SigactionFlags::IGNORED
            } else {
                SigactionFlags::empty()
            };
            action.first.store((u64::from(bits.bits()) << 32) | default_handler as u64, Ordering::Relaxed);
        }
    }
    let arch = unsafe { &mut *area.arch.get() };
    {
        // The asm decides whether to use the altstack, based on whether the saved stack pointer
        // was already on that stack. Thus, setting the altstack to the entire address space, is
        // equivalent to not using any altstack at all (the default).
        arch.altstack_top = usize::MAX;
        arch.altstack_bottom = 0;
        #[cfg(not(target_arch = "x86"))]
        {
            arch.pctl = core::ptr::addr_of!(PROC_CONTROL_STRUCT) as usize;
        }
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

#[derive(Clone, Copy, Default, PartialEq)]
pub enum Sigaltstack {
    #[default]
    Disabled,

    Enabled { onstack: bool, base: *mut (), size: usize },
}
pub unsafe fn sigaltstack(new: Option<&Sigaltstack>, old_out: Option<&mut Sigaltstack>) -> Result<()> {
    let _g = tmp_disable_signals();
    let tcb = &mut *Tcb::current().unwrap().os_specific.arch.get();

    let old = if tcb.altstack_bottom == 0 && tcb.altstack_top == usize::MAX {
        Sigaltstack::Disabled
    } else {
        Sigaltstack::Enabled {
            base: tcb.altstack_bottom as *mut (),
            size: tcb.altstack_top - tcb.altstack_bottom,
            onstack: (tcb.altstack_bottom..tcb.altstack_top).contains(&crate::arch::current_sp()),
        }
    };

    if matches!(old, Sigaltstack::Enabled { onstack: true, .. }) && new != Some(&old) {
        return Err(Error::new(EPERM));
    }

    if let Some(old_out) = old_out {
        *old_out = old;
    }
    if let Some(new) = new {
        match *new {
            Sigaltstack::Disabled => {
                tcb.altstack_bottom = 0;
                tcb.altstack_top = usize::MAX;
            }
            Sigaltstack::Enabled { onstack: true, .. } => return Err(Error::new(EINVAL)),
            Sigaltstack::Enabled { base, size, onstack: false } => {
                if size < MIN_SIGALTSTACK_SIZE {
                    return Err(Error::new(ENOMEM));
                }

                tcb.altstack_bottom = base as usize;
                tcb.altstack_top = base as usize + size;
            }
        }
    }
    Ok(())
}

pub const MIN_SIGALTSTACK_SIZE: usize = 8192;

pub fn currently_pending() -> u64 {
    let control = &unsafe { Tcb::current().unwrap() }.os_specific.control;
    let w0 = control.word[0].load(Ordering::Relaxed);
    let w1 = control.word[1].load(Ordering::Relaxed);
    let pending_blocked_lo = w0 & !(w0 >> 32);
    let pending_unblocked_hi = w1 & !(w0 >> 32);
    pending_blocked_lo | (pending_unblocked_hi << 32)
}
