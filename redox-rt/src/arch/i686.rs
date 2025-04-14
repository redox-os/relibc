use core::{cell::SyncUnsafeCell, mem::offset_of, ptr::NonNull, sync::atomic::Ordering};

use syscall::*;

use crate::{
    proc::{fork_inner, FdGuard, ForkArgs},
    protocol::{ProcCall, RtSigInfo},
    signal::{inner_fastcall, PosixStackt, RtSigarea, SigStack, PROC_CONTROL_STRUCT},
    RtTcb,
};

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
    pub tmp_ebx: usize,
    pub tmp_ecx: usize,
    pub tmp_edx: usize,
    pub tmp_edi: usize,
    pub tmp_esi: usize,
    pub tmp_rt_inf: RtSigInfo,
    pub tmp_id_inf: u64,
    pub tmp_mm0: u64,
    pub pctl: usize, // TODO: reference pctl directly
    pub disable_signals_depth: u64,
    pub last_sig_was_restart: bool,
    pub last_sigstack: Option<NonNull<SigStack>>,
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
    pub eip: usize,    // avail +40
    pub esp: usize,    // avail +44
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

unsafe extern "fastcall" fn fork_impl(args: &ForkArgs, initial_rsp: *mut usize) -> usize {
    Error::mux(fork_inner(initial_rsp, args))
}

// TODO: duplicate code with x86_64
unsafe extern "cdecl" fn child_hook(
    cur_filetable_fd: usize,
    new_proc_fd: usize,
    new_thr_fd: usize,
) {
    let _ = syscall::close(cur_filetable_fd);
    crate::child_hook_common(crate::ChildHookCommonArgs {
        new_thr_fd: FdGuard::new(new_thr_fd),
        new_proc_fd: if new_proc_fd == usize::MAX {
            None
        } else {
            Some(FdGuard::new(new_proc_fd))
        },
    });
}

asmfunction!(__relibc_internal_fork_wrapper (usize) -> usize: ["
    mov ecx, [esp+4]

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

    mov edx, esp
    call {fork_impl}

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
    mov gs:[{tcb_sa_off} + {sa_tmp_ecx}], ecx
    mov gs:[{tcb_sa_off} + {sa_tmp_ebx}], ebx
    mov gs:[{tcb_sa_off} + {sa_tmp_edi}], edi
    mov gs:[{tcb_sa_off} + {sa_tmp_esi}], esi
1:
    // Read standard signal word - first for this thread
    mov edx, gs:[{tcb_sc_off} + {sc_word} + 4]
    mov eax, gs:[{tcb_sc_off} + {sc_word}]
    and eax, edx
    bsf eax, eax
    jnz 9f

    mov ecx, gs:[{tcb_sa_off} + {sa_pctl}]

    // Read standard signal word - for the process
    mov eax, [ecx + {pctl_pending}]
    and eax, edx
    bsf eax, eax
    jz 3f

    // Read si_pid and si_uid, atomically.
    movq gs:[{tcb_sa_off} + {sa_tmp_mm0}], mm0
    movq mm0, [ecx + {pctl_sender_infos} + eax * 8]
    movq gs:[{tcb_sa_off} + {sa_tmp_id_inf}], mm0
    movq mm0, gs:[{tcb_sa_off} + {sa_tmp_mm0}]

    // Try clearing the pending bit, otherwise retry if another thread did that first
    lock btr [ecx + {pctl_pending}], eax
    jnc 1b
    jmp 2f
3:
    // Read realtime thread and process signal word together
    mov edx, [ecx + {pctl_pending} + 4]
    mov eax, gs:[{tcb_sc_off} + {sc_word} + 8]
    or eax, edx
    and eax, gs:[{tcb_sc_off} + {sc_word} + 12]
    jz 7f // spurious signal
    bsf eax, eax

    // If thread was specifically targeted, send the signal to it first.
    bt edx, eax
    jc 8f

    // SYS_CALL(fd, payload_base, payload_len, metadata_len, metadata_base)
    // eax      ebx ecx           edx          esi           edi

    mov ebx, [{proc_fd}]
    mov ecx, gs:[0]
    add ecx, {tcb_sa_off} + {sa_tmp_rt_inf}
    lea edx, [eax+32]
    mov [ecx], edx
    mov esi, 1
    mov edi, {proc_call}
    mov eax, {SYS_CALL}
    int 0x80
    test eax, eax
    jnz 1b

    mov eax, ecx
    jmp 2f
8:
    add eax, 32
9:
    // Read si_pid and si_uid, atomically.
    movq gs:[{tcb_sa_off} + {sa_tmp_mm0}], mm0
    movq mm0, gs:[{tcb_sc_off} + {sc_sender_infos} + eax * 8]
    movq gs:[{tcb_sa_off} + {sa_tmp_id_inf}], mm0
    movq mm0, gs:[{tcb_sa_off} + {sa_tmp_mm0}]
    mov edx, eax
    shr edx, 5
    mov ecx, eax
    and ecx, 31
    lock btr gs:[{tcb_sc_off} + {sc_word} + edx * 8], ecx

    add eax, 64
2:
    and esp, -{STACK_ALIGN}

    mov edx, eax
    add edx, edx
    bt dword ptr [{pctl} + {pctl_actions} + edx * 8 + 4], 28
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
    push dword ptr gs:[{tcb_sa_off} + {sa_tmp_ecx}]
    push dword ptr gs:[{tcb_sa_off} + {sa_tmp_eax}]
    push dword ptr gs:[{tcb_sa_off} + {sa_tmp_ebx}]
    push dword ptr gs:[{tcb_sa_off} + {sa_tmp_edi}]
    push dword ptr gs:[{tcb_sa_off} + {sa_tmp_esi}]
    push ebp

    sub esp, 2 * 4 + 29 * 16
    fxsave [esp]

    mov [esp - 4], eax
    sub esp, 48

    mov ecx, esp
    call {inner}

    fxrstor [esp + 48]
    add esp, 48 + 29 * 16 + 2 * 4

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
    mov eax, gs:[0]
    lea esp, [eax + {tcb_sc_off} + {sc_saved_eflags}]
    popfd

    mov esp, gs:[{tcb_sa_off} + {sa_tmp_esp}]

    mov eax, gs:[{tcb_sc_off} + {sc_saved_eip}]
    mov gs:[{tcb_sa_off} + {sa_tmp_eip}], eax

    mov eax, gs:[{tcb_sa_off} + {sa_tmp_eax}]
    mov ecx, gs:[{tcb_sa_off} + {sa_tmp_ecx}]
    mov edx, gs:[{tcb_sa_off} + {sa_tmp_edx}]

    and dword ptr gs:[{tcb_sc_off} + {sc_control}], ~1
    .globl __relibc_internal_sigentry_crit_third
__relibc_internal_sigentry_crit_third:
    jmp dword ptr gs:[{tcb_sa_off} + {sa_tmp_eip}]
"] <= [
    inner = sym inner_fastcall,
    sa_tmp_eip = const offset_of!(SigArea, tmp_eip),
    sa_tmp_esp = const offset_of!(SigArea, tmp_esp),
    sa_tmp_eax = const offset_of!(SigArea, tmp_eax),
    sa_tmp_ebx = const offset_of!(SigArea, tmp_ebx),
    sa_tmp_ecx = const offset_of!(SigArea, tmp_ecx),
    sa_tmp_edx = const offset_of!(SigArea, tmp_edx),
    sa_tmp_edi = const offset_of!(SigArea, tmp_edi),
    sa_tmp_esi = const offset_of!(SigArea, tmp_esi),
    sa_tmp_mm0 = const offset_of!(SigArea, tmp_mm0),
    sa_tmp_rt_inf = const offset_of!(SigArea, tmp_rt_inf),
    sa_tmp_id_inf = const offset_of!(SigArea, tmp_id_inf),
    sa_altstack_top = const offset_of!(SigArea, altstack_top),
    sa_altstack_bottom = const offset_of!(SigArea, altstack_bottom),
    sa_pctl = const offset_of!(SigArea, pctl),
    sc_control = const offset_of!(Sigcontrol, control_flags),
    sc_saved_eflags = const offset_of!(Sigcontrol, saved_archdep_reg),
    sc_saved_eip = const offset_of!(Sigcontrol, saved_ip),
    sc_word = const offset_of!(Sigcontrol, word),
    sc_sender_infos = const offset_of!(Sigcontrol, sender_infos),
    tcb_sa_off = const offset_of!(crate::Tcb, os_specific) + offset_of!(RtSigarea, arch),
    tcb_sc_off = const offset_of!(crate::Tcb, os_specific) + offset_of!(RtSigarea, control),
    pctl_actions = const offset_of!(SigProcControl, actions),
    pctl_sender_infos = const offset_of!(SigProcControl, sender_infos),
    pctl_pending = const offset_of!(SigProcControl, pending),
    pctl = sym PROC_CONTROL_STRUCT,
    proc_fd = sym PROC_FD,
    proc_call = sym PROC_CALL,
    STACK_ALIGN = const 16,
    SYS_CALL = const syscall::SYS_CALL,
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
    fn __relibc_internal_sigentry_crit_third();
}
pub unsafe fn arch_pre(stack: &mut SigStack, area: &mut SigArea) -> PosixStackt {
    if stack.regs.eip == __relibc_internal_sigentry_crit_first as usize {
        let stack_ptr = stack.regs.esp as *const usize;
        stack.regs.esp = stack_ptr.read();
        stack.regs.eip = stack_ptr.sub(1).read();
    } else if stack.regs.eip == __relibc_internal_sigentry_crit_second as usize
        || stack.regs.eip == __relibc_internal_sigentry_crit_third as usize
    {
        stack.regs.eip = area.tmp_eip;
    }
    PosixStackt {
        sp: stack.regs.esp as *mut (),
        size: 0,  // TODO
        flags: 0, // TODO
    }
}
#[no_mangle]
pub unsafe fn manually_enter_trampoline() {
    let c = &crate::Tcb::current().unwrap().os_specific.control;
    c.control_flags.store(
        c.control_flags.load(Ordering::Relaxed) | syscall::flag::INHIBIT_DELIVERY.bits(),
        Ordering::Release,
    );
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

pub static PROC_FD: SyncUnsafeCell<usize> = SyncUnsafeCell::new(usize::MAX);
static PROC_CALL: [usize; 1] = [ProcCall::Sigdeq as usize];
