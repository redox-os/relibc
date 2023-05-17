use syscall::error::*;

use crate::{fork_inner, FdGuard};

// Setup a stack starting from the very end of the address space, and then growing downwards.
pub(crate) const STACK_TOP: usize = 1 << 31;
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
unsafe extern "cdecl" fn __relibc_internal_fork_impl(initial_rsp: *mut usize) -> usize {
    Error::mux(fork_inner(initial_rsp))
}

#[no_mangle]
unsafe extern "cdecl" fn __relibc_internal_fork_hook(cur_filetable_fd: usize, new_pid_fd: usize) {
    let _ = syscall::close(cur_filetable_fd);
    let _ = syscall::close(new_pid_fd);
}

//TODO: x86
core::arch::global_asm!(
    "
    .p2align 6
    .globl __relibc_internal_fork_wrapper
    .type __relibc_internal_fork_wrapper, @function
__relibc_internal_fork_wrapper:
    push ebp
    mov ebp, esp

    // Push preserved registers
    push ebx
    push edi
    push esi
    push ebp

    sub esp, 32

    //TODO stmxcsr [esp+16]
    fnstcw [esp+24]

    push esp
    call __relibc_internal_fork_impl
    pop esp
    jmp 2f

    .size __relibc_internal_fork_wrapper, . - __relibc_internal_fork_wrapper

    .p2align 6
    .globl __relibc_internal_fork_ret
    .type __relibc_internal_fork_ret, @function
__relibc_internal_fork_ret:
    // Arguments already on the stack
    call __relibc_internal_fork_hook

    //TODO ldmxcsr [esp+16]
    fldcw [esp+24]

    xor eax, eax

    .p2align 4
2:
    add esp, 32

    // Pop preserved registers
    pop ebp
    pop esi
    pop edi
    pop ebx

    pop ebp
    ret

    .size __relibc_internal_fork_ret, . - __relibc_internal_fork_ret"
);

extern "cdecl" {
    pub(crate) fn __relibc_internal_fork_wrapper() -> usize;
    pub(crate) fn __relibc_internal_fork_ret();
}
