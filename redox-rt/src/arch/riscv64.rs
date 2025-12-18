use core::cell::SyncUnsafeCell;

use crate::{
    RtTcb, Tcb,
    proc::{FdGuard, FdGuardUpper, ForkArgs, fork_inner},
    protocol::{ProcCall, RtSigInfo},
    signal::{PosixStackt, RtSigarea, SigStack, get_sigaltstack, inner_c},
};
use core::{mem::offset_of, ptr::NonNull, sync::atomic::Ordering};
use syscall::{data::*, error::*};

use super::ForkScratchpad;

// Setup a stack starting from the very end of the address space, and then growing downwards.
pub const STACK_TOP: usize = 1 << 47;
pub const STACK_SIZE: usize = 1024 * 1024;

#[derive(Debug, Default)]
#[repr(C)]
pub struct SigArea {
    pub tmp_sp: u64,
    pub tmp_t1: u64,
    pub tmp_t2: u64,
    pub tmp_t3: u64,
    pub tmp_t4: u64,
    pub tmp_a0: u64,
    pub tmp_a1: u64,
    pub tmp_a2: u64,
    pub tmp_a3: u64,
    pub tmp_a4: u64,
    pub tmp_a7: u64,

    pub pctl: usize, // TODO: remove
    pub tmp_ip: u64,
    pub tmp_rt_inf: RtSigInfo,
    pub tmp_id_inf: u64,
    pub altstack_top: usize,
    pub altstack_bottom: usize,
    pub disable_signals_depth: u64,
    pub last_sig_was_restart: bool,
    pub last_sigstack: Option<NonNull<SigStack>>,
}
#[repr(C)]
#[derive(Debug, Default)]
pub struct ArchIntRegs {
    pub int_regs: [u64; 31],
    pub pc: u64,
    pub fp_regs: [u64; 32],
    pub fcsr: u32,
    _pad: u32,
}

/// Deactive TLS, used before exec() on Redox to not trick target executable into thinking TLS
/// is already initialized as if it was a thread.
pub unsafe fn deactivate_tcb(open_via_dup: &FdGuardUpper) -> Result<()> {
    let mut env = syscall::EnvRegisters::default();

    let file = open_via_dup.dup(b"regs/env")?;

    env.tp = 0;

    file.write(&mut env)?;
    Ok(())
}

unsafe extern "C" fn fork_impl(args: &ForkArgs, initial_rsp: *mut usize) -> usize {
    Error::mux(fork_inner(initial_rsp, args))
}

unsafe extern "C" fn child_hook(scratchpad: &ForkScratchpad) {
    let _ = syscall::close(scratchpad.cur_filetable_fd);
    crate::child_hook_common(crate::ChildHookCommonArgs {
        new_thr_fd: FdGuard::new(scratchpad.new_thr_fd),
        new_proc_fd: if scratchpad.new_proc_fd == usize::MAX {
            None
        } else {
            Some(FdGuard::new(scratchpad.new_proc_fd))
        },
    });
}

asmfunction!(__relibc_internal_fork_wrapper (usize) -> usize: ["
    .attribute arch, \"rv64gc\"  # rust bug 80608
    addi sp, sp, -200
    sd   s0, 0(sp)
    sd   s1, 8(sp)
    sd   s2, 16(sp)
    sd   s3, 24(sp)
    sd   s4, 32(sp)
    sd   s5, 40(sp)
    sd   s6, 48(sp)
    sd   s7, 56(sp)
    sd   s8, 64(sp)
    sd   s9, 72(sp)
    sd   s10, 80(sp)
    sd   s11, 88(sp)
    sd   ra, 96(sp)

    fsd  fs0, 104(sp)
    fsd  fs1, 112(sp)
    fsd  fs2, 120(sp)
    fsd  fs3, 128(sp)
    fsd  fs4, 136(sp)
    fsd  fs5, 144(sp)
    fsd  fs6, 152(sp)
    fsd  fs7, 160(sp)
    fsd  fs8, 168(sp)
    fsd  fs9, 176(sp)
    fsd  fs10, 184(sp)
    fsd  fs11, 192(sp)

    // a0 is forwarded from this function
    mv   a1, sp
    jal  {fork_impl}

    ld   s0, 0(sp)
    ld   s1, 8(sp)
    ld   s2, 16(sp)
    ld   s3, 24(sp)
    ld   s4, 32(sp)
    ld   s5, 40(sp)
    ld   s6, 48(sp)
    ld   s7, 56(sp)
    ld   s8, 64(sp)
    ld   s9, 72(sp)
    ld   s10, 80(sp)
    ld   s11, 88(sp)
    ld   ra, 96(sp)

    fld  fs0, 104(sp)
    fld  fs1, 112(sp)
    fld  fs2, 120(sp)
    fld  fs3, 128(sp)
    fld  fs4, 136(sp)
    fld  fs5, 144(sp)
    fld  fs6, 152(sp)
    fld  fs7, 160(sp)
    fld  fs8, 168(sp)
    fld  fs9, 176(sp)
    fld  fs10, 184(sp)
    fld  fs11, 192(sp)

    addi sp, sp, 200
    ret
"] <= [fork_impl = sym fork_impl]);

asmfunction!(__relibc_internal_fork_ret: ["
    .attribute arch, \"rv64gc\"  # rust bug 80608

    # scratchpad is in a1, move to a0 for child_hook
    mv a0, a1

    jal  {child_hook}

    mv   a0, zero

    ld   s0, 0(sp)
    ld   s1, 8(sp)
    ld   s2, 16(sp)
    ld   s3, 24(sp)
    ld   s4, 32(sp)
    ld   s5, 40(sp)
    ld   s6, 48(sp)
    ld   s7, 56(sp)
    ld   s8, 64(sp)
    ld   s9, 72(sp)
    ld   s10, 80(sp)
    ld   s11, 88(sp)
    ld   ra, 96(sp)

    fld  fs0, 104(sp)
    fld  fs1, 112(sp)
    fld  fs2, 120(sp)
    fld  fs3, 128(sp)
    fld  fs4, 136(sp)
    fld  fs5, 144(sp)
    fld  fs6, 152(sp)
    fld  fs7, 160(sp)
    fld  fs8, 168(sp)
    fld  fs9, 176(sp)
    fld  fs10, 184(sp)
    fld  fs11, 192(sp)

    addi sp, sp, 200
    ret
"] <= [child_hook = sym child_hook]);

asmfunction!(__relibc_internal_sigentry: ["
    .attribute arch, \"rv64gc\"  # rust bug 80608
    // Save some registers
    ld   t0, -8(tp) // Tcb
    sd   sp, ({tcb_sa_off} + {sa_tmp_sp})(t0)
    sd   t1, ({tcb_sa_off} + {sa_tmp_t1})(t0)
    sd   t2, ({tcb_sa_off} + {sa_tmp_t2})(t0)
    sd   t3, ({tcb_sa_off} + {sa_tmp_t3})(t0)
    sd   t4, ({tcb_sa_off} + {sa_tmp_t4})(t0)
    ld   t4, ({tcb_sa_off} + {sa_off_pctl})(t0)

    // First, select signal, always pick first available bit
99:
    // Read first signal word
    ld   t1, ({tcb_sc_off} + {sc_word})(t0)
    srli t2, t1, 32 // bitset to low word
    and  t1, t1, t2  // masked bitset in low word
    beqz t1, 3f

    // Found in first thread signal word
    mv   t3, x0
2:  andi t2, t1, 1
    bnez t2, 10f
    addi t3, t3, 1
    srli t1, t1, 1
    j 2b

    // If no unblocked thread signal was found, check for process.
    // This is competitive; we need to atomically check if *we* cleared the process-wide pending
    // bit, otherwise restart.
3:  lw   t1, {pctl_off_pending}(t4)
    and  t1, t1, t2
    beqz t1, 3f
    // Found in first process signal word
    li   t3, -1
2:  andi t2, t1, 1
    addi t3, t3, 1
    srli t1, t1, 1
    beqz t2, 2b
    slli t1, t3, 3 // * 8 == size_of SenderInfo
    add  t1, t1, t4
    ld   t1, {pctl_off_sender_infos}(t1)
    sd   t1, ({tcb_sa_off} + {sa_tmp_id_inf})(t0)
    li   t1, 1
    sll  t1, t1, t3
    not  t1, t1
    addi t2, t4, {pctl_off_pending}
    amoand.w.aq t2, t1, (t2)
    and  t1, t1, t2
    bne  t1, t2, 9f

3:
    // Read second signal word - both process and thread simultaneously.
    // This must be done since POSIX requires low realtime signals to be picked first.
    ld   t1, ({tcb_sc_off} + {sc_word} + 8)(t0)
    lw   t2, ({pctl_off_pending} + 4)(t4)
    or   t4, t1, t2
    srli t2, t1, 32
    and  t4, t2, t4
    beqz t4, 7f
    li   t3, -1
2:  andi t2, t4, 1
    addi t3, t3, 1
    srli t4, t4, 1
    beqz t2, 2b
    li   t2, 1
    sll  t2, t2, t3
    and  t1, t1, t2
    addi t3, t3, 32
    bnez t1, 10f // thread signal

    // otherwise, try (competitively) dequeueing realtime signal

    // SYS_CALL(fd, payload_base, payload_len, metadata_len, metadata_base)
    // a7       a0  a1            a2           a3            a4

    // TODO: This SYS_CALL invocation has not yet been tested due to toolchain issues.

    sd   a0, ({tcb_sa_off} + {sa_tmp_a0})(t0)
    sd   a1, ({tcb_sa_off} + {sa_tmp_a1})(t0)
    sd   a2, ({tcb_sa_off} + {sa_tmp_a2})(t0)
    sd   a3, ({tcb_sa_off} + {sa_tmp_a3})(t0)
    sd   a4, ({tcb_sa_off} + {sa_tmp_a4})(t0)
    sd   a7, ({tcb_sa_off} + {sa_tmp_a7})(t0)
    li   a7, {SYS_CALL}
    addi a2, t3, -32
    add  a1, t0, {tcb_sa_off} + {sa_tmp_rt_inf} // out pointer of dequeued realtime sig
    sd a2, (a1)
    li a2, {RTINF_SIZE}
    li a3, 1
1337:
    auipc a4, %pcrel_hi({proc_fd})
    addi a4, a4, %pcrel_lo(1337b)
    ecall
    ld   a3, ({tcb_sa_off} + {sa_tmp_a3})(t0)
    ld   a4, ({tcb_sa_off} + {sa_tmp_a4})(t0)
    bnez a0, 99b // assumes error can only be EAGAIN
    j   9f

10: // thread signal. t3 holds signal number
    srli t1, t3, 5
    bnez t1, 2f // FIXME senderinfo?
    sll  t2, t3, 3 // * 8 == size_of SenderInfo
    add  t2, t2, t0
    ld   t2, ({tcb_sc_off} + {sc_sender_infos})(t2)
    sd   t2, ({tcb_sa_off} + {sa_tmp_id_inf})(t0)
2:  andi t4, t3, 31
    li   t2, 1
    sll  t2, t2, t4
    not  t2, t2
    sll  t1, t1, 3
    add  t1, t1, t0
    addi t1, t1, {tcb_sc_off} + {sc_word}
    amoand.w.aq x0, t2, (t1)
    addi t3, t3, 64 // indicate signal was targeted at thread

9: // process signal t3 holds signal number

    // By now we have selected a signal, stored in eax (6-bit). We now need to choose whether or
    // not to switch to the alternate signal stack. If SA_ONSTACK is clear for this signal, then
    // skip the sigaltstack logic.
    ld   t4, ({tcb_sa_off} + {sa_off_pctl})(t0)
    andi t1, t3, 63
    slli t1, t1, 4 // * 16 == size_of RawAction
    add  t1, t1, t4
    ld   t1, {pctl_off_actions}(t1)
    slli t1, t1, 63-58 // SA_ONSTACK in sign bit
    bgez t1, 3f

    // If current RSP is above altstack region, switch to altstack
    ld  t1, ({tcb_sa_off} + {sa_altstack_top})(t0)
    bgtu sp, t1, 2f
    ld  t2, ({tcb_sa_off} + {sa_altstack_bottom})(t0)
    bgtu sp, t3, 3f
2:  mv  sp, t1
3:
    // form mcontext on stack
    addi sp, sp, -33 * 8
    fsd  f0, (0 * 8)(sp)
    fsd  f1, (1 * 8)(sp)
    fsd  f2, (2 * 8)(sp)
    fsd  f3, (3 * 8)(sp)
    fsd  f4, (4 * 8)(sp)
    fsd  f5, (5 * 8)(sp)
    fsd  f6, (6 * 8)(sp)
    fsd  f7, (7 * 8)(sp)
    fsd  f8, (8 * 8)(sp)
    fsd  f9, (9 * 8)(sp)
    fsd  f10, (10 * 8)(sp)
    fsd  f11, (11 * 8)(sp)
    fsd  f12, (12 * 8)(sp)
    fsd  f13, (13 * 8)(sp)
    fsd  f14, (14 * 8)(sp)
    fsd  f15, (15 * 8)(sp)
    fsd  f16, (16 * 8)(sp)
    fsd  f17, (17 * 8)(sp)
    fsd  f18, (18 * 8)(sp)
    fsd  f19, (19 * 8)(sp)
    fsd  f20, (20 * 8)(sp)
    fsd  f21, (21 * 8)(sp)
    fsd  f22, (22 * 8)(sp)
    fsd  f23, (23 * 8)(sp)
    fsd  f24, (24 * 8)(sp)
    fsd  f25, (25 * 8)(sp)
    fsd  f26, (26 * 8)(sp)
    fsd  f27, (27 * 8)(sp)
    fsd  f28, (28 * 8)(sp)
    fsd  f29, (29 * 8)(sp)
    fsd  f30, (30 * 8)(sp)
    fsd  f31, (31 * 8)(sp)
    csrr t1, fcsr
    sw   t1, (32 * 8)(sp)

    addi sp, sp, -32 * 8
    sd   x1, 0(sp)
    ld   t1, ({tcb_sa_off} + {sa_tmp_sp})(t0)
    sd   t1, (1 * 8)(sp)  // x2 is sp
    sd   x3, (2 * 8)(sp)
    sd   x4, (3 * 8)(sp)
    ld   t1, ({tcb_sc_off} + {sc_saved_t0})(t0)
    sd   t1, (4 * 8)(sp) // x5 is t0
    ld   t1, ({tcb_sa_off} + {sa_tmp_t1})(t0)
    sd   t1, (5 * 8)(sp) // x6 is t1
    ld   t1, ({tcb_sa_off} + {sa_tmp_t2})(t0)
    sd   t1, (6 * 8)(sp) // x7 is t2
    sd   x8, (7 * 8)(sp)
    sd   x9, (8 * 8)(sp)
    sd   x10, (9 * 8)(sp)
    sd   x11, (10 * 8)(sp)
    sd   x12, (11 * 8)(sp)
    sd   x13, (12 * 8)(sp)
    sd   x14, (13 * 8)(sp)
    sd   x15, (14 * 8)(sp)
    sd   x16, (15 * 8)(sp)
    sd   x17, (16 * 8)(sp)
    sd   x18, (17 * 8)(sp)
    sd   x19, (18 * 8)(sp)
    sd   x20, (19 * 8)(sp)
    sd   x21, (20 * 8)(sp)
    sd   x22, (21 * 8)(sp)
    sd   x23, (22 * 8)(sp)
    sd   x24, (23 * 8)(sp)
    sd   x25, (24 * 8)(sp)
    sd   x26, (25 * 8)(sp)
    sd   x27, (26 * 8)(sp)
    ld   t1, ({tcb_sa_off} + {sa_tmp_t3})(t0)
    sd   t1, (27 * 8)(sp) // t3 is x28
    ld   t1, ({tcb_sa_off} + {sa_tmp_t4})(t0)
    sd   t1, (28 * 8)(sp) // t4 is x29
    sd   x30, (29 * 8)(sp)
    sd   x31, (30 * 8)(sp)
    ld   t1, ({tcb_sc_off} + {sc_saved_ip})(t0)
    sd   t1, (31 * 8)(sp)

    // form ucontext
    addi sp, sp, -64
    sw   t3, 60(sp)

    mv   t0, sp
    jal  {inner}

    addi sp, sp, 64

    addi t0, sp, 32 * 8
    fld  f0, (0 * 8)(t0)
    fld  f1, (1 * 8)(t0)
    fld  f2, (2 * 8)(t0)
    fld  f3, (3 * 8)(t0)
    fld  f4, (4 * 8)(t0)
    fld  f5, (5 * 8)(t0)
    fld  f6, (6 * 8)(t0)
    fld  f7, (7 * 8)(t0)
    fld  f8, (8 * 8)(t0)
    fld  f9, (9 * 8)(t0)
    fld  f10, (10 * 8)(t0)
    fld  f11, (11 * 8)(t0)
    fld  f12, (12 * 8)(t0)
    fld  f13, (13 * 8)(t0)
    fld  f14, (14 * 8)(t0)
    fld  f15, (15 * 8)(t0)
    fld  f16, (16 * 8)(t0)
    fld  f17, (17 * 8)(t0)
    fld  f18, (18 * 8)(t0)
    fld  f19, (19 * 8)(t0)
    fld  f20, (20 * 8)(t0)
    fld  f21, (21 * 8)(t0)
    fld  f22, (22 * 8)(t0)
    fld  f23, (23 * 8)(t0)
    fld  f24, (24 * 8)(t0)
    fld  f25, (25 * 8)(t0)
    fld  f26, (26 * 8)(t0)
    fld  f27, (27 * 8)(t0)
    fld  f28, (28 * 8)(t0)
    fld  f29, (29 * 8)(t0)
    fld  f30, (30 * 8)(t0)
    fld  f31, (31 * 8)(t0)
    lw   t1, (32 * 8)(t0)
    csrw fcsr, t1

    ld   x1, 0(sp)
    // skip sp
    // skip gp
    ld   x4, (3 * 8)(sp)
    ld   x5, (4 * 8)(sp)
    ld   x6, (5 * 8)(sp)
    ld   x7, (6 * 8)(sp)
    ld   x8, (7 * 8)(sp)
    ld   x9, (8 * 8)(sp)
    ld   x10, (9 * 8)(sp)
    ld   x11, (10 * 8)(sp)
    ld   x12, (11 * 8)(sp)
    ld   x13, (12 * 8)(sp)
    ld   x14, (13 * 8)(sp)
    ld   x15, (14 * 8)(sp)
    ld   x16, (15 * 8)(sp)
    ld   x17, (16 * 8)(sp)
    ld   x18, (17 * 8)(sp)
    ld   x19, (18 * 8)(sp)
    ld   x20, (19 * 8)(sp)
    ld   x21, (20 * 8)(sp)
    ld   x22, (21 * 8)(sp)
    ld   x23, (22 * 8)(sp)
    ld   x24, (23 * 8)(sp)
    ld   x25, (24 * 8)(sp)
    ld   x26, (25 * 8)(sp)
    ld   x27, (26 * 8)(sp)
    ld   x28, (27 * 8)(sp)
    ld   x29, (28 * 8)(sp)
    ld   x30, (29 * 8)(sp)
    ld   x31, (30 * 8)(sp)
    ld   gp, (31 * 8)(sp) // new IP; this clobbers register x3/gp which is ABI reserved
    .global __relibc_internal_sigentry_crit_first
__relibc_internal_sigentry_crit_first:
    ld   sp, (1 * 8)(sp)
    .global __relibc_internal_sigentry_crit_second
__relibc_internal_sigentry_crit_second:
    jr   gp
7:
    // A spurious signal occurred. Signals are still disabled here, but will need to be re-enabled.

    // restore stack
    ld   sp, ({tcb_sa_off} + {sa_tmp_sp})(t0)

    // move saved IP away from control, allowing arch_pre to save us if interrupted.
    ld t1, ({tcb_sc_off} + {sc_saved_ip})(t0)
    sd t1, ({tcb_sa_off} + {sa_tmp_ip})(t0)

    // restore regs
    ld   t2, ({tcb_sa_off} + {sa_tmp_t2})(t0)
    ld   t3, ({tcb_sa_off} + {sa_tmp_t3})(t0)
    ld   t4, ({tcb_sa_off} + {sa_tmp_t4})(t0)

    // move saved t0 away from control as well
    mv   t1, t0
    ld   t0, ({tcb_sc_off} + {sc_saved_t0})(t0)

    // Re-enable signals. This code can be interrupted after this signal, so we need to define
    // 'crit_third'.
    ld   gp, ({tcb_sc_off} + {sc_control})(t1)
    andi gp, gp, ~1
    sd   gp, ({tcb_sc_off} + {sc_control})(t1)

    .globl __relibc_internal_sigentry_crit_third
__relibc_internal_sigentry_crit_third:
    ld   gp, ({tcb_sa_off} + {sa_tmp_ip})(t1)
    .globl __relibc_internal_sigentry_crit_fourth
__relibc_internal_sigentry_crit_fourth:
    ld   t1, ({tcb_sa_off} + {sa_tmp_t1})(t1)
    .globl __relibc_internal_sigentry_crit_fifth
__relibc_internal_sigentry_crit_fifth:
    jr   gp
 "] <= [
    tcb_sc_off = const (offset_of!(crate::Tcb, os_specific) + offset_of!(RtSigarea, control)),
    sc_word = const offset_of!(Sigcontrol, word),
    sc_saved_t0 = const offset_of!(Sigcontrol, saved_archdep_reg),
    sc_saved_ip = const offset_of!(Sigcontrol, saved_ip),
    sc_sender_infos = const offset_of!(Sigcontrol, sender_infos),
    sc_control = const offset_of!(Sigcontrol, control_flags),

    tcb_sa_off = const (offset_of!(crate::Tcb, os_specific) + offset_of!(RtSigarea, arch)),
    sa_off_pctl = const offset_of!(SigArea, pctl),
    sa_tmp_sp = const offset_of!(SigArea, tmp_sp),
    sa_tmp_t1 = const offset_of!(SigArea, tmp_t1),
    sa_tmp_t2 = const offset_of!(SigArea, tmp_t2),
    sa_tmp_t3 = const offset_of!(SigArea, tmp_t3),
    sa_tmp_t4 = const offset_of!(SigArea, tmp_t4),
    sa_tmp_a0 = const offset_of!(SigArea, tmp_a0),
    sa_tmp_a1 = const offset_of!(SigArea, tmp_a1),
    sa_tmp_a2 = const offset_of!(SigArea, tmp_a2),
    sa_tmp_a3 = const offset_of!(SigArea, tmp_a3),
    sa_tmp_a4 = const offset_of!(SigArea, tmp_a4),
    sa_tmp_a7 = const offset_of!(SigArea, tmp_a7),
    sa_tmp_ip = const offset_of!(SigArea, tmp_ip),
    sa_tmp_id_inf = const offset_of!(SigArea, tmp_id_inf),
    sa_tmp_rt_inf = const offset_of!(SigArea, tmp_rt_inf),
    sa_altstack_top = const offset_of!(SigArea, altstack_top),
    sa_altstack_bottom = const offset_of!(SigArea, altstack_bottom),

    pctl_off_actions = const offset_of!(SigProcControl, actions),
    inner = sym inner_c,
    pctl_off_pending = const offset_of!(SigProcControl, pending),
    pctl_off_sender_infos = const offset_of!(SigProcControl, sender_infos),
    SYS_CALL = const syscall::SYS_CALL,
    RTINF_SIZE = const size_of::<RtSigInfo>(),
    proc_fd = sym PROC_FD,
]);

asmfunction!(__relibc_internal_rlct_clone_ret: ["
    ld t0, 0(sp)
    ld a0, 8(sp)
    ld a1, 16(sp)
    ld a2, 24(sp)
    ld a3, 32(sp)
    ld a4, 40(sp)
    ld a5, 48(sp)
    addi sp, sp, 56

    jalr t0
    ret
"] <= []);

pub fn current_sp() -> usize {
    let sp: usize;
    unsafe {
        core::arch::asm!(
        "mv {}, sp",
        out(reg) sp,
        options(nomem));
    }
    sp
}

pub unsafe fn manually_enter_trampoline() {
    let ctl = &Tcb::current().unwrap().os_specific.control;

    ctl.control_flags.store(
        ctl.control_flags.load(Ordering::Relaxed) | syscall::flag::INHIBIT_DELIVERY.bits(),
        Ordering::Release,
    );
    ctl.saved_archdep_reg.set(0);
    let ip_location = &ctl.saved_ip as *const _ as usize;

    core::arch::asm!("
        jal 2f
        j 3f
    2:
        sd ra, 0(t0)
        la t0, __relibc_internal_sigentry
        jalr x0, t0
    3:
    ", inout("t0") ip_location => _, out("ra") _);
}

unsafe extern "C" {
    fn __relibc_internal_sigentry_crit_first();
    fn __relibc_internal_sigentry_crit_second();
    fn __relibc_internal_sigentry_crit_third();
    fn __relibc_internal_sigentry_crit_fourth();
    fn __relibc_internal_sigentry_crit_fifth();
}
pub unsafe fn arch_pre(stack: &mut SigStack, area: &mut SigArea) -> PosixStackt {
    // It is impossible to update SP and PC atomically. Instead, we abuse the fact that
    // signals are disabled in the prologue of the signal trampoline, which allows us to emulate
    // atomicity inside the critical section, consisting of one instruction at 'crit_first', and
    // one at 'crit_second', see asm.

    if stack.regs.pc == __relibc_internal_sigentry_crit_first as u64 {
        // Reexecute 'ld sp, (1 * 8)(sp)'
        let stack_ptr = stack.regs.int_regs[1] as *const u64; // x2
        stack.regs.int_regs[1] = stack_ptr.add(1).read();
        // and 'jr gp' steps.
        stack.regs.pc = stack.regs.int_regs[2];
    } else if stack.regs.pc == __relibc_internal_sigentry_crit_second as u64
        || stack.regs.pc == __relibc_internal_sigentry_crit_fifth as u64
    {
        // just reexecute the jump
        stack.regs.pc = stack.regs.int_regs[2];
    } else if stack.regs.pc == __relibc_internal_sigentry_crit_third as u64 {
        // ld   gp, ({tcb_sa_off} + {sa_tmp_ip})(t1)
        stack.regs.int_regs[2] = area.tmp_ip;
        // ld   t1, ({tcb_sa_off} + {sa_tmp_t1})(t1)
        stack.regs.int_regs[5] = area.tmp_t1;
        // j    gp
        stack.regs.pc = stack.regs.int_regs[2];
    } else if stack.regs.pc == __relibc_internal_sigentry_crit_fourth as u64 {
        // ld   t1, ({tcb_sa_off} + {sa_tmp_t1})(t1)
        stack.regs.int_regs[5] = area.tmp_t1;
        // jr   gp
        stack.regs.pc = stack.regs.int_regs[2];
    }

    get_sigaltstack(area, stack.regs.int_regs[1] as usize).into()
}
pub(crate) static PROC_FD: SyncUnsafeCell<usize> = SyncUnsafeCell::new(usize::MAX);
static PROC_CALL: [usize; 1] = [ProcCall::Sigdeq as usize];
