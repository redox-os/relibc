use core::mem::offset_of;

use syscall::*;

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
    // Read pending half of first signal. This can be done nonatomically wrt the mask bits, since
    // only this thread is allowed to modify the latter.

    // Read first signal word
    mov eax, gs:[{tcb_sc_off} + {sc_word}]
    mov edx, gs:[{tcb_sc_off} + {sc_word} + 4]
    not edx
    and eax, edx
    and eax, {SIGW0_PENDING_MASK}
    bsf eax, eax
    jnz 2f

    // Read second signal word
    mov eax, gs:[{tcb_sc_off} + {sc_word} + 8]
    mov edx, gs:[{tcb_sc_off} + {sc_word} + 12]
    not edx
    and eax, edx
    and eax, {SIGW1_PENDING_MASK}
    bsf eax, eax
    jz 7f
    add eax, 32
2:
    and esp, -{STACK_ALIGN}
    bt gs:[{tcb_sa_off} + {sa_onstack}], eax
    jnc 4f

    mov edx, gs:[{tcb_sa_off} + {sa_altstack_top}]
    cmp esp, edx
    ja 3f

    cmp esp, gs:[{tcb_sa_off} + {sa_altstack_bottom}]
    jnbe 4f
3:
    mov esp, edx
4:
    // Now that we have a stack, we can finally start populating the signal stack.
    push fs
    .byte 0x66, 0x6a, 0x00 // pushw 0
    push ss
    .byte 0x66, 0x6a, 0x00 // pushw 0
    push dword ptr gs:[{tcb_sc_off} + {sc_saved_esp}]
    push dword ptr gs:[{tcb_sc_off} + {sc_saved_eflags}]
    push cs
    .byte 0x66, 0x6a, 0x00 // pushw 0
    push dword ptr gs:[{tcb_sc_off} + {sc_saved_eip}]

    push dword ptr gs:[{tcb_sc_off} + {sc_saved_edx}]
    push ecx
    push dword ptr gs:[{tcb_sc_off} + {sc_saved_eax}]
    push ebx
    push edi
    push esi
    push ebp

    push eax
    sub esp, 512 + 8
    fxsave [esp]

    mov ecx, esp
    call {inner}

    fxrstor [esp]
    add esp, 512 + 12

    pop ebp
    pop esi
    pop edi
    pop ebx
    pop eax
    pop ecx
    pop edx

    pop dword ptr gs:[{tcb_sa_off} + {sa_tmp}]
    add esp, 4
    popfd
    pop esp
    jmp dword ptr gs:[{tcb_sa_off} + {sa_tmp}]
7:
    ud2
"] <= [
    inner = sym inner_fastcall,
    sa_tmp = const offset_of!(SigArea, tmp),
    sa_altstack_top = const offset_of!(SigArea, altstack_top),
    sa_altstack_bottom = const offset_of!(SigArea, altstack_bottom),
    sa_onstack = const offset_of!(SigArea, onstack),
    sc_saved_eax = const offset_of!(Sigcontrol, saved_scratch_a),
    sc_saved_edx = const offset_of!(Sigcontrol, saved_scratch_b),
    sc_saved_eflags = const offset_of!(Sigcontrol, saved_flags),
    sc_saved_eip = const offset_of!(Sigcontrol, saved_ip),
    sc_saved_esp = const offset_of!(Sigcontrol, saved_sp),
    sc_word = const offset_of!(Sigcontrol, word),
    tcb_sa_off = const offset_of!(crate::Tcb, os_specific) + offset_of!(RtSigarea, arch),
    tcb_sc_off = const offset_of!(crate::Tcb, os_specific) + offset_of!(RtSigarea, control),
    SIGW0_PENDING_MASK = const !(
        SIGW0_TSTP_IS_STOP_BIT | SIGW0_TTIN_IS_STOP_BIT | SIGW0_TTOU_IS_STOP_BIT | SIGW0_NOCLDSTOP_BIT | SIGW0_UNUSED1 | SIGW0_UNUSED2
    ),
    SIGW1_PENDING_MASK = const !0,
    STACK_ALIGN = const 16,
]);

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
