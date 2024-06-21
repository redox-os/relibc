use core::cell::Cell;
use core::ffi::c_int;
use core::sync::atomic::{AtomicU64, Ordering};

use syscall::{Error, IntRegisters, Result, SigProcControl, Sigcontrol, EINVAL, SIGCHLD, SIGCONT, SIGKILL, SIGSTOP, SIGTSTP, SIGTTIN, SIGTTOU, SIGURG, SIGW0_TSTP_IS_STOP_BIT, SIGW0_TTIN_IS_STOP_BIT, SIGW0_TTOU_IS_STOP_BIT, SIGWINCH};

use crate::{arch::*, Tcb};
use crate::sync::Mutex;

#[cfg(target_arch = "x86_64")]
static CPUID_EAX1_ECX: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

pub fn sighandler_function() -> usize {
    #[cfg(target_arch = "x86_64")]
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
    sa_handler: usize,
    sig_num: usize,

    #[cfg(target_arch = "x86_64")]
    fx: [u8; 4096],

    #[cfg(target_arch = "x86")]
    fx: [u8; 512],

    _pad: [usize; 4], // pad to 192 = 3 * 64 bytes
    regs: IntRegisters, // 160 bytes currently
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

pub fn get_sigmask() -> Result<u64> {
    let mut mask = 0;
    modify_sigmask(Some(&mut mask), Option::<fn(u32, bool) -> u32>::None)?;
    Ok(mask)
}
pub fn set_sigmask(new: Option<u64>, old: Option<&mut u64>) -> Result<()> {
    modify_sigmask(old, new.map(move |newmask| move |_, upper| if upper { newmask >> 32 } else { newmask } as u32))
}
pub fn or_sigmask(new: Option<u64>, old: Option<&mut u64>) -> Result<()> {
    // Parsing nightmare...
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
        *old = combine_mask(words);
    }
    let Some(mut op) = op else {
        return Ok(());
    };

    let mut can_raise = 0;
    let mut cant_raise = 0;

    for i in 0..2 {
        while let Err(changed) = ctl.word[i].compare_exchange(words[i], ((words[i] >> 32) << 32) | u64::from(op(words[i] as u32, i == 1)), Ordering::Relaxed, Ordering::Relaxed) {
            // If kernel observed a signal being unblocked and pending simultaneously, it will have
            // set a flag causing it to check for the INHIBIT_SIGNALS flag every time the context
            // is switched to. To avoid race conditions, we should NOT auto-raise those signals in
            // userspace as a result of unblocking it. The kernel will instead take care of that later.
            can_raise |= (changed & (changed >> 32)) << (32 * i);
            cant_raise |= (changed & !(changed >> 32)) << (32 * i);

            words[i] = changed;
        }
    }

    // TODO: Prioritize cant_raise realtime signals?

    Ok(())
}

#[derive(Clone, Copy)]
pub enum Sigaction {
    Default,
    Ignore,
    Handled {
        handler: SignalHandler,
        mask: u64,
        flags: SigactionFlags,
    },
}
pub fn sigaction(signal: u8, new: Option<&Sigaction>, old: Option<&mut Sigaction>) -> Result<()> {
    if matches!(usize::from(signal), 0 | 32 | SIGKILL | SIGSTOP | 65..) {
        return Err(Error::new(EINVAL));
    }

    let _sigguard = tmp_disable_signals();
    let ctl = current_sigctl();
    let guard = SIGACTIONS.lock();
    let old_ignmask = IGNMASK.load(Ordering::Relaxed);

    if let Some(old) = old {
    }

    let Some(new) = new else {
        return Ok(());
    };

    match (usize::from(signal), new) {
        (_, Sigaction::Ignore) | (SIGCHLD | SIGURG | SIGWINCH, Sigaction::Default) => {
            IGNMASK.store(old_ignmask | sig_bit(signal.into()), Ordering::Relaxed);

            // mark the signal as masked
            ctl.word[usize::from(signal) / 32].fetch_or(1 << ((signal - 1) % 32), Ordering::Relaxed);

            // POSIX specifies that pending signals shall be discarded if set to SIG_IGN by
            // sigaction.
            // TODO: handle tmp_disable_signals
        }
        // TODO: Handle pending signals before these flags are set.
        (SIGTSTP, Sigaction::Default) => {
            PROC_CONTROL_STRUCT.word[0].fetch_or(SIGW0_TSTP_IS_STOP_BIT, Ordering::SeqCst);
        }
        (SIGTTIN, Sigaction::Default) => {
            PROC_CONTROL_STRUCT.word[0].fetch_or(SIGW0_TTIN_IS_STOP_BIT, Ordering::SeqCst);
        }
        (SIGTTOU, Sigaction::Default) => {
            PROC_CONTROL_STRUCT.word[0].fetch_or(SIGW0_TTOU_IS_STOP_BIT, Ordering::SeqCst);
        }

        (_, Sigaction::Default) => (),
        (_, Sigaction::Handled { .. }) => (),
    }

    todo!()
}

fn current_sigctl() -> &'static Sigcontrol {
    &unsafe { Tcb::current() }.unwrap().os_specific.control
}

pub struct TmpDisableSignalsGuard { _inner: () }

pub fn tmp_disable_signals() -> TmpDisableSignalsGuard {
    unsafe {
        let ctl = current_sigctl().control_flags.get();
        ctl.write_volatile(ctl.read_volatile() | syscall::flag::INHIBIT_DELIVERY);
        // TODO: fence?
        Tcb::current().unwrap().os_specific.arch.disable_signals_depth += 1;
    }

    TmpDisableSignalsGuard { _inner: () }
}
impl Drop for TmpDisableSignalsGuard {
    fn drop(&mut self) {
        unsafe {
            let depth = &mut Tcb::current().unwrap().os_specific.arch.disable_signals_depth;
            *depth -= 1;

            if *depth == 0 {
                let ctl = current_sigctl().control_flags.get();
                ctl.write_volatile(ctl.read_volatile() & !syscall::flag::INHIBIT_DELIVERY);
            }
        }
    }
}

bitflags::bitflags! {
    // Some flags are ignored by the rt, but they match relibc's 1:1 to simplify conversion.
    #[derive(Clone, Copy)]
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

static SIGACTIONS: Mutex<[Sigaction; 64]> = Mutex::new([Sigaction::Default; 64]);
static IGNMASK: AtomicU64 = AtomicU64::new(sig_bit(SIGCHLD) | sig_bit(SIGURG) | sig_bit(SIGWINCH));

static PROC_CONTROL_STRUCT: SigProcControl = SigProcControl {
    word: [
        AtomicU64::new(SIGW0_TSTP_IS_STOP_BIT | SIGW0_TTIN_IS_STOP_BIT | SIGW0_TTOU_IS_STOP_BIT),
        AtomicU64::new(0),
    ],
};

fn expand_mask(mask: u64) -> [u64; 2] {
    [mask & 0xffff_ffff, mask >> 32]
}
fn combine_mask([lo, hi]: [u64; 2]) -> u64 {
    lo | ((hi & 0xffff_ffff) << 32)
}

const fn sig_bit(sig: usize) -> u64 {
    //assert_ne!(sig, 32);
    //assert_ne!(sig, 0);
    1 << (sig - 1)
}

pub fn setup_sighandler(control: &Sigcontrol) {
    {
        let mut sigactions = SIGACTIONS.lock();
    }

    #[cfg(target_arch = "x86_64")]
    {
        let cpuid_eax1_ecx = unsafe { core::arch::x86_64::__cpuid(1) }.ecx;
        CPUID_EAX1_ECX.store(cpuid_eax1_ecx, core::sync::atomic::Ordering::Relaxed);
    }

    let data = syscall::SetSighandlerData {
        user_handler: sighandler_function(),
        excp_handler: 0, // TODO
        thread_control_addr: control as *const Sigcontrol as usize,
        proc_control_addr: &PROC_CONTROL_STRUCT as *const SigProcControl as usize,
    };

    let fd = syscall::open(
        "thisproc:current/sighandler",
        syscall::O_WRONLY | syscall::O_CLOEXEC,
    )
    .expect("failed to open thisproc:current/sighandler");
    syscall::write(fd, &data).expect("failed to write to thisproc:current/sighandler");
    let _ = syscall::close(fd);
}
#[derive(Debug, Default)]
pub struct RtSigarea {
    pub control: Sigcontrol,
    pub arch: crate::arch::SigArea,
}
