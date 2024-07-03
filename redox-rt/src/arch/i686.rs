use core::mem::offset_of;
use core::sync::atomic::Ordering;

use syscall::*;

use crate::proc::{fork_inner, FdGuard};
use crate::signal::{inner_fastcall, PROC_CONTROL_STRUCT, RtSigarea, SigStack};

// Setup a stack starting from the very end of the address space, and then growing downwards.
pub(crate) const STACK_TOP: usize = 1 << 31;
pub(crate) const STACK_SIZE: usize = 1024 * 1024;

#[derive(Debug, Default)]
#[repr(C)]
pub struct SigArea {
    pub altstack_top: usize,
    pub altstack_bottom: usize,
    pub tmp_eip: usize,
    pub tmp_esp: usize,
    pub tmp_eax: usize,
    pub tmp_edx: usize,
    pub disable_signals_depth: u64,
}
#[derive(Debug, Default)]
#[repr(C, align(16))]
pub struct ArchIntRegs {
    pub fxsave: [u16; 29],

    // ensure fxsave region is 16 byte aligned
    pub _pad: [usize; 2], // fxsave "available" +0

    pub ebp: usize, // fxsave "available" +8
    pub esi: usize, // avail +12
    pub edi: usize, // avail +16
    pub ebx: usize, // avail +20
    pub eax: usize, // avail +24
    pub ecx: usize, // avail +28
    pub edx: usize, // avail +32

    pub eflags: usize, // avail +36
    pub eip: usize, // avail +40
    pub esp: usize, // avail +44
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
    // Save some registers
    mov gs:[{tcb_sa_off} + {sa_tmp_esp}], esp
    mov gs:[{tcb_sa_off} + {sa_tmp_eax}], eax
    mov gs:[{tcb_sa_off} + {sa_tmp_edx}], edx

    // Read pending half of first signal. This can be done nonatomically wrt the mask bits, since
    // only this thread is allowed to modify the latter.

    // Read first signal word
    mov eax, gs:[{tcb_sc_off} + {sc_word}]
    and eax, gs:[{tcb_sc_off} + {sc_word} + 4]
    bsf eax, eax
    jnz 2f

    // Read second signal word
    mov eax, gs:[{tcb_sc_off} + {sc_word} + 8]
    and eax, gs:[{tcb_sc_off} + {sc_word} + 12]
    bsf eax, eax
    jz 7f
    add eax, 32
2:
    and esp, -{STACK_ALIGN}

    mov edx, eax
    add edx, edx
    bt dword ptr [{pctl} + {pctl_off_actions} + edx * 8 + 4], 28
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
    push dword ptr gs:[{tcb_sa_off} + {sa_tmp_esp}]
    push dword ptr gs:[{tcb_sc_off} + {sc_saved_eip}]
    push dword ptr gs:[{tcb_sc_off} + {sc_saved_eflags}]

    push dword ptr gs:[{tcb_sa_off} + {sa_tmp_edx}]
    push ecx
    push dword ptr gs:[{tcb_sa_off} + {sa_tmp_eax}]
    push ebx
    push edi
    push esi
    push ebp

    sub esp, 2 * 4 + 29 * 16
    fxsave [esp]

    push eax
    sub esp, 3 * 4

    mov ecx, esp
    call {inner}

    fxrstor [esp + 16]
    add esp, 16 + 29 * 16 + 2 * 4

    pop ebp
    pop esi
    pop edi
    pop ebx
    pop eax
    pop ecx
    pop edx

    popfd
    pop dword ptr gs:[{tcb_sa_off} + {sa_tmp_eip}]

    .globl __relibc_internal_sigentry_crit_first
__relibc_internal_sigentry_crit_first:
    pop esp

    .globl __relibc_internal_sigentry_crit_second
__relibc_internal_sigentry_crit_second:
    jmp dword ptr gs:[{tcb_sa_off} + {sa_tmp_eip}]
7:
    ud2
"] <= [
    inner = sym inner_fastcall,
    sa_tmp_eip = const offset_of!(SigArea, tmp_eip),
    sa_tmp_esp = const offset_of!(SigArea, tmp_esp),
    sa_tmp_eax = const offset_of!(SigArea, tmp_eax),
    sa_tmp_edx = const offset_of!(SigArea, tmp_edx),
    sa_altstack_top = const offset_of!(SigArea, altstack_top),
    sa_altstack_bottom = const offset_of!(SigArea, altstack_bottom),
    sc_saved_eflags = const offset_of!(Sigcontrol, saved_archdep_reg),
    sc_saved_eip = const offset_of!(Sigcontrol, saved_ip),
    sc_word = const offset_of!(Sigcontrol, word),
    tcb_sa_off = const offset_of!(crate::Tcb, os_specific) + offset_of!(RtSigarea, arch),
    tcb_sc_off = const offset_of!(crate::Tcb, os_specific) + offset_of!(RtSigarea, control),
    pctl_off_actions = const offset_of!(SigProcControl, actions),
    pctl = sym PROC_CONTROL_STRUCT,
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
extern "C" {
    fn __relibc_internal_sigentry_crit_first();
    fn __relibc_internal_sigentry_crit_second();
}
pub unsafe fn arch_pre(stack: &mut SigStack, area: &mut SigArea) {
    if stack.regs.eip == __relibc_internal_sigentry_crit_first as usize {
        let stack_ptr = stack.regs.esp as *const usize;
        stack.regs.esp = stack_ptr.read();
        stack.regs.eip = stack_ptr.sub(1).read();
    } else if stack.regs.eip == __relibc_internal_sigentry_crit_second as usize {
        stack.regs.eip = area.tmp_eip;
    }
}
pub unsafe fn manually_enter_trampoline() {
    let c = &crate::Tcb::current().unwrap().os_specific.control;
    c.control_flags.store(c.control_flags.load(Ordering::Relaxed) | syscall::flag::INHIBIT_DELIVERY.bits(), Ordering::Release);
    c.saved_archdep_reg.set(0); // TODO: Just reset DF on x86?

    core::arch::asm!("
        call 2f
        jmp 3f
    2:
        pop dword ptr gs:[{tcb_sc_off} + {sc_saved_eip}]
        jmp __relibc_internal_sigentry
    3:
    ",
        tcb_sc_off = const offset_of!(crate::Tcb, os_specific) + offset_of!(RtSigarea, control),
        sc_saved_eip = const offset_of!(Sigcontrol, saved_ip),
    );
}
/// Get current stack pointer, weak granularity guarantees.
pub fn current_sp() -> usize {
    let sp: usize;
    unsafe {
        core::arch::asm!("mov {}, esp", out(reg) sp);
    }
    sp
}
