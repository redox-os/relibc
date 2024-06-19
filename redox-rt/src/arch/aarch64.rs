use syscall::error::*;

use crate::{fork_inner, FdGuard};

// Setup a stack starting from the very end of the address space, and then growing downwards.
pub(crate) const STACK_TOP: usize = 1 << 47;
pub(crate) const STACK_SIZE: usize = 1024 * 1024;

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

#[no_mangle]
unsafe extern "C" fn __relibc_internal_fork_impl(initial_rsp: *mut usize) -> usize {
    Error::mux(fork_inner(initial_rsp))
}

#[no_mangle]
unsafe extern "C" fn __relibc_internal_fork_hook(cur_filetable_fd: usize, new_pid_fd: usize) {
    let _ = syscall::close(cur_filetable_fd);
    let _ = syscall::close(new_pid_fd);
}

core::arch::global_asm!(
    "
    .p2align 6
    .globl __relibc_internal_fork_wrapper
    .type __relibc_internal_fork_wrapper, @function
__relibc_internal_fork_wrapper:
    stp     x29, x30, [sp, #-16]!
    stp     x27, x28, [sp, #-16]!
    stp     x25, x26, [sp, #-16]!
    stp     x23, x24, [sp, #-16]!
    stp     x21, x22, [sp, #-16]!
    stp     x19, x20, [sp, #-16]!

    sub sp, sp, #32

    //TODO: store floating point regs

    mov x0, sp
    bl __relibc_internal_fork_impl
    b 2f

    .size __relibc_internal_fork_wrapper, . - __relibc_internal_fork_wrapper

    .p2align 6
    .globl __relibc_internal_fork_ret
    .type __relibc_internal_fork_ret, @function
__relibc_internal_fork_ret:
    ldp x0, x1, [sp]
    bl __relibc_internal_fork_hook

    //TODO: load floating point regs

    mov x0, xzr

    .p2align 4
2:
    add sp, sp, #32
    ldp     x19, x20, [sp], #16
    ldp     x21, x22, [sp], #16
    ldp     x23, x24, [sp], #16
    ldp     x25, x26, [sp], #16
    ldp     x27, x28, [sp], #16
    ldp     x29, x30, [sp], #16

    ret

    .size __relibc_internal_fork_ret, . - __relibc_internal_fork_ret"
);

extern "C" {
    pub(crate) fn __relibc_internal_fork_wrapper() -> usize;
    pub(crate) fn __relibc_internal_fork_ret();
}
