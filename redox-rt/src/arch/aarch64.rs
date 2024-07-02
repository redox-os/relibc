use core::mem::offset_of;

use syscall::data::*;
use syscall::error::*;

use crate::proc::{fork_inner, FdGuard};
use crate::signal::SigStack;
use crate::signal::{inner_c, RtSigarea, PROC_CONTROL_STRUCT};
use crate::Tcb;

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
    pub tmp_sp: usize,
    pub onstack: u64,
    pub disable_signals_depth: u64,
    pub pctl: usize, // TODO: remove
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

unsafe extern "C" fn fork_impl(initial_rsp: *mut usize) -> usize {
    Error::mux(fork_inner(initial_rsp))
}

unsafe extern "C" fn child_hook(cur_filetable_fd: usize, new_pid_fd: usize) {
    let _ = syscall::close(cur_filetable_fd);
    let _ = syscall::close(new_pid_fd);
}

asmfunction!(__relibc_internal_fork_wrapper -> usize: ["
    stp     x29, x30, [sp, #-16]!
    stp     x27, x28, [sp, #-16]!
    stp     x25, x26, [sp, #-16]!
    stp     x23, x24, [sp, #-16]!
    stp     x21, x22, [sp, #-16]!
    stp     x19, x20, [sp, #-16]!

    sub sp, sp, #32

    //TODO: store floating point regs

    mov x0, sp
    bl {fork_impl}

    add sp, sp, #128
    ret
"] <= [fork_impl = sym fork_impl]);

asmfunction!(__relibc_internal_fork_ret: ["
    ldp x0, x1, [sp]
    bl {child_hook}

    //TODO: load floating point regs

    mov x0, xzr

    add sp, sp, #32
    ldp     x19, x20, [sp], #16
    ldp     x21, x22, [sp], #16
    ldp     x23, x24, [sp], #16
    ldp     x25, x26, [sp], #16
    ldp     x27, x28, [sp], #16
    ldp     x29, x30, [sp], #16

    ret
"] <= [child_hook = sym child_hook]);

asmfunction!(__relibc_internal_sigentry: ["
    // old pc and x0 are saved in the sigcontrol struct
    mrs x0, tpidr_el0 // ABI ptr
    ldr x0, [x0] // TCB ptr

    // save x1-x3 and sp
    stp x1, x2, [x0, #{tcb_sa_off} + {sa_tmp_x1_x2}]
    stp x3, x4, [x0, #{tcb_sa_off} + {sa_tmp_x3_x4}]
    mov x1, sp
    str x1, [x0, #{tcb_sa_off} + {sa_tmp_sp}]

    sub x1, x1, 128
    and x1, x1, -16
    mov sp, x1

    ldr x3, [x0, #{tcb_sa_off} + {sa_pctl}]

    // load x1 and x2 with each word (tearing between x1 and x2 can occur)
    // acquire ordering
    add x2, x0, #{tcb_sc_off} + {sc_word}
    ldaxp x1, x2, [x2]

    // reduce them by ANDing the upper and lower 32 bits
    and x1, x1, x1, lsr #32 // put result in lo half
    and x2, x2, x2, lsl #32 // put result in hi half
    orr x1, x1, x2 // combine them into the set of pending unblocked

    // count trailing zeroes, to find signal bit
    rbit x1, x1
    clz x1, x1
    mov x2, #32
    sub x1, x2, x1

    // TODO: NOT ATOMIC!
    add x2, x3, w1, uxtb #4 // actions_base + sig_idx * sizeof Action
    ldxp x2, x3, [x2]

    // skip sigaltstack step if SA_ONSTACK is clear
    // tbz x2, #57, 2f
2:
    ldr x2, [x0, #{tcb_sc_off} + {sc_saved_pc}]
    ldr x3, [x0, #{tcb_sc_off} + {sc_saved_x0}]
    stp x2, x3, [sp], #-16

    ldr x2, [x0, #{tcb_sa_off} + {sa_tmp_sp}]
    mrs x3, nzcv
    stp x2, x3, [sp], #-16

    ldp x2, x3, [x0, #{tcb_sa_off} + {sa_tmp_x1_x2}]
    stp x2, x3, [sp], #-16
    ldp x3, x4, [x0, #{tcb_sa_off} + {sa_tmp_x3_x4}]
    stp x4, x3, [sp], #-16
    stp x6, x5, [sp], #-16
    stp x8, x7, [sp], #-16
    stp x10, x9, [sp], #-16
    stp x12, x11, [sp], #-16
    stp x14, x13, [sp], #-16
    stp x16, x15, [sp], #-16
    stp x18, x17, [sp], #-16
    stp x20, x19, [sp], #-16
    stp x22, x21, [sp], #-16
    stp x24, x23, [sp], #-16
    stp x26, x25, [sp], #-16
    stp x28, x27, [sp], #-16
    stp x30, x29, [sp], #-16

    mov x0, sp
    bl {inner}

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

    // x18 is reserved by ABI as 'platform register', so clobbering it should be safe.
    mov x18, sp
    ldr x0, [sp], #16
    mov sp, x0
    mov x0, x18

    ldp x18, x0, [x0]
    br x18
"] <= [
    tcb_sc_off = const (offset_of!(crate::Tcb, os_specific) + offset_of!(RtSigarea, control)),
    tcb_sa_off = const (offset_of!(crate::Tcb, os_specific) + offset_of!(RtSigarea, arch)),
    sa_tmp_x1_x2 = const offset_of!(SigArea, tmp_x1_x2),
    sa_tmp_x3_x4 = const offset_of!(SigArea, tmp_x3_x4),
    sa_tmp_sp = const offset_of!(SigArea, tmp_sp),
    sa_pctl = const offset_of!(SigArea, pctl),
    sc_saved_pc = const offset_of!(Sigcontrol, saved_ip),
    sc_saved_x0 = const offset_of!(Sigcontrol, saved_archdep_reg),
    sc_word = const offset_of!(Sigcontrol, word),
    inner = sym inner_c,
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
        ldr lr, [x0]
        b __relibc_internal_sigentry
    3:
    ", inout("x0") ip_location => _, out("lr") _);
}

pub unsafe fn arch_pre(stack: &mut SigStack, os: &mut SigArea) {}
