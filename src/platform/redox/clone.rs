use core::mem::size_of;

use syscall::{
    data::Map,
    error::Result,
    flag::{MapFlags, O_CLOEXEC},
    SetSighandlerData, SIGCONT,
};

use super::{
    extra::{create_set_addr_space_buf, FdGuard},
    signal::sighandler_function,
};

pub use redox_exec::*;

/// Spawns a new context sharing the same address space as the current one (i.e. a new thread).
pub unsafe fn rlct_clone_impl(stack: *mut usize) -> Result<usize> {
    let cur_pid_fd = FdGuard::new(syscall::open("thisproc:current/open_via_dup", O_CLOEXEC)?);
    let (new_pid_fd, new_pid) = new_context()?;

    copy_str(*cur_pid_fd, *new_pid_fd, "name")?;

    // Inherit existing address space
    {
        let cur_addr_space_fd = FdGuard::new(syscall::dup(*cur_pid_fd, b"addrspace")?);
        let new_addr_space_sel_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"current-addrspace")?);

        let buf = create_set_addr_space_buf(
            *cur_addr_space_fd,
            __relibc_internal_rlct_clone_ret as usize,
            stack as usize,
        );
        let _ = syscall::write(*new_addr_space_sel_fd, &buf)?;
    }

    // Inherit file table
    {
        let cur_filetable_fd = FdGuard::new(syscall::dup(*cur_pid_fd, b"filetable")?);
        let new_filetable_sel_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"current-filetable")?);

        let _ = syscall::write(
            *new_filetable_sel_fd,
            &usize::to_ne_bytes(*cur_filetable_fd),
        )?;
    }

    // Inherit sigactions (on Linux, CLONE_THREAD requires CLONE_SIGHAND which implies the sigactions
    // table is reused).
    {
        let cur_sigaction_fd = FdGuard::new(syscall::dup(*cur_pid_fd, b"sigactions")?);
        let new_sigaction_sel_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"current-sigactions")?);

        let _ = syscall::write(
            *new_sigaction_sel_fd,
            &usize::to_ne_bytes(*cur_sigaction_fd),
        )?;
    }
    // Inherit sighandler, but not the sigaltstack.
    {
        let new_sighandler_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"sighandler")?);
        let data = SetSighandlerData {
            entry: sighandler_function(),
            altstack_base: 0,
            altstack_len: 0,
        };
        let _ = syscall::write(*new_sighandler_fd, &data)?;
    }

    // Sigprocmask starts as "block all", and is initialized when the thread has actually returned
    // from clone_ret.

    // TODO: Should some of these registers be inherited?
    //copy_env_regs(*cur_pid_fd, *new_pid_fd)?;

    // Unblock context.
    let start_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"start")?);
    let _ = syscall::write(*start_fd, &[0])?;

    Ok(new_pid)
}

extern "C" {
    fn __relibc_internal_rlct_clone_ret();
}

#[cfg(target_arch = "aarch64")]
core::arch::global_asm!(
    "
    .globl __relibc_internal_rlct_clone_ret
    .type __relibc_internal_rlct_clone_ret, @function
    .p2align 6
__relibc_internal_rlct_clone_ret:
    # Load registers
    ldp x8, x0, [sp], #16
    ldp x1, x2, [sp], #16
    ldp x3, x4, [sp], #16

    # Call entry point
    blr x8

    ret
    .size __relibc_internal_rlct_clone_ret, . - __relibc_internal_rlct_clone_ret
"
);

#[cfg(target_arch = "x86")]
core::arch::global_asm!(
    "
    .globl __relibc_internal_rlct_clone_ret
    .type __relibc_internal_rlct_clone_ret, @function
    .p2align 6
__relibc_internal_rlct_clone_ret:
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
    .size __relibc_internal_rlct_clone_ret, . - __relibc_internal_rlct_clone_ret
"
);

#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    "
    .globl __relibc_internal_rlct_clone_ret
    .type __relibc_internal_rlct_clone_ret, @function
    .p2align 6
__relibc_internal_rlct_clone_ret:
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
    .size __relibc_internal_rlct_clone_ret, . - __relibc_internal_rlct_clone_ret
"
);
