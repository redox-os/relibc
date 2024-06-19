use syscall::error::*;

use crate::proc::{fork_inner, FdGuard};
use crate::signal::inner_c;

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

asmfunction!(__relibc_internal_fork_wrapper -> usize: ["
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

"] <= []);
asmfunction!(__relibc_internal_fork_ret: ["
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
"] <= []);
// TODO: is the memset necessary?
asmfunction!(__relibc_internal_sigentry_fxsave: ["
    sub rsp, 4096

    fxsave64 [rsp]

    mov rdi, rsp
    call {inner}

    fxrstor64 [rsp]
    add rsp, 4096

    //mov eax, {{SYS_SIGRETURN}}
    syscall
"] <= [inner = sym inner_c]);
asmfunction!(__relibc_internal_sigentry_xsave: ["
    sub rsp, 4096

    cld
    mov rdi, rsp
    xor eax, eax
    mov ecx, 4096
    rep stosb

    mov eax, 0xffffffff
    mov edx, eax
    xsave [rsp]

    mov rdi, rsp
    call {inner}

    mov eax, 0xffffffff
    mov edx, eax
    xrstor [rsp]
    add rsp, 4096

    //mov eax, {{SYS_SIGRETURN}}
    syscall
"] <= [inner = sym inner_c]);

asmfunction!(__relibc_internal_rlct_clone_ret -> usize: ["
    # Load registers
    pop rax
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop r8
    pop r9

    sub rsp, 8

    mov DWORD PTR [rsp], 0x00001F80
    ldmxcsr [rsp]
    mov WORD PTR [rsp], 0x037F
    fldcw [rsp]

    add rsp, 8

    # Call entry point
    call rax

    ret
"] <= []);
