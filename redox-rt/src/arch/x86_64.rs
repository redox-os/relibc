use core::mem::offset_of;
use core::sync::atomic::AtomicU8;

use syscall::data::Sigcontrol;
use syscall::error::*;
use syscall::flag::*;

use crate::proc::{fork_inner, FdGuard};
use crate::signal::SigStack;
use crate::signal::{inner_c, RtSigarea};

// Setup a stack starting from the very end of the address space, and then growing downwards.
pub(crate) const STACK_TOP: usize = 1 << 47;
pub(crate) const STACK_SIZE: usize = 1024 * 1024;

#[derive(Debug, Default)]
#[repr(C)]
pub struct SigArea {
    pub tmp_rip: usize,
    pub tmp_rsp: usize,
    pub tmp_rax: usize,
    pub tmp_rdx: usize,

    pub altstack_top: usize,
    pub altstack_bottom: usize,
    pub onstack: u64,
    pub disable_signals_depth: u64,
}

#[repr(C, align(64))]
#[derive(Debug, Default)]
pub struct ArchIntRegs {
    _pad: [usize; 2], // ensure size is divisible by 32

    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbp: usize,
    pub rbx: usize,
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rax: usize,
    pub rcx: usize,
    pub rdx: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub rflags: usize,
    pub rip: usize,
    pub rsp: usize,
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

unsafe extern "sysv64" fn fork_impl(initial_rsp: *mut usize) -> usize {
    Error::mux(fork_inner(initial_rsp))
}

unsafe extern "sysv64" fn child_hook(cur_filetable_fd: usize, new_pid_fd: usize) {
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
    call {fork_impl}

    add rsp, 80

    pop rbp
    ret

"] <= [fork_impl = sym fork_impl]);
asmfunction!(__relibc_internal_fork_ret: ["
    mov rdi, [rsp]
    mov rsi, [rsp + 8]
    call {child_hook}

    ldmxcsr [rsp+16]
    fldcw [rsp+24]

    xor rax, rax

    add rsp, 32
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx

    pop rbp
    ret
"] <= [child_hook = sym child_hook]);
asmfunction!(__relibc_internal_rlct_clone_ret: ["
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
    // Save some registers
    mov fs:[{tcb_sa_off} + {sa_tmp_rsp}], rsp
    mov fs:[{tcb_sa_off} + {sa_tmp_rax}], rax
    mov fs:[{tcb_sa_off} + {sa_tmp_rdx}], rdx

    // First, select signal, always pick first available bit

    // Read first signal word
    mov rax, fs:[{tcb_sc_off} + {sc_word}]
    mov rdx, rax
    shr rdx, 32
    and eax, edx
    and eax, {SIGW0_PENDING_MASK}
    bsf eax, eax
    jnz 2f

    // Read second signal word
    mov rax, fs:[{tcb_sc_off} + {sc_word} + 8]
    mov rdx, rax
    shr rdx, 32
    and eax, edx
    and eax, {SIGW1_PENDING_MASK}
    bsf eax, eax
    jz 7f
    add eax, 32
2:
    sub rsp, {REDZONE_SIZE}
    and rsp, -{STACK_ALIGN}

    // By now we have selected a signal, stored in eax (6-bit). We now need to choose whether or
    // not to switch to the alternate signal stack. If SA_ONSTACK is clear for this signal, then
    // skip the sigaltstack logic.
    bt fs:[{tcb_sa_off} + {sa_onstack}], eax
    jnc 4f

    // Otherwise, the altstack is already active. The sigaltstack being disabled, is equivalent
    // to setting 'top' to usize::MAX and 'bottom' to 0.

    // If current RSP is above altstack region, switch to altstack
    mov rdx, fs:[{tcb_sa_off} + {sa_altstack_top}]
    cmp rsp, rdx
    cmova rsp, rdx

    // If current RSP is below altstack region, also switch to altstack
    cmp rsp, fs:[{tcb_sa_off} + {sa_altstack_bottom}]
    cmovbe rsp, rdx

    .p2align 4
4:
    // Now that we have a stack, we can finally start initializing the signal stack!

    push fs:[{tcb_sa_off} + {sa_tmp_rsp}]
    push fs:[{tcb_sc_off} + {sc_saved_rip}]
    push fs:[{tcb_sc_off} + {sc_saved_rflags}]

    push rdi
    push rsi
    push fs:[{tcb_sa_off} + {sa_tmp_rdx}]
    push rcx
    push fs:[{tcb_sa_off} + {sa_tmp_rax}]
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
    sub rsp, 16

    push rax // selected signal

    sub rsp, 4096 + 24

    cld
    mov rdi, rsp
    xor eax, eax
    mov ecx, 4096 + 24
    rep stosb

    // TODO: self-modifying?
    cmp byte ptr [rip + {supports_xsave}], 0
    je 6f

    mov eax, 0xffffffff
    mov edx, eax
    xsave [rsp]

    mov rdi, rsp
    call {inner}

    mov eax, 0xffffffff
    mov edx, eax
    xrstor [rsp]

5:
    add rsp, 4096 + 32 + 16
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

    popfq
    pop qword ptr fs:[{tcb_sa_off} + {sa_tmp_rip}]

    // x86 lacks atomic instructions for setting both the stack and instruction pointer
    // simultaneously, except the slow microcoded IRETQ instruction. Thus, we let the arch_pre
    // function emulate atomicity between the pop rsp and indirect jump.

    .globl __relibc_internal_sigentry_crit_first
__relibc_internal_sigentry_crit_first:

    pop rsp

    .globl __relibc_internal_sigentry_crit_second
__relibc_internal_sigentry_crit_second:
    jmp qword ptr fs:[{tcb_sa_off} + {sa_tmp_rip}]
6:
    fxsave64 [rsp]

    mov rdi, rsp
    call {inner}

    fxrstor64 [rsp]
    jmp 5b
7:
    ud2
    // Spurious signal
"] <= [
    inner = sym inner_c,
    sa_tmp_rip = const offset_of!(SigArea, tmp_rip),
    sa_tmp_rsp = const offset_of!(SigArea, tmp_rsp),
    sa_tmp_rax = const offset_of!(SigArea, tmp_rax),
    sa_tmp_rdx = const offset_of!(SigArea, tmp_rdx),
    sa_altstack_top = const offset_of!(SigArea, altstack_top),
    sa_altstack_bottom = const offset_of!(SigArea, altstack_bottom),
    sa_onstack = const offset_of!(SigArea, onstack),
    sc_saved_rflags = const offset_of!(Sigcontrol, saved_archdep_reg),
    sc_saved_rip = const offset_of!(Sigcontrol, saved_ip),
    sc_word = const offset_of!(Sigcontrol, word),
    tcb_sa_off = const offset_of!(crate::Tcb, os_specific) + offset_of!(RtSigarea, arch),
    tcb_sc_off = const offset_of!(crate::Tcb, os_specific) + offset_of!(RtSigarea, control),
    supports_xsave = sym SUPPORTS_XSAVE,
    SIGW0_PENDING_MASK = const !0,
    SIGW1_PENDING_MASK = const !0,
    REDZONE_SIZE = const 128,
    STACK_ALIGN = const 64, // if xsave is used
]);

extern "C" {
    fn __relibc_internal_sigentry_crit_first();
    fn __relibc_internal_sigentry_crit_second();
}
pub unsafe fn arch_pre(stack: &mut SigStack, area: &mut SigArea) {
    // It is impossible to update RSP and RIP atomically on x86_64, without using IRETQ, which is
    // almost as slow as calling a SIGRETURN syscall would be. Instead, we abuse the fact that
    // signals are disabled in the prologue of the signal trampoline, which allows us to emulate
    // atomicity inside the critical section, consisting of one instruction at 'crit_first', and
    // one at 'crit_second', see asm.

    if stack.regs.rip == __relibc_internal_sigentry_crit_first as usize {
        // Reexecute pop rsp and jump steps. This case needs to be different from the one below,
        // since rsp has not been overwritten with the previous context's stack, just yet. At this
        // point, we know [rsp+0] contains the saved RSP, and [rsp-8] contains the saved RIP.
        let stack_ptr = stack.regs.rsp as *const usize;
        stack.regs.rsp = stack_ptr.read();
        stack.regs.rip = stack_ptr.sub(1).read();
    } else if stack.regs.rip == __relibc_internal_sigentry_crit_second as usize {
        // Almost finished, just reexecute the jump before tmp_rip is overwritten by this
        // deeper-level signal.
        stack.regs.rip = area.tmp_rip;
    }
}

static SUPPORTS_XSAVE: AtomicU8 = AtomicU8::new(1); // FIXME
