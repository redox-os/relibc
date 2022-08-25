use syscall::error::*;

use crate::{FdGuard, fork_inner};

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

core::arch::global_asm!("
    .p2align 6
    .globl __relibc_internal_fork_wrapper
    .type __relibc_internal_fork_wrapper, @function
__relibc_internal_fork_wrapper:
    str x19, [sp, #-8]!
    str x20, [sp, #-8]!
    str x21, [sp, #-8]!
    str x22, [sp, #-8]!
    str x23, [sp, #-8]!
    str x24, [sp, #-8]!
    str x25, [sp, #-8]!
    str x26, [sp, #-8]!
    str x27, [sp, #-8]!
    str x28, [sp, #-8]!
    str x29, [sp, #-8]!
    str x30, [sp, #-8]!

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
    ldr x0, [sp]
    ldr x1, [sp, #8]
    bl __relibc_internal_fork_hook

    //TODO: load floating point regs

    mov x0, xzr

    .p2align 4
2:
    add sp, sp, #32
    ldr x30, [sp], #8
    ldr x29, [sp], #8
    ldr x28, [sp], #8
    ldr x27, [sp], #8
    ldr x26, [sp], #8
    ldr x25, [sp], #8
    ldr x24, [sp], #8
    ldr x23, [sp], #8
    ldr x22, [sp], #8
    ldr x21, [sp], #8
    ldr x20, [sp], #8
    ldr x19, [sp], #8
    ret

    .size __relibc_internal_fork_ret, . - __relibc_internal_fork_ret"
);

extern "C" {
    pub(crate) fn __relibc_internal_fork_wrapper() -> usize;
    pub(crate) fn __relibc_internal_fork_ret();
}
