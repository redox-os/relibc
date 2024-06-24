use syscall::error::*;

use crate::proc::{fork_inner, FdGuard};
use crate::signal::{inner_fastcall, RtSigarea};

// Setup a stack starting from the very end of the address space, and then growing downwards.
pub(crate) const STACK_TOP: usize = 1 << 31;
pub(crate) const STACK_SIZE: usize = 1024 * 1024;

#[derive(Debug, Default)]
pub struct SigArea {
    pub altstack_top: usize,
    pub altstack_bottom: usize,
    pub tmp: usize,
    pub onstack: u64,
    pub disable_signals_depth: u64,
}

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

unsafe extern "cdecl" fn fork_impl(initial_rsp: *mut usize) -> usize {
    Error::mux(fork_inner(initial_rsp))
}

unsafe extern "cdecl" fn child_hook(cur_filetable_fd: usize, new_pid_fd: usize) {
    let _ = syscall::close(cur_filetable_fd);
    let _ = syscall::close(new_pid_fd);
}

asmfunction!(__relibc_internal_fork_wrapper -> usize: ["
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
    call {fork_impl}
    pop esp
    jmp 2f
"] <= [fork_impl = sym fork_impl]);

asmfunction!(__relibc_internal_fork_ret: ["
    // Arguments already on the stack
    call {child_hook}

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
"] <= [child_hook = sym child_hook]);
asmfunction!(__relibc_internal_sigentry: ["
    sub esp, 512
    fxsave [esp]

    mov ecx, esp
    call {inner}

    add esp, 512
    fxrstor [esp]

    ud2
"] <= [inner = sym inner_fastcall]);

asmfunction!(__relibc_internal_rlct_clone_ret -> usize: ["
    # Load registers
    pop eax

    sub esp, 8

    mov DWORD PTR [esp], 0x00001F80
    # TODO: ldmxcsr [esp]
    mov WORD PTR [esp], 0x037F
    fldcw [esp]

    add esp, 8

    # Call entry point
    call eax

    ret
"] <= []);
