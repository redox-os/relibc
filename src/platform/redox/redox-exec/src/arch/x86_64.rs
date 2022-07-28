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

    env.fsbase = 0;
    env.gsbase = 0;

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
unsafe extern "sysv64" fn __relibc_internal_fork_impl(initial_rsp: *mut usize) -> usize {
    Error::mux(fork_inner(initial_rsp))
}

#[no_mangle]
unsafe extern "sysv64" fn __relibc_internal_fork_hook(cur_filetable_fd: usize, new_pid_fd: usize) {
    let _ = syscall::close(cur_filetable_fd);
    let _ = syscall::close(new_pid_fd);
}

core::arch::global_asm!("
    .p2align 6
    .globl __relibc_internal_fork_wrapper
    .type __relibc_internal_fork_wrapper, @function
__relibc_internal_fork_wrapper:
    push rbp
    mov rbp, rsp

    push rbx
    push rbp
    push r12
    push r13
    push r14
    push r15

    sub rsp, 32

    stmxcsr [rsp+16]
    fnstcw [rsp+24]

    mov rdi, rsp
    call __relibc_internal_fork_impl
    jmp 2f

    .size __relibc_internal_fork_wrapper, . - __relibc_internal_fork_wrapper

    .p2align 6
    .globl __relibc_internal_fork_ret
    .type __relibc_internal_fork_ret, @function
__relibc_internal_fork_ret:
    mov rdi, [rsp]
    mov rsi, [rsp + 8]
    call __relibc_internal_fork_hook

    ldmxcsr [rsp+16]
    fldcw [rsp+24]

    xor rax, rax

    .p2align 4
2:
    add rsp, 32
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx

    pop rbp
    ret

    .size __relibc_internal_fork_ret, . - __relibc_internal_fork_ret"
);

extern "sysv64" {
    pub(crate) fn __relibc_internal_fork_wrapper() -> usize;
    pub(crate) fn __relibc_internal_fork_ret();
}
