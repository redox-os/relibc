use core::{
    mem::offset_of,
    ptr::NonNull,
    sync::atomic::{AtomicU8, Ordering},
};

use syscall::{
    data::{SigProcControl, Sigcontrol},
    error::*,
};

use crate::{
    proc::{fork_inner, FdGuard},
    signal::{inner_c, PosixStackt, RtSigarea, SigStack, PROC_CONTROL_STRUCT},
    RtTcb, Tcb,
};

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
    pub tmp_rdi: usize,
    pub tmp_rsi: usize,
    pub tmp_ptr: usize,

    pub altstack_top: usize,
    pub altstack_bottom: usize,
    pub disable_signals_depth: u64,
    pub last_sig_was_restart: bool,
    pub last_sigstack: Option<NonNull<SigStack>>,
}

#[repr(C, align(16))]
#[derive(Debug, Default)]
pub struct ArchIntRegs {
    pub ymm_upper: [u128; 16],
    pub fxsave: [u128; 29],
    pub r15: usize, // fxsave "available" +0
    pub r14: usize, // available +8
    pub r13: usize, // available +16
    pub r12: usize, // available +24
    pub rbp: usize, // available +32
    pub rbx: usize, // available +40
    pub r11: usize, // outside fxsave, and so on
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
    // TODO: Currently pidfd == threadfd, but this will not be the case later.
    RtTcb::current()
        .thr_fd
        .get()
        .write(Some(FdGuard::new(new_pid_fd)));
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

    ldmxcsr [rsp + 16]
    fldcw [rsp + 24]

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

    mov DWORD PTR [rsp - 8], 0x00001F80
    ldmxcsr [rsp - 8]
    mov WORD PTR [rsp - 8], 0x037F
    fldcw [rsp - 8]

    # Call entry point
    call rax

    ret
"] <= []);

asmfunction!(__relibc_internal_sigentry: ["
    // Save some registers
    mov fs:[{tcb_sa_off} + {sa_tmp_rsp}], rsp
    mov fs:[{tcb_sa_off} + {sa_tmp_rax}], rax
    mov fs:[{tcb_sa_off} + {sa_tmp_rdx}], rdx
    mov fs:[{tcb_sa_off} + {sa_tmp_rdi}], rdi
    mov fs:[{tcb_sa_off} + {sa_tmp_rsi}], rsi

    // First, select signal, always pick first available bit
1:
    // Read standard signal word - first targeting this thread
    mov rax, fs:[{tcb_sc_off} + {sc_word}]
    mov rdx, rax
    shr rdx, 32
    and eax, edx
    bsf eax, eax
    jnz 2f

    // If no unblocked thread signal was found, check for process.
    // This is competitive; we need to atomically check if *we* cleared the process-wide pending
    // bit, otherwise restart.
    mov eax, [rip + {pctl} + {pctl_off_pending}]
    and eax, edx
    bsf eax, eax
    jz 8f
    lock btr [rip + {pctl} + {pctl_off_pending}], eax
    jc 9f
8:
    // Read second signal word - both process and thread simultaneously.
    // This must be done since POSIX requires low realtime signals to be picked first.
    mov edx, fs:[{tcb_sc_off} + {sc_word} + 8]
    mov eax, [rip + {pctl} + {pctl_off_pending} + 4]
    or eax, edx
    and eax, fs:[{tcb_sc_off} + {sc_word} + 12]
    bsf eax, eax
    jz 7f

    bt edx, eax // check if signal was sent to thread specifically
    jc 2f // if so, continue as usual

    // otherwise, try (competitively) dequeueing realtime signal
    mov esi, eax
    mov eax, {SYS_SIGDEQUEUE}
    mov rdi, fs:[0]
    add rdi, {tcb_sa_off} + {sa_tmp_ptr} // out pointer of dequeued realtime sig
    syscall
    test eax, eax
    jnz 1b // assumes error can only be EAGAIN
    lea eax, [esi + 32]
    jmp 9f
2:
    mov edx, eax
    shr edx, 5
    lock btr fs:[{tcb_sc_off} + {sc_word} + edx * 4], eax
    add eax, 64 // indicate signal was targeted at thread
9:
    sub rsp, {REDZONE_SIZE}
    and rsp, -{STACK_ALIGN}

    // By now we have selected a signal, stored in eax (6-bit). We now need to choose whether or
    // not to switch to the alternate signal stack. If SA_ONSTACK is clear for this signal, then
    // skip the sigaltstack logic.
    lea rdx, [rip + {pctl} + {pctl_off_actions}]

    // LEA doesn't support 16x, so just do two x8s.
    lea rdx, [rdx + 8 * rax]
    lea rdx, [rdx + 8 * rax]

    bt qword ptr [rdx], {SA_ONSTACK_BIT}
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

    push fs:[{tcb_sa_off} + {sa_tmp_rdi}]
    push fs:[{tcb_sa_off} + {sa_tmp_rsi}]
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
    sub rsp, (29 + 16) * 16 // fxsave region minus available bytes
    fxsave64 [rsp + 16 * 16]

    // TODO: self-modifying?
    cmp byte ptr [rip + {supports_avx}], 0
    je 5f

    // Prefer vextractf128 over vextracti128 since the former only requires AVX version 1.
    vextractf128 [rsp + 15 * 16], ymm0, 1
    vextractf128 [rsp + 14 * 16], ymm1, 1
    vextractf128 [rsp + 13 * 16], ymm2, 1
    vextractf128 [rsp + 12 * 16], ymm3, 1
    vextractf128 [rsp + 11 * 16], ymm4, 1
    vextractf128 [rsp + 10 * 16], ymm5, 1
    vextractf128 [rsp + 9 * 16], ymm6, 1
    vextractf128 [rsp + 8 * 16], ymm7, 1
    vextractf128 [rsp + 7 * 16], ymm8, 1
    vextractf128 [rsp + 6 * 16], ymm9, 1
    vextractf128 [rsp + 5 * 16], ymm10, 1
    vextractf128 [rsp + 4 * 16], ymm11, 1
    vextractf128 [rsp + 3 * 16], ymm12, 1
    vextractf128 [rsp + 2 * 16], ymm13, 1
    vextractf128 [rsp + 16], ymm14, 1
    vextractf128 [rsp], ymm15, 1
5:
    push rax // selected signal
    push fs:[{tcb_sa_off} + {sa_tmp_ptr}]
    sub rsp, 48 // alloc space for ucontext fields

    mov rdi, rsp
    call {inner}

    add rsp, 64

    fxrstor64 [rsp + 16 * 16]

    cmp byte ptr [rip + {supports_avx}], 0
    je 6f

    vinsertf128 ymm0, ymm0, [rsp + 15 * 16], 1
    vinsertf128 ymm1, ymm1, [rsp + 14 * 16], 1
    vinsertf128 ymm2, ymm2, [rsp + 13 * 16], 1
    vinsertf128 ymm2, ymm2, [rsp + 12 * 16], 1
    vinsertf128 ymm2, ymm2, [rsp + 11 * 16], 1
    vinsertf128 ymm2, ymm2, [rsp + 10 * 16], 1
    vinsertf128 ymm2, ymm2, [rsp + 9 * 16], 1
    vinsertf128 ymm2, ymm2, [rsp + 8 * 16], 1
    vinsertf128 ymm2, ymm2, [rsp + 7 * 16], 1
    vinsertf128 ymm2, ymm2, [rsp + 6 * 16], 1
    vinsertf128 ymm2, ymm2, [rsp + 5 * 16], 1
    vinsertf128 ymm2, ymm2, [rsp + 4 * 16], 1
    vinsertf128 ymm2, ymm2, [rsp + 3 * 16], 1
    vinsertf128 ymm2, ymm2, [rsp + 2 * 16], 1
    vinsertf128 ymm2, ymm2, [rsp + 16], 1
    vinsertf128 ymm2, ymm2, [rsp], 1
6:
    add rsp, (29 + 16) * 16

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
7:
    // A spurious signal occurred. Signals are still disabled here, but will need to be re-enabled.

    // restore flags
    mov rax, fs:[0] // load FS base
    // TODO: Use lahf/sahf rather than pushfq/popfq?
    lea rsp, [rax + {tcb_sc_off} + {sc_saved_rflags}]
    popfq

    // restore stack
    mov rsp, fs:[{tcb_sa_off} + {sa_tmp_rsp}]

    // move saved RIP away from control, allowing arch_pre to save us if interrupted.
    mov rax, fs:[{tcb_sc_off} + {sc_saved_rip}]
    mov fs:[{tcb_sa_off} + {sa_tmp_rip}], rax

    // restore regs
    mov rax, fs:[{tcb_sa_off} + {sa_tmp_rax}]
    mov rdx, fs:[{tcb_sa_off} + {sa_tmp_rdx}]

    // Re-enable signals. This code can be interrupted after this signal, so we need to define
    // 'crit_third'.
    and qword ptr fs:[{tcb_sc_off} + {sc_control}], ~1

    .globl __relibc_internal_sigentry_crit_third
__relibc_internal_sigentry_crit_third:
    jmp qword ptr fs:[{tcb_sa_off} + {sa_tmp_rip}]
"] <= [
    inner = sym inner_c,
    sa_tmp_rip = const offset_of!(SigArea, tmp_rip),
    sa_tmp_rsp = const offset_of!(SigArea, tmp_rsp),
    sa_tmp_rax = const offset_of!(SigArea, tmp_rax),
    sa_tmp_rdx = const offset_of!(SigArea, tmp_rdx),
    sa_tmp_rdi = const offset_of!(SigArea, tmp_rdi),
    sa_tmp_rsi = const offset_of!(SigArea, tmp_rsi),
    sa_tmp_ptr = const offset_of!(SigArea, tmp_ptr),
    sa_altstack_top = const offset_of!(SigArea, altstack_top),
    sa_altstack_bottom = const offset_of!(SigArea, altstack_bottom),
    sc_saved_rflags = const offset_of!(Sigcontrol, saved_archdep_reg),
    sc_saved_rip = const offset_of!(Sigcontrol, saved_ip),
    sc_word = const offset_of!(Sigcontrol, word),
    sc_control = const offset_of!(Sigcontrol, control_flags),
    tcb_sa_off = const offset_of!(crate::Tcb, os_specific) + offset_of!(RtSigarea, arch),
    tcb_sc_off = const offset_of!(crate::Tcb, os_specific) + offset_of!(RtSigarea, control),
    pctl_off_actions = const offset_of!(SigProcControl, actions),
    pctl_off_pending = const offset_of!(SigProcControl, pending),
    pctl = sym PROC_CONTROL_STRUCT,
    supports_avx = sym SUPPORTS_AVX,
    REDZONE_SIZE = const 128,
    STACK_ALIGN = const 16,
    SA_ONSTACK_BIT = const 58, // (1 << 58) >> 32 = 0x0400_0000
    SYS_SIGDEQUEUE = const syscall::SYS_SIGDEQUEUE,
]);

extern "C" {
    fn __relibc_internal_sigentry_crit_first();
    fn __relibc_internal_sigentry_crit_second();
    fn __relibc_internal_sigentry_crit_third();
}
/// Fixes some edge cases, and calculates the value for uc_stack.
pub unsafe fn arch_pre(stack: &mut SigStack, area: &mut SigArea) -> PosixStackt {
    // It is impossible to update RSP and RIP atomically on x86_64, without using IRETQ, which is
    // almost as slow as calling a SIGRETURN syscall would be. Instead, we abuse the fact that
    // signals are disabled in the prologue of the signal trampoline, which allows us to emulate
    // atomicity inside the critical section, consisting of one instruction at 'crit_first', one at
    // 'crit_second', and one at 'crit_third', see asm.

    if stack.regs.rip == __relibc_internal_sigentry_crit_first as usize {
        // Reexecute pop rsp and jump steps. This case needs to be different from the one below,
        // since rsp has not been overwritten with the previous context's stack, just yet. At this
        // point, we know [rsp+0] contains the saved RSP, and [rsp-8] contains the saved RIP.
        let stack_ptr = stack.regs.rsp as *const usize;
        stack.regs.rsp = stack_ptr.read();
        stack.regs.rip = stack_ptr.sub(1).read();
    } else if stack.regs.rip == __relibc_internal_sigentry_crit_second as usize
        || stack.regs.rip == __relibc_internal_sigentry_crit_third as usize
    {
        // Almost finished, just reexecute the jump before tmp_rip is overwritten by this
        // deeper-level signal.
        stack.regs.rip = area.tmp_rip;
    }

    PosixStackt {
        sp: stack.regs.rsp as *mut (),
        size: 0,  // TODO
        flags: 0, // TODO
    }
}

pub(crate) static SUPPORTS_AVX: AtomicU8 = AtomicU8::new(0);

// __relibc will be prepended to the name, so mangling is fine
#[no_mangle]
pub unsafe fn manually_enter_trampoline() {
    let c = &Tcb::current().unwrap().os_specific.control;
    c.control_flags.store(
        c.control_flags.load(Ordering::Relaxed) | syscall::flag::INHIBIT_DELIVERY.bits(),
        Ordering::Release,
    );
    c.saved_archdep_reg.set(0); // TODO: Just reset DF on x86?

    core::arch::asm!("
        lea rax, [rip + 2f]
        mov fs:[{tcb_sc_off} + {sc_saved_rip}], rax
        jmp __relibc_internal_sigentry
    2:
    ",
        out("rax") _,
        tcb_sc_off = const offset_of!(crate::Tcb, os_specific) + offset_of!(RtSigarea, control),
        sc_saved_rip = const offset_of!(Sigcontrol, saved_ip),
    );
}

/// Get current stack pointer, weak granularity guarantees.
pub fn current_sp() -> usize {
    let sp: usize;
    unsafe {
        core::arch::asm!("mov {}, rsp", out(reg) sp);
    }
    sp
}
