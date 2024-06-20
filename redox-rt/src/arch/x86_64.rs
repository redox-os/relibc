use core::mem::offset_of;
use core::sync::atomic::AtomicU8;

use syscall::data::Sigcontrol;
use syscall::error::*;

use crate::proc::{fork_inner, FdGuard};
use crate::signal::inner_c;

// Setup a stack starting from the very end of the address space, and then growing downwards.
pub(crate) const STACK_TOP: usize = 1 << 47;
pub(crate) const STACK_SIZE: usize = 1024 * 1024;

#[derive(Debug, Default)]
pub struct SigArea {
    altstack_top: usize,
    altstack_bottom: usize,
    tmp: usize,
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

asmfunction!(__relibc_internal_sigentry: ["
    // Get offset to TCB
    mov rax, gs:[0]

    // If current RSP is above altstack region, switch to altstack
    mov rdx, [rax + {tcb_sa_off} + {sa_altstack_top}]
    cmp rdx, rsp
    cmova rsp, rdx

    // If current RSP is below altstack region, also switch to altstack
    mov rdx, [rax + {tcb_sa_off} + {sa_altstack_bottom}]
    cmp rdx, rax
    cmovbe rsp, rdx

    // Otherwise, the altstack is already active. The sigaltstack being disabled, is equivalent
    // to setting 'top' to usize::MAX and 'bottom' to 0.
    //
    // Now that we have a stack, we can finally start initializing the signal stack!

    push ss
    push [rax + {tcb_sc_off} + {sc_saved_rsp}]
    push [rax + {tcb_sc_off} + {sc_saved_rflags}]
    push cs
    push [rax + {tcb_sc_off} + {sc_saved_rip}]

    push rdi
    push rsi
    push [rax + {tcb_sc_off} + {sc_saved_rdx}]
    push rcx
    push [rax + {tcb_sc_off} + {sc_saved_rax}]
    push r8
    push r9
    push r10
    push r11
    push rbx
    push rbp
    push r12
    push r13
    push r14
    push r15

    sub rsp, 4096 + 32

    cld
    mov rdi, rsp
    xor eax, eax
    mov ecx, 4096 + 32
    rep stosb

    // TODO: self-modifying?
    cmp byte ptr [{supports_xsave}], 0
    je 3f

    mov eax, 0xffffffff
    mov edx, eax
    xsave [rsp]

    mov rdi, rsp
    call {inner}

    mov eax, 0xffffffff
    mov edx, eax
    xrstor [rsp]

    add rsp, 4096 + 32
2:
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx
    pop r11
    pop r10
    pop r9
    pop r8
    pop rax
    pop rcx
    pop rdx
    pop rsi
    pop rdi

    pop gs:[{tcb_sa_off} + {sa_tmp}]
    add rsp, 8
    popfq
    pop rsp
    jmp gs:[{tcb_sa_off} + {sa_tmp}]
3:
    fxsave64 [rsp]

    mov rdi, rsp
    call {inner}

    fxrstor64 [rsp]
    jmp 2b
"] <= [
    inner = sym inner_c,
    sa_tmp = const offset_of!(SigArea, tmp),
    sa_altstack_top = const offset_of!(SigArea, altstack_top),
    sa_altstack_bottom = const offset_of!(SigArea, altstack_bottom),
    sc_saved_rax = const offset_of!(Sigcontrol, saved_scratch_a),
    sc_saved_rdx = const offset_of!(Sigcontrol, saved_scratch_b),
    sc_saved_rflags = const offset_of!(Sigcontrol, saved_flags),
    sc_saved_rip = const offset_of!(Sigcontrol, saved_ip),
    sc_saved_rsp = const offset_of!(Sigcontrol, saved_sp),
    tcb_sa_off = const 0, // FIXME
    tcb_sc_off = const 0, // FIXME
    supports_xsave = sym SUPPORTS_XSAVE,
]);

static SUPPORTS_XSAVE: AtomicU8 = AtomicU8::new(0); // FIXME
