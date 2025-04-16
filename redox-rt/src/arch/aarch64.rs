use core::{cell::SyncUnsafeCell, mem::offset_of, ptr::NonNull};

use syscall::{data::*, error::*};

use crate::{
    proc::{fork_inner, FdGuard, ForkArgs},
    protocol::{ProcCall, RtSigInfo},
    signal::{inner_c, PosixStackt, RtSigarea, SigStack, PROC_CONTROL_STRUCT},
    RtTcb, Tcb,
};

// Setup a stack starting from the very end of the address space, and then growing downwards.
pub(crate) const STACK_TOP: usize = 1 << 47;
pub(crate) const STACK_SIZE: usize = 1024 * 1024;

#[derive(Debug, Default)]
#[repr(C)]
pub struct SigArea {
    pub altstack_top: usize,
    pub altstack_bottom: usize,
    pub tmp_x1_x2: [usize; 2],
    pub tmp_x3_x4: [usize; 2],
    pub tmp_x5_x6: [usize; 2],
    pub tmp_x7_x8: [usize; 2],
    pub tmp_sp: usize,
    pub onstack: u64,
    pub disable_signals_depth: u64,
    pub pctl: usize, // TODO: remove
    pub last_sig_was_restart: bool,
    pub last_sigstack: Option<NonNull<SigStack>>,
    pub tmp_rt_inf: RtSigInfo,
    pub tmp_id_inf: u64,
}
#[repr(C)]
#[derive(Debug, Default)]
pub struct ArchIntRegs {
    pub x30: usize,
    pub x29: usize,
    pub x28: usize,
    pub x27: usize,
    pub x26: usize,
    pub x25: usize,
    pub x24: usize,
    pub x23: usize,
    pub x22: usize,
    pub x21: usize,
    pub x20: usize,
    pub x19: usize,
    pub x18: usize,
    pub x17: usize,
    pub x16: usize,
    pub x15: usize,
    pub x14: usize,
    pub x13: usize,
    pub x12: usize,
    pub x11: usize,
    pub x10: usize,
    pub x9: usize,
    pub x8: usize,
    pub x7: usize,
    pub x6: usize,
    pub x5: usize,
    pub x4: usize,
    pub x3: usize,
    pub x2: usize,
    pub x1: usize,

    pub sp: usize,
    pub nzcv: usize, // user-accessible PSTATE bits

    pub pc: usize,
    pub x0: usize,
}

/// Deactive TLS, used before exec() on Redox to not trick target executable into thinking TLS
/// is already initialized as if it was a thread.
pub unsafe fn deactivate_tcb(open_via_dup: usize) -> Result<()> {
    let mut env = syscall::EnvRegisters::default();

    let file = FdGuard::new(syscall::dup(open_via_dup, b"regs/env")?);

    env.tpidr_el0 = 0;

    let _ = syscall::write(*file, &mut env)?;
    Ok(())
}

pub fn copy_env_regs(cur_pid_fd: usize, new_pid_fd: usize) -> Result<()> {
    // Copy environment registers.
    {
        let cur_env_regs_fd = FdGuard::new(syscall::dup(cur_pid_fd, b"regs/env")?);
        let new_env_regs_fd = FdGuard::new(syscall::dup(new_pid_fd, b"regs/env")?);

        let mut env_regs = syscall::EnvRegisters::default();
        let _ = syscall::read(*cur_env_regs_fd, &mut env_regs)?;
        let _ = syscall::write(*new_env_regs_fd, &env_regs)?;
    }

    Ok(())
}

unsafe extern "C" fn fork_impl(args: &ForkArgs, initial_rsp: *mut usize) -> usize {
    Error::mux(fork_inner(initial_rsp, args))
}

unsafe extern "C" fn child_hook(cur_filetable_fd: usize, new_proc_fd: usize, new_thr_fd: usize) {
    //let _ = syscall::write(1, alloc::format!("A{cur_filetable_fd}B{new_proc_fd}C{new_thr_fd}\n").as_bytes());
    let _ = syscall::close(cur_filetable_fd);
    crate::child_hook_common(crate::ChildHookCommonArgs {
        new_thr_fd: FdGuard::new(new_thr_fd),
        new_proc_fd: if new_proc_fd == usize::MAX {
            None
        } else {
            Some(FdGuard::new(new_proc_fd))
        },
    });
}

asmfunction!(__relibc_internal_fork_wrapper (usize) -> usize: ["
    stp     x29, x30, [sp, #-16]!
    stp     x27, x28, [sp, #-16]!
    stp     x25, x26, [sp, #-16]!
    stp     x23, x24, [sp, #-16]!
    stp     x21, x22, [sp, #-16]!
    stp     x19, x20, [sp, #-16]!

    sub sp, sp, #32

    //TODO: store floating point regs

    // x0: &ForkArgs
    mov x1, sp
    bl {fork_impl}

    add sp, sp, #32
    ldp     x19, x20, [sp], #16
    ldp     x21, x22, [sp], #16
    ldp     x23, x24, [sp], #16
    ldp     x25, x26, [sp], #16
    ldp     x27, x28, [sp], #16
    ldp     x29, x30, [sp], #16
    ret
"] <= [fork_impl = sym fork_impl]);

asmfunction!(__relibc_internal_fork_ret: ["
    ldp x0, x1, [sp], #16
    ldp x2, x3, [sp], #16
    bl {child_hook}

    //TODO: load floating point regs

    mov x0, xzr

    ldp     x19, x20, [sp], #16
    ldp     x21, x22, [sp], #16
    ldp     x23, x24, [sp], #16
    ldp     x25, x26, [sp], #16
    ldp     x27, x28, [sp], #16
    ldp     x29, x30, [sp], #16

    ret
"] <= [child_hook = sym child_hook]);

// https://devblogs.microsoft.com/oldnewthing/20220811-00/?p=106963
asmfunction!(__relibc_internal_sigentry: ["
    // Clear any active reservation.
    clrex

    // The old pc and x0 are saved in the sigcontrol struct.
    mrs x0, tpidr_el0 // ABI ptr
    ldr x0, [x0] // TCB ptr

    // Save x1-x6 and sp
    stp x1, x2, [x0, #{tcb_sa_off} + {sa_tmp_x1_x2}]
    stp x3, x4, [x0, #{tcb_sa_off} + {sa_tmp_x3_x4}]
    stp x5, x6, [x0, #{tcb_sa_off} + {sa_tmp_x5_x6}]
    stp x7, x8, [x0, #{tcb_sa_off} + {sa_tmp_x7_x8}]
    mov x1, sp
    str x1, [x0, #{tcb_sa_off} + {sa_tmp_sp}]

    ldr x6, [x0, #{tcb_sa_off} + {sa_pctl}]
1:
    // Load x1 with the thread's bits
    add x5, x0, #{tcb_sc_off} + {sc_word}
    ldaxr x1, [x5]

    // First check if there are standard thread signals,
    and x4, x1, x1, lsr #32 // x4 := x1 & (x1 >> 32)
    cbnz x4, 3f // jump if x4 != 0
    clrex

    // and if not, load process pending bitset.
    add x5, x6, #{pctl_pending}
    ldaxr x2, [x5]

    // Check if there are standard proc signals:
    lsr x3, x1, #32 // mask
    and w3, w2, w3 // pending unblocked proc
    cbz w3, 4f // skip 'fetch_andn' step if zero

    // If there was one, find which one, and try clearing the bit (last value in x3, addr in x6)
    // this picks the MSB rather than the LSB, unlike x86. POSIX does not require any specific
    // ordering though.
    clz w3, w3
    mov w4, #31
    sub w3, w4, w3
    // x3 now contains the sig_idx

    mov x4, #1
    lsl x4, x4, x3 // bit to remove

    sub x4, x2, x4 // bit was certainly set, so sub is allowed
    // x4 is now the new mask to be set
    add x5, x6, #{pctl_pending}

    add x2, x5, #{pctl_sender_infos}
    add x2, x2, w3, uxtb 3
    ldar x2, [x2]

    // Try clearing the bit, retrying on failure.
    stxr w1, x4, [x5] // try setting pending set to x4, set w1 := 0 on success
    cbnz w1, 1b // retry everything if this fails
    mov x1, x3
    b 2f
4:
    // Check for realtime signals, thread/proc.
    clrex

    // Load the pending set again. TODO: optimize this?
    add x1, x6, #{pctl_pending}
    ldaxr x2, [x1]
    lsr x2, x2, #32

    add x5, x0, #{tcb_sc_off} + {sc_word} + 8
    ldar x1, [x5]

    orr x2, x1, x2
    and x2, x2, x2, lsr #32
    cbz x2, 7f

    rbit x3, x2
    clz x3, x3
    mov x4, #31
    sub x2, x4, x3
    // x2 now contains sig_idx - 32

    // If realtime signal was directed at thread, handle it as an idempotent signal.
    lsr x3, x1, x2
    tbnz x3, #0, 5f

    // SYS_CALL(fd, payload_base, payload_len, metadata_len, metadata_base | (flags << 8))
    // x8       x0  x1            x2           x3            x4

    mov x5, x0 // save TCB pointer
    ldr x8, ={SYS_CALL}
    adrp x0, {proc_fd}
    ldr x0, [x0, #:lo12:{proc_fd}]
    add x1, x5, #{tcb_sa_off} + {sa_tmp_rt_inf}
    str x2, [x1]
    mov x2, #{RTINF_SIZE}
    adrp x4, {proc_call}
    add x4, x4, :lo12:{proc_call}
    mov x3, #1
    svc 0
    mov x0, x5 // restore TCB pointer
    cbnz x0, 1b

    b 2f
5:
    // A realtime signal was sent to this thread, try clearing its bit.
    // x3 contains last rt signal word, x2 contains rt_idx
    clrex

    // Calculate the absolute sig_idx
    add x1, x3, 32

    // Load si_pid and si_uid
    add x2, x0, #{tcb_sc_off} + {sc_sender_infos}
    add x2, x2, w1, uxtb #3
    ldar x2, [x2]

    add x3, x0, #{tcb_sc_off} + {sc_word} + 8
    ldxr x2, [x3]

    // Calculate new mask
    mov x4, #1
    lsl x4, x4, x2
    sub x2, x2, x4 // remove bit

    stxr w5, x2, [x3]
    cbnz w5, 1b
    str x2, [x0, #{tcb_sa_off} + {sa_tmp_id_inf}]
    b 2f
3:
    // A standard signal was sent to this thread, try clearing its bit.
    clz x1, x1
    mov x2, #31
    sub x1, x2, x1

    // Load si_pid and si_uid
    add x2, x0, #{tcb_sc_off} + {sc_sender_infos}
    add x2, x2, w1, uxtb #3
    ldar x2, [x2]

    // Clear bit from mask
    mov x3, #1
    lsl x3, x3, x1
    sub x4, x4, x3

    // Try updating the mask
    stxr w3, x1, [x5]
    cbnz w3, 1b

    str x2, [x0, #{tcb_sa_off} + {sa_tmp_id_inf}]
2:
    ldr x3, [x0, #{tcb_sa_off} + {sa_pctl}]
    add x2, x2, {pctl_actions}
    add x2, x3, w1, uxtb #4 // actions_base + sig_idx * sizeof Action
    // TODO: NOT ATOMIC (tearing allowed between regs)!
    ldxp x2, x3, [x2]
    clrex

    // Calculate new sp wrt redzone and alignment
    mov x4, sp
    sub x4, x4, {REDZONE_SIZE}
    and x4, x4, -{STACK_ALIGN}
    mov sp, x4

    // skip sigaltstack step if SA_ONSTACK is clear
    // tbz x2, #{SA_ONSTACK_BIT}, 2f

    ldr x2, [x0, #{tcb_sc_off} + {sc_saved_pc}]
    ldr x3, [x0, #{tcb_sc_off} + {sc_saved_x0}]
    stp x2, x3, [sp, #-16]!

    ldr x2, [x0, #{tcb_sa_off} + {sa_tmp_sp}]
    mrs x3, nzcv
    stp x2, x3, [sp, #-16]!

    ldp x2, x3, [x0, #{tcb_sa_off} + {sa_tmp_x1_x2}]
    stp x2, x3, [sp, #-16]!
    ldp x3, x4, [x0, #{tcb_sa_off} + {sa_tmp_x3_x4}]
    stp x4, x3, [sp, #-16]!
    ldp x5, x6, [x0, #{tcb_sa_off} + {sa_tmp_x5_x6}]
    stp x6, x5, [sp, #-16]!
    ldp x7, x8, [x0, #{tcb_sa_off} + {sa_tmp_x7_x8}]
    stp x8, x7, [sp, #-16]!

    stp x10, x9, [sp, #-16]!
    stp x12, x11, [sp, #-16]!
    stp x14, x13, [sp, #-16]!
    stp x16, x15, [sp, #-16]!
    stp x18, x17, [sp, #-16]!
    stp x20, x19, [sp, #-16]!
    stp x22, x21, [sp, #-16]!
    stp x24, x23, [sp, #-16]!
    stp x26, x25, [sp, #-16]!
    stp x28, x27, [sp, #-16]!
    stp x30, x29, [sp, #-16]!

    str w1, [sp, #-4]
    sub sp, sp, #64

    mov x0, sp
    bl {inner}

    add sp, sp, #64

    ldp x30, x29, [sp], #16
    ldp x28, x27, [sp], #16
    ldp x26, x25, [sp], #16
    ldp x24, x23, [sp], #16
    ldp x22, x21, [sp], #16
    ldp x20, x19, [sp], #16
    ldp x18, x17, [sp], #16
    ldp x16, x15, [sp], #16
    ldp x14, x13, [sp], #16
    ldp x12, x11, [sp], #16
    ldp x10, x9, [sp], #16
    ldp x8, x7, [sp], #16
    ldp x6, x5, [sp], #16
    ldp x4, x3, [sp], #16
    ldp x2, x1, [sp], #16

    ldr x0, [sp, #8]
    msr nzcv, x0

8:
    // x18 is reserved by ABI as 'platform register', so clobbering it should be safe.
    mov x18, sp
    ldr x0, [x18]
    mov sp, x0

    ldp x18, x0, [x18, #16]
    br x18
7:
    // Spurious signal, i.e. all bitsets were 0 at the time they were checked
    clrex

    ldr x1, [x0, #{tcb_sc_off} + {sc_flags}]
    and x1, x1, ~1
    str x1, [x0, #{tcb_sc_off} + {sc_flags}]

    ldp x1, x2, [x0, #{tcb_sa_off} + {sa_tmp_x1_x2}]
    ldp x3, x4, [x0, #{tcb_sa_off} + {sa_tmp_x3_x4}]
    ldp x5, x6, [x0, #{tcb_sa_off} + {sa_tmp_x5_x6}]
    ldr x18, [x0, #{tcb_sc_off} + {sc_saved_pc}]
    ldr x0, [x0, #{tcb_sc_off} + {sc_saved_x0}]
    br x18
"] <= [
    pctl_pending = const (offset_of!(SigProcControl, pending)),
    pctl_actions = const (offset_of!(SigProcControl, actions)),
    pctl_sender_infos = const (offset_of!(SigProcControl, sender_infos)),
    tcb_sc_off = const (offset_of!(crate::Tcb, os_specific) + offset_of!(RtSigarea, control)),
    tcb_sa_off = const (offset_of!(crate::Tcb, os_specific) + offset_of!(RtSigarea, arch)),
    sa_tmp_x1_x2 = const offset_of!(SigArea, tmp_x1_x2),
    sa_tmp_x3_x4 = const offset_of!(SigArea, tmp_x3_x4),
    sa_tmp_x5_x6 = const offset_of!(SigArea, tmp_x5_x6),
    sa_tmp_x7_x8 = const offset_of!(SigArea, tmp_x7_x8),
    sa_tmp_sp = const offset_of!(SigArea, tmp_sp),
    sa_tmp_rt_inf = const offset_of!(SigArea, tmp_rt_inf),
    sa_tmp_id_inf = const offset_of!(SigArea, tmp_id_inf),
    sa_pctl = const offset_of!(SigArea, pctl),
    sc_saved_pc = const offset_of!(Sigcontrol, saved_ip),
    sc_saved_x0 = const offset_of!(Sigcontrol, saved_archdep_reg),
    sc_sender_infos = const offset_of!(Sigcontrol, sender_infos),
    sc_word = const offset_of!(Sigcontrol, word),
    sc_flags = const offset_of!(Sigcontrol, control_flags),
    proc_fd = sym PROC_FD,
    inner = sym inner_c,
    proc_call = sym PROC_CALL,

    SA_ONSTACK_BIT = const 58, // (1 << 58) >> 32 = 0x0400_0000

    SYS_CALL = const syscall::SYS_CALL,

    STACK_ALIGN = const 16,
    REDZONE_SIZE = const 128,
    RTINF_SIZE = const size_of::<RtSigInfo>(),
]);

asmfunction!(__relibc_internal_rlct_clone_ret: ["
    # Load registers
    ldp x8, x0, [sp], #16
    ldp x1, x2, [sp], #16
    ldp x3, x4, [sp], #16

    # Call entry point
    blr x8

    ret
"] <= []);

pub fn current_sp() -> usize {
    let sp: usize;
    unsafe {
        core::arch::asm!("mov {}, sp", out(reg) sp);
    }
    sp
}

pub unsafe fn manually_enter_trampoline() {
    let ctl = &Tcb::current().unwrap().os_specific.control;

    ctl.saved_archdep_reg.set(0);
    let ip_location = &ctl.saved_ip as *const _ as usize;

    core::arch::asm!("
        bl 2f
        b 3f
    2:
        str lr, [x0]
        b __relibc_internal_sigentry
    3:
    ", inout("x0") ip_location => _, out("lr") _);
}

pub unsafe fn arch_pre(stack: &mut SigStack, os: &mut SigArea) -> PosixStackt {
    PosixStackt {
        sp: core::ptr::null_mut(), // TODO
        size: 0,                   // TODO
        flags: 0,                  // TODO
    }
}
pub(crate) static PROC_FD: SyncUnsafeCell<usize> = SyncUnsafeCell::new(usize::MAX);
static PROC_CALL: [usize; 1] = [ProcCall::Sigdeq as usize];
