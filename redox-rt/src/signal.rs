use core::{ffi::c_int, ptr::NonNull, sync::atomic::Ordering};

use syscall::{
    CallFlags, EAGAIN, EINTR, EINVAL, ENOMEM, EPERM, Error, RawAction, Result, SenderInfo,
    SetSighandlerData, SigProcControl, Sigcontrol, SigcontrolFlags, TimeSpec, data::AtomicU64,
};

use crate::{
    RtTcb, Tcb,
    arch::*,
    current_proc_fd,
    protocol::{
        ProcCall, RtSigInfo, SIGCHLD, SIGCONT, SIGKILL, SIGSTOP, SIGTSTP, SIGTTIN, SIGTTOU, SIGURG,
        SIGWINCH, ThreadCall,
    },
    static_proc_info,
    sync::Mutex,
    sys::{proc_call, this_thread_call},
};

#[cfg(target_arch = "x86_64")]
static CPUID_EAX1_ECX: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

pub fn sighandler_function() -> usize {
    // TODO: HWCAP?

    __relibc_internal_sigentry as usize
}

/// ucontext_t representation
#[repr(C)]
pub struct SigStack {
    #[cfg(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64"
    ))]
    _pad: [usize; 1], // pad from 7*8 to 64

    #[cfg(target_arch = "x86")]
    _pad: [usize; 3], // pad from 9*4 to 12*4

    pub link: *mut SigStack,

    pub old_stack: PosixStackt,
    pub old_mask: u64,
    pub(crate) sival: usize,
    pub(crate) sig_code: u32,
    pub(crate) sig_num: u32,

    // x86_64: 864 bytes
    // i686: 512 bytes
    // aarch64: 272 bytes (SIMD TODO)
    // riscv64: 520 bytes (vector extensions TODO)
    pub regs: ArchIntRegs,
}
#[repr(C)]
pub struct PosixStackt {
    pub sp: *mut (),
    pub flags: i32,
    pub size: usize,
}

pub const SS_ONSTACK: usize = 1;
pub const SS_DISABLE: usize = 2;

impl From<Sigaltstack> for PosixStackt {
    fn from(value: Sigaltstack) -> Self {
        match value {
            Sigaltstack::Disabled => PosixStackt {
                sp: core::ptr::null_mut(),
                size: 0,
                flags: SS_DISABLE.try_into().unwrap(),
            },
            Sigaltstack::Enabled {
                onstack,
                base,
                size,
            } => PosixStackt {
                sp: base.cast(),
                size,
                flags: if onstack {
                    SS_ONSTACK.try_into().unwrap()
                } else {
                    0
                },
            },
        }
    }
}

#[repr(C)]
// TODO: This struct is for practical reasons locked to Linux's ABI, but avoid redefining
// it here. Alternatively, check at compile time that the structs are equivalent.
pub struct SiginfoAbi {
    pub si_signo: i32,
    pub si_errno: i32,
    pub si_code: i32,
    pub si_pid: i32,      // pid_t
    pub si_uid: i32,      // uid_t
    pub si_addr: *mut (), // *mut c_void
    pub si_status: i32,
    pub si_value: usize, // sigval
}

#[inline(always)]
unsafe fn inner(stack: &mut SigStack) {
    let os = unsafe { &Tcb::current().unwrap().os_specific };

    let stack_ptr = NonNull::from(&mut *stack);
    stack.link = core::mem::replace(
        unsafe { &mut (*os.arch.get()).last_sigstack },
        Some(stack_ptr),
    )
    .map_or_else(core::ptr::null_mut, |x| x.as_ptr());

    let signals_were_disabled = unsafe { (*os.arch.get()).disable_signals_depth > 0 };

    let targeted_thread_not_process = stack.sig_num >= 64;
    stack.sig_num %= 64;

    // asm counts from 0
    stack.sig_num += 1;

    let (sender_pid, sender_uid) = {
        let area = unsafe { &mut *os.arch.get() };

        // Undefined if the signal was not realtime
        stack.sival = area.tmp_rt_inf.arg;

        stack.old_stack = unsafe { arch_pre(stack, area) };

        if (stack.sig_num - 1) / 32 == 1 && !targeted_thread_not_process {
            stack.sig_code = area.tmp_rt_inf.code as u32;
            (area.tmp_rt_inf.pid, area.tmp_rt_inf.uid)
        } else {
            stack.sig_code = 0; // TODO: SI_USER constant?
            // TODO: Handle SIGCHLD. Maybe that should always be queued though?
            let inf = SenderInfo::from_raw(area.tmp_id_inf);
            (inf.pid, inf.ruid)
        }
    };

    let sigaction = {
        let guard = SIGACTIONS_LOCK.lock();
        let action = convert_old(&PROC_CONTROL_STRUCT.actions[stack.sig_num as usize - 1]);
        if action.flags.contains(SigactionFlags::RESETHAND) {
            drop(guard);
            // TODO: handle error?
            let _ = sigaction(
                stack.sig_num as u8,
                Some(&Sigaction {
                    kind: SigactionKind::Default,
                    mask: 0,
                    flags: SigactionFlags::empty(),
                }),
                None,
            );
        }
        action
    };
    let shall_restart = sigaction.flags.contains(SigactionFlags::RESTART);
    let sig = (stack.sig_num & 0x3f) as u8;

    let handler = match sigaction.kind {
        // TODO: Since sigaction may be called while procmgr is checking the IGNORED bit, it is
        // likely possible there can be a race condition resulting in the signal trampoline running
        // and reaching this code. If so, we do already know whether the signal is IGNORED *now*,
        // and so we should return early ideally without even temporarily touching the signal mask.
        SigactionKind::Ignore => {
            panic!("ctl {:#x?} signal {}", os.control, stack.sig_num)
        }
        // this case should be treated equally as the one above
        //
        // _ if sigaction.flags.contains(SigactionFlags::IGNORED) => {
        //     panic!("ctl2 {:#x?} signal {}", os.control, stack.sig_num)
        // }
        SigactionKind::Default if usize::from(sig) == SIGCONT => SignalHandler { handler: None },
        SigactionKind::Default => {
            let _ = proc_call(
                current_proc_fd().as_raw_fd(),
                &mut [],
                CallFlags::empty(),
                &[ProcCall::Exit as u64, u64::from(sig) << 8],
            );
            panic!()
        }
        SigactionKind::Handled { handler } => handler,
    };

    // Set sigmask to sa_mask and unmark the signal as pending.
    let prev_sigallow = get_allowset_raw(&os.control.word);

    let mut sigallow_inside = !sigaction.mask & prev_sigallow;
    if !sigaction.flags.contains(SigactionFlags::NODEFER) {
        sigallow_inside &= !sig_bit(stack.sig_num);
    }

    let _pending_when_sa_mask = set_allowset_raw(&os.control.word, prev_sigallow, sigallow_inside);

    // TODO: If sa_mask caused signals to be unblocked, deliver one or all of those first?

    // Re-enable signals again.
    let control_flags = &os.control.control_flags;
    control_flags.store(
        control_flags.load(Ordering::Relaxed) & !SigcontrolFlags::INHIBIT_DELIVERY.bits(),
        Ordering::Release,
    );
    core::sync::atomic::compiler_fence(Ordering::Acquire);

    stack.old_mask = prev_sigallow;

    // Call handler, either sa_handler or sa_siginfo depending on flag.
    if sigaction.flags.contains(SigactionFlags::SIGINFO)
        && let Some(sigaction) = unsafe { handler.sigaction }
    {
        let info = SiginfoAbi {
            si_signo: stack.sig_num as c_int,
            si_addr: core::ptr::null_mut(),
            si_code: stack.sig_code as i32,
            si_errno: 0,
            si_pid: sender_pid as i32,
            si_status: 0,
            si_uid: sender_uid as i32,
            si_value: stack.sival,
        };
        unsafe {
            sigaction(
                stack.sig_num as c_int,
                core::ptr::addr_of!(info).cast(),
                stack as *mut SigStack as *mut (),
            )
        };
    } else if let Some(handler) = unsafe { handler.handler } {
        handler(stack.sig_num as c_int);
    }

    // Disable signals while we modify the sigmask again
    control_flags.store(
        control_flags.load(Ordering::Relaxed) | SigcontrolFlags::INHIBIT_DELIVERY.bits(),
        Ordering::Release,
    );
    core::sync::atomic::compiler_fence(Ordering::Acquire);

    // Update allowset again, and obtain the set of pending unblocked signals with this new
    // allowset. If this is nonempty, we must deliver the signal here rather than wait for it
    // asynchronously, as shown e.g. in https://gitlab.redox-os.org/redox-os/relibc/-/issues/239.
    // We do this even if signals_were_disabled beforehand (where the caller hence explicitly
    // must have jumped to this trampoline).

    let new_mask = stack.old_mask;
    let old_mask = get_allowset_raw(&os.control.word);

    let pending_unblocked = {
        let thread_pending = set_allowset_raw(&os.control.word, old_mask, new_mask);
        let proc_pending = PROC_CONTROL_STRUCT.pending.load(Ordering::Relaxed);
        (thread_pending | proc_pending) & new_mask
    };

    unsafe { (*os.arch.get()).last_sig_was_restart = shall_restart };

    // TODO: Support setting uc_link to jump back to a different context?
    unsafe { (*os.arch.get()).last_sigstack = NonNull::new(stack.link) };

    // TODO: Support restoring uc_stack?

    if pending_unblocked > 0 {
        // If there were signals visible at the time we checked (otherwise it can be considered
        // "asynchronous" and will be delivered the next time an EINTR is seen, or after the
        // post-preemption check in the kernel), we delay clearing the INHIBIT_DELIVERY bit, and
        // rearrange the saved instr ptr field on the stack and the sc_saved_rip field, so that it
        // returns to the start of the trampoline in a way that makes it look like a new signal was
        // immediately delivered. There's no need to worry about the __crit_* sections since
        // INHIBIT_DELIVERY is still set.
        arch_ret_to_sig(stack, &os.control);
    } else if !signals_were_disabled {
        // Re-enable signals again, except in the case where they were disabled and the trampoline
        // was explicitly jumped to when handling EINTR.
        core::sync::atomic::compiler_fence(Ordering::Release);
        control_flags.store(
            control_flags.load(Ordering::Relaxed) & !SigcontrolFlags::INHIBIT_DELIVERY.bits(),
            Ordering::Relaxed,
        );
    }
}
#[cfg(not(target_arch = "x86"))]
pub(crate) unsafe extern "C" fn inner_c(stack: usize) {
    unsafe { inner(&mut *(stack as *mut SigStack)) }
}
#[cfg(target_arch = "x86")]
pub(crate) unsafe extern "fastcall" fn inner_fastcall(stack: usize) {
    unsafe { inner(&mut *(stack as *mut SigStack)) }
}

pub fn get_sigmask() -> Result<u64> {
    let mut mask = 0;
    modify_sigmask(Some(&mut mask), Option::<fn(u64) -> u64>::None)?;
    Ok(mask)
}
pub fn set_sigmask(new: Option<u64>, old: Option<&mut u64>) -> Result<()> {
    modify_sigmask(old, new.map(move |newmask| move |_| newmask))
}
pub fn or_sigmask(new: Option<u64>, old: Option<&mut u64>) -> Result<()> {
    // Parsing nightmare... :)
    modify_sigmask(
        old,
        new.map(move |newmask| move |oldmask| oldmask | newmask),
    )
}
pub fn andn_sigmask(new: Option<u64>, old: Option<&mut u64>) -> Result<()> {
    modify_sigmask(
        old,
        new.map(move |newmask| move |oldmask| oldmask & !newmask),
    )
}
fn get_allowset_raw(words: &[AtomicU64; 2]) -> u64 {
    (words[0].load(Ordering::Relaxed) >> 32) | ((words[1].load(Ordering::Relaxed) >> 32) << 32)
}
/// Sets mask from old to new, returning what was pending at the time.
fn set_allowset_raw(words: &[AtomicU64; 2], old: u64, new_raw: u64) -> u64 {
    // TODO: should these bits always be set, or never be set?
    let new = new_raw | ALLOWSET_ALWAYS;

    // This assumes *only this thread* can change the allowset. If this rule is broken, the use of
    // fetch_add will corrupt the words entirely. fetch_add is very efficient on x86, being
    // generated as LOCK XADD which is the fastest RMW instruction AFAIK.
    let prev_w0 = words[0].fetch_add(
        ((new & 0xffff_ffff) << 32).wrapping_sub((old & 0xffff_ffff) << 32),
        Ordering::Relaxed,
    ) & 0xffff_ffff;
    let prev_w1 = words[1].fetch_add(
        ((new >> 32) << 32).wrapping_sub((old >> 32) << 32),
        Ordering::Relaxed,
    ) & 0xffff_ffff;

    prev_w0 | (prev_w1 << 32)
}
const ALLOWSET_ALWAYS: u64 = sig_bit(SIGSTOP as u32) | sig_bit(SIGKILL as u32);
fn modify_sigmask(old: Option<&mut u64>, op: Option<impl FnOnce(u64) -> u64>) -> Result<()> {
    let _guard = tmp_disable_signals();
    let ctl = current_sigctl();

    let prev = get_allowset_raw(&ctl.word);

    if let Some(old) = old {
        *old = !prev;
    }
    let Some(op) = op else {
        return Ok(());
    };

    let next = !op(!prev);

    let thread_pending = set_allowset_raw(&ctl.word, prev, next);
    let proc_pending = PROC_CONTROL_STRUCT.pending.load(Ordering::Relaxed);

    // POSIX requires that at least one pending unblocked signal be delivered before
    // pthread_sigmask returns, if there is one.
    if (thread_pending | proc_pending) & next != 0 {
        unsafe {
            manually_enter_trampoline();
        }
    }

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
                SigactionKind::Handled { handler } => {
                    if self.flags.contains(SigactionFlags::SIGINFO) {
                        handler.sigaction.map_or(0, |a| a as usize)
                    } else {
                        handler.handler.map_or(0, |a| a as usize)
                    }
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
        SigactionKind::Handled {
            handler: unsafe { core::mem::transmute(handler as usize) },
        }
    };

    Sigaction {
        mask: old_mask,
        flags,
        kind,
    }
}

pub fn sigaction(signal: u8, new: Option<&Sigaction>, old: Option<&mut Sigaction>) -> Result<()> {
    let _sigguard = tmp_disable_signals();
    let ctl = current_sigctl();

    let _guard = SIGACTIONS_LOCK.lock();
    sigaction_inner(ctl, signal, new, old)
}
fn sigaction_inner(
    ctl: &Sigcontrol,
    signal: u8,
    new: Option<&Sigaction>,
    old: Option<&mut Sigaction>,
) -> Result<()> {
    // TODO: Now that the goal of keeping logic out of the IPC backend, no longer holds when
    // procmgr has taken over signal handling from the kernel, it would probably make sense to make
    // parts of this function an IPC call, for synchronization purposes. Apart from SA_RESETHAND
    // logic which may need to be fast, regular sigaction is typically in the 'configuration'
    // category, allowed to be slower.

    if matches!(usize::from(signal), 0 | 32 | 65..) {
        return Err(Error::new(EINVAL));
    }
    if matches!(usize::from(signal), SIGKILL | SIGSTOP) {
        if new.is_some() {
            return Err(Error::new(EINVAL));
        }
        if let Some(old) = old {
            // TODO: Is this the correct value to set it to?
            *old = Sigaction {
                kind: SigactionKind::Default,
                mask: 0,
                flags: SigactionFlags::empty(),
            };
        }
        return Ok(());
    }

    let action = &PROC_CONTROL_STRUCT.actions[usize::from(signal) - 1];

    if let Some(old) = old {
        *old = convert_old(action);
    }

    let Some(new) = new else {
        return Ok(());
    };

    let explicit_handler = new.ip();

    let sig_group = (signal - 1) / 32;
    let sig_idx = (signal - 1) % 32;

    let (mask, flags, handler) = match (usize::from(signal), new.kind) {
        (_, SigactionKind::Ignore) | (SIGURG | SIGWINCH, SigactionKind::Default) => {
            // TODO: relibc and procmgr have access to all threads, redox_rt doesn't currently.
            // Do this for all threads!
            ctl.word[usize::from(sig_group)].fetch_and(!(1 << sig_idx), Ordering::Relaxed);
            PROC_CONTROL_STRUCT
                .pending
                .fetch_and(!sig_bit(signal.into()), Ordering::Relaxed);

            // TODO: handle tmp_disable_signals
            (
                MASK_DONTCARE,
                SigactionFlags::IGNORED,
                if matches!(new.kind, SigactionKind::Default) {
                    default_handler as usize
                } else {
                    0
                },
            )
        }
        // TODO: Handle pending signals before these flags are set.
        (SIGTSTP | SIGTTOU | SIGTTIN, SigactionKind::Default) => (
            MASK_DONTCARE,
            SigactionFlags::SIG_SPECIFIC,
            default_handler as usize,
        ),
        (SIGCHLD, SigactionKind::Default) => {
            let nocldstop_bit = new.flags & SigactionFlags::SIG_SPECIFIC;

            // Default action is to ignore. Hence all pending SIGCHLD signals should be discarded.

            // TODO: relibc and procmgr have access to all threads, redox_rt doesn't currently.
            // Do this for all threads!
            ctl.word[usize::from(sig_group)].fetch_and(!(1 << sig_idx), Ordering::Relaxed);
            PROC_CONTROL_STRUCT
                .pending
                .fetch_and(!sig_bit(signal.into()), Ordering::Relaxed);

            (
                MASK_DONTCARE,
                SigactionFlags::IGNORED | nocldstop_bit,
                default_handler as usize,
            )
        }

        (_, SigactionKind::Default) => (new.mask, new.flags, default_handler as usize),
        (_, SigactionKind::Handled { .. }) => (new.mask, new.flags, explicit_handler),
    };
    let new_first = (handler as u64) | (u64::from(flags.bits() & STORED_FLAGS) << 32);
    action.first.store(new_first, Ordering::Relaxed);
    action.user_data.store(mask, Ordering::Relaxed);

    Ok(())
}

fn current_sigctl() -> &'static Sigcontrol {
    &unsafe { Tcb::current() }.unwrap().os_specific.control
}

pub struct TmpDisableSignalsGuard {
    _inner: (),
}

pub fn tmp_disable_signals() -> TmpDisableSignalsGuard {
    unsafe {
        let ctl = &current_sigctl().control_flags;
        ctl.store(
            ctl.load(Ordering::Relaxed) | syscall::flag::INHIBIT_DELIVERY.bits(),
            Ordering::Release,
        );
        core::sync::atomic::compiler_fence(Ordering::Acquire);

        // TODO: fence?
        (*Tcb::current().unwrap().os_specific.arch.get()).disable_signals_depth += 1;
    }

    TmpDisableSignalsGuard { _inner: () }
}
impl Drop for TmpDisableSignalsGuard {
    fn drop(&mut self) {
        unsafe {
            let depth =
                &mut (*Tcb::current().unwrap().os_specific.arch.get()).disable_signals_depth;
            *depth -= 1;

            if *depth == 0 {
                let ctl = &current_sigctl().control_flags;
                ctl.store(
                    ctl.load(Ordering::Relaxed) & !syscall::flag::INHIBIT_DELIVERY.bits(),
                    Ordering::Release,
                );
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

fn default_handler(_sig: c_int) {
    unreachable!();
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
    sender_infos: [const { AtomicU64::new(0) }; 32],
};

const fn sig_bit(sig: u32) -> u64 {
    //assert_ne!(sig, 32);
    //assert_ne!(sig, 0);
    1 << (sig - 1)
}

pub fn setup_sighandler(tcb: &RtTcb, first_thread: bool) {
    if first_thread {
        let _guard = SIGACTIONS_LOCK.lock();
        for (sig_idx, action) in PROC_CONTROL_STRUCT.actions.iter().enumerate() {
            let sig = sig_idx + 1;
            let bits = if matches!(sig, SIGTSTP | SIGTTIN | SIGTTOU) {
                SigactionFlags::SIG_SPECIFIC
            } else if matches!(sig, SIGCHLD | SIGURG | SIGWINCH) {
                SigactionFlags::IGNORED
            } else {
                SigactionFlags::empty()
            };
            action.first.store(
                (u64::from(bits.bits()) << 32) | default_handler as u64,
                Ordering::Relaxed,
            );
        }
    }
    let arch = unsafe { &mut *tcb.arch.get() };
    {
        // The asm decides whether to use the altstack, based on whether the saved stack pointer
        // was already on that stack. Thus, setting the altstack to the entire address space, is
        // equivalent to not using any altstack at all (the default).
        arch.altstack_top = usize::MAX;
        arch.altstack_bottom = 0;
        #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
        {
            arch.pctl = core::ptr::addr_of!(PROC_CONTROL_STRUCT) as usize;
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        let cpuid_eax1_ecx = unsafe { core::arch::x86_64::__cpuid(1) }.ecx;
        CPUID_EAX1_ECX.store(cpuid_eax1_ecx, core::sync::atomic::Ordering::Relaxed);
        SUPPORTS_AVX.store(u8::from(cpuid_eax1_ecx & 1 << 28 != 0), Ordering::Relaxed);
    }

    let data = current_setsighandler_struct();

    let fd = tcb
        .thread_fd()
        .dup(b"sighandler")
        .expect("failed to open sighandler fd");
    fd.write(&data).expect("failed to write to sighandler fd");
    this_thread_call(
        &mut [],
        CallFlags::empty(),
        &[ThreadCall::SyncSigTctl as u64],
    )
    .expect("failed to sync signal tctl");

    // TODO: Inherited set of ignored signals
    // TODO: handle error
    let _ = set_sigmask(Some(0), None);
}
pub type RtSigarea = RtTcb; // TODO
pub fn current_setsighandler_struct() -> SetSighandlerData {
    SetSighandlerData {
        user_handler: sighandler_function(),
        excp_handler: 0, // TODO
        thread_control_addr: core::ptr::addr_of!(
            unsafe { Tcb::current() }.unwrap().os_specific.control
        ) as usize,
        proc_control_addr: &PROC_CONTROL_STRUCT as *const SigProcControl as usize,
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum Sigaltstack {
    #[default]
    Disabled,

    Enabled {
        onstack: bool,
        base: *mut (),
        size: usize,
    },
}

pub(crate) fn get_sigaltstack(tcb: &SigArea, sp: usize) -> Sigaltstack {
    if tcb.altstack_bottom == 0 && tcb.altstack_top == usize::MAX {
        Sigaltstack::Disabled
    } else {
        Sigaltstack::Enabled {
            base: tcb.altstack_bottom as *mut (),
            size: tcb.altstack_top - tcb.altstack_bottom,
            onstack: (tcb.altstack_bottom..=tcb.altstack_top).contains(&sp),
        }
    }
}

pub unsafe fn sigaltstack(
    new: Option<&Sigaltstack>,
    old_out: Option<&mut Sigaltstack>,
) -> Result<()> {
    let _g = tmp_disable_signals();
    let tcb = unsafe { &mut *Tcb::current().unwrap().os_specific.arch.get() };

    let old = get_sigaltstack(tcb, crate::arch::current_sp());

    if matches!(old, Sigaltstack::Enabled { onstack: true, .. })
        && new.is_some_and(|new| *new != old)
    {
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
            Sigaltstack::Enabled {
                base,
                size,
                onstack: false,
            } => {
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

pub const MIN_SIGALTSTACK_SIZE: usize = 2048;

pub fn currently_pending_blocked() -> u64 {
    let control = &unsafe { Tcb::current().unwrap() }.os_specific.control;
    let w0 = control.word[0].load(Ordering::Relaxed);
    let w1 = control.word[1].load(Ordering::Relaxed);
    let allow = (w0 >> 32) | ((w1 >> 32) << 32);
    let thread_pending = (w0 & 0xffff_ffff) | ((w1 & 0xffff_ffff) << 32);
    let proc_pending = PROC_CONTROL_STRUCT.pending.load(Ordering::Relaxed);

    core::sync::atomic::fence(Ordering::Acquire); // TODO: Correct ordering?

    (thread_pending | proc_pending) & !allow
}
pub enum Unreachable {}

pub fn await_signal_async(inner_allowset: u64) -> Result<Unreachable> {
    let _guard = tmp_disable_signals();
    let control = &unsafe { Tcb::current().unwrap() }.os_specific.control;

    let old_allowset = get_allowset_raw(&control.word);
    set_allowset_raw(&control.word, old_allowset, inner_allowset);

    let res = syscall::nanosleep(
        &TimeSpec {
            tv_sec: i64::MAX,
            tv_nsec: 0,
        },
        &mut TimeSpec::default(),
    );

    if res == Err(Error::new(EINTR)) {
        unsafe {
            manually_enter_trampoline();
        }
    }
    // POSIX says it shall restore the mask to what it was prior to the call, which is interpreted
    // as allowing any changes to sigprocmask inside the signal handler, to be discarded.
    set_allowset_raw(&control.word, inner_allowset, old_allowset);

    res?;
    unreachable!()
}

/// Run a callback with the specified signal mask, atomically
pub fn callback_or_signal_async<T, F: FnOnce() -> Result<T>>(
    inner_allowset: u64,
    callback: F,
) -> Result<T> {
    let _guard = tmp_disable_signals();
    let control = &unsafe { Tcb::current().unwrap() }.os_specific.control;

    let old_allowset = get_allowset_raw(&control.word);
    set_allowset_raw(&control.word, old_allowset, inner_allowset);
    let res = callback();
    if let Err(err) = &res {
        if err.errno == EINTR {
            // Run trampoline if EINTR returned
            unsafe {
                manually_enter_trampoline();
            }
        }
    }

    // POSIX says it shall restore the mask to what it was prior to the call, which is interpreted
    // as allowing any changes to sigprocmask inside the signal handler, to be discarded.
    set_allowset_raw(&control.word, inner_allowset, old_allowset);

    res
}

/*#[unsafe(no_mangle)]
pub extern "C" fn __redox_rt_debug_sigctl() {
    let tcb = &RtTcb::current().control;
    let _ = syscall::write(1, alloc::format!("SIGCTL: {tcb:#x?}\n").as_bytes());
}*/

// TODO: deadline-based API
pub fn await_signal_sync(inner_allowset: u64, timeout: Option<&TimeSpec>) -> Result<SiginfoAbi> {
    let _guard = tmp_disable_signals();
    let control = &unsafe { Tcb::current().unwrap() }.os_specific.control;

    let old_allowset = get_allowset_raw(&control.word);
    let proc_pending = PROC_CONTROL_STRUCT.pending.load(Ordering::Acquire);
    let thread_pending = set_allowset_raw(&control.word, old_allowset, inner_allowset);

    // Check if there are already signals matching the requested set, before waiting.
    if let Some(info) = try_claim_multiple(proc_pending, thread_pending, inner_allowset, control) {
        // TODO: RAII
        set_allowset_raw(&control.word, inner_allowset, old_allowset);
        return Ok(info);
    }

    let res = match timeout {
        Some(t) => syscall::nanosleep(&t, &mut TimeSpec::default()),
        None => syscall::nanosleep(
            &TimeSpec {
                tv_sec: i64::MAX,
                tv_nsec: 0,
            },
            &mut TimeSpec::default(),
        ),
    };

    let thread_pending = set_allowset_raw(&control.word, inner_allowset, old_allowset);
    let proc_pending = PROC_CONTROL_STRUCT.pending.load(Ordering::Acquire);

    if let Err(error) = res
        && error.errno != EINTR
    {
        return Err(error);
    }

    // Then check if there were any signals left after waiting.
    try_claim_multiple(proc_pending, thread_pending, inner_allowset, control)
        // Normally ETIMEDOUT but not for sigtimedwait.
        .ok_or(Error::new(EAGAIN))
}

fn try_claim_multiple(
    mut proc_pending: u64,
    mut thread_pending: u64,
    allowset: u64,
    control: &Sigcontrol,
) -> Option<SiginfoAbi> {
    while (proc_pending | thread_pending) & allowset != 0 {
        let sig_idx = ((proc_pending | thread_pending) & allowset).trailing_zeros();
        if thread_pending & allowset & (1 << sig_idx) != 0
            && let Some(res) = try_claim_single(sig_idx, Some(control))
        {
            return Some(res);
        }
        thread_pending &= !(1 << sig_idx);
        if proc_pending & allowset & (1 << sig_idx) != 0
            && let Some(res) = try_claim_single(sig_idx, None)
        {
            return Some(res);
        }
        proc_pending &= !(1 << sig_idx);
    }
    None
}
fn try_claim_single(sig_idx: u32, thread_control: Option<&Sigcontrol>) -> Option<SiginfoAbi> {
    let sig_group = sig_idx / 32;

    if sig_group == 1 && thread_control.is_none() {
        // Queued (realtime) signal
        let rt_inf: RtSigInfo = unsafe {
            let mut buf = [0_u8; size_of::<RtSigInfo>()];
            buf[..4].copy_from_slice(&(sig_idx - 32).to_ne_bytes());
            proc_call(
                static_proc_info().proc_fd.as_ref().unwrap().as_raw_fd(),
                &mut buf,
                CallFlags::empty(),
                &[ProcCall::Sigdeq as u64],
            )
            .ok()?;
            core::mem::transmute(buf)
        };
        Some(SiginfoAbi {
            si_signo: sig_idx as i32 + 1,
            si_errno: 0,
            si_code: rt_inf.code,
            si_pid: rt_inf.pid as i32,
            si_uid: rt_inf.uid as i32,
            si_status: 0,
            si_value: rt_inf.arg,
            si_addr: core::ptr::null_mut(),
        })
    } else {
        // Idempotent (standard or thread realtime) signal
        let info = SenderInfo::from_raw(match thread_control {
            Some(ctl) => {
                // Only this thread can clear pending bits, so this will always succeed.
                let info = match ctl.sender_infos.get(sig_idx as usize) {
                    Some(i) => i.load(Ordering::Relaxed),
                    // TODO: Protocol will need to be extended in order to allow passing si_uid and
                    // si_pid for per-thread (non-queued) realtime signals.
                    None => 0,
                };
                // TODO: Ordering?
                ctl.word[sig_group as usize].fetch_and(!(1 << (sig_idx % 32)), Ordering::Release);
                info
            }
            None => {
                // Must have sig_group == 0 here due to the above if stmt
                let info =
                    PROC_CONTROL_STRUCT.sender_infos[sig_idx as usize].load(Ordering::Acquire);
                if PROC_CONTROL_STRUCT
                    .pending
                    .fetch_and(!(1 << sig_idx), Ordering::Release)
                    & (1 << sig_idx)
                    == 0
                {
                    // already claimed
                    return None;
                }
                info
            }
        });
        Some(SiginfoAbi {
            si_signo: sig_idx as i32 + 1,
            si_errno: 0,
            si_code: 0, // TODO: SI_USER const?
            si_pid: info.pid as i32,
            si_uid: info.ruid as i32,
            si_status: 0,
            si_value: 0, // undefined
            si_addr: core::ptr::null_mut(),
        })
    }
}
pub fn apply_inherited_sigignmask(inherited: u64) {
    let _sig_guard = tmp_disable_signals();
    let _guard = SIGACTIONS_LOCK.lock();
    let ctl = current_sigctl();

    // Set all signals in the inherited set that have explicitly been set to SIG_IGN. Those whose
    // (default) effective action is to ignore but are set to SIG_DFL, are not in this set, and the
    // initial SIG_DFL state would be "Ignore" anyway but still return SIG_DFL when asked, as
    // usual.
    for bit in (0..64).filter(|b| inherited & (1 << b) != 0) {
        let sig = u8::try_from(bit + 1).unwrap();
        let _ = sigaction_inner(
            ctl,
            sig,
            Some(&Sigaction {
                // TODO: correct fields?
                flags: SigactionFlags::IGNORED,
                kind: SigactionKind::Ignore,
                mask: 0,
            }),
            None,
        );
    }
}
pub fn get_sigignmask_to_inherit() -> u64 {
    let _sig_guard = tmp_disable_signals();
    let _guard = SIGACTIONS_LOCK.lock();
    let ctl = current_sigctl();

    let mut mask = 0_u64;
    // Fill the mask with the set of the inherited signals that have explicitly been set to
    // SIG_IGN. Again this excludes signals that would have returned SIG_DFL from sigaction,
    // despite their default action being "Ignore".

    for bit in 0..64 {
        let sig = u8::try_from(bit + 1).unwrap();
        let mut old = Sigaction::default();
        let _ = sigaction_inner(ctl, sig, None, Some(&mut old));
        if matches!(old.kind, SigactionKind::Ignore) {
            mask |= 1 << bit;
        }
    }

    mask
}
