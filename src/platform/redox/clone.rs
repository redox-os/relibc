use syscall::{
    data::{Map, Sighandler},
    error::Result,
    flag::{MapFlags, O_CLOEXEC, SIGCONT},
};

use super::extra::{create_set_addr_space_buf, FdGuard};

pub use redox_exec::*;

/// Spawns a new context sharing the same address space as the current one (i.e. a new thread).
pub unsafe fn rlct_clone_impl(stack: *mut usize) -> Result<usize> {
    let cur_pid_fd = FdGuard::new(syscall::open("thisproc:current/open_via_dup", O_CLOEXEC)?);
    let (new_pid_fd, new_pid) = new_context()?;

    // Setup the sighandler
    {
        let new_sighandler_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"sighandler")?);

        let _ = syscall::write(*new_sighandler_fd, &Sighandler {
            altstack_base: 0,
            altstack_size: 0,
            handler: crate::platform::sys::signal::__relibc_internal_sighandler as usize,
        })?;
    }
    // Reuse sigmask
    {
        let cur_sigmask_fd = FdGuard::new(syscall::dup(*cur_pid_fd, b"sigmask")?);
        let new_sigmask_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"sigmask")?);

        let mut buf = 0_u64.to_ne_bytes();

        let _ = syscall::read(*cur_sigmask_fd, &mut buf)?;
        let _ = syscall::write(*new_sigmask_fd, &buf);
    }
    // Reuse sigactions (on Linux, CLONE_THREAD requires CLONE_SIGHAND which implies the sigactions
    // table is reused).
    {
        let cur_sigaction_fd = FdGuard::new(syscall::dup(*cur_pid_fd, b"sigactions")?);
        let new_sigaction_sel_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"current-sigactions")?);

        let _ = syscall::write(
            *new_sigaction_sel_fd,
            &usize::to_ne_bytes(*cur_sigaction_fd),
        )?;
    }


    copy_str(*cur_pid_fd, *new_pid_fd, "name")?;

    // Reuse existing address space
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

    // Reuse file table
    {
        let cur_filetable_fd = FdGuard::new(syscall::dup(*cur_pid_fd, b"filetable")?);
        let new_filetable_sel_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"current-filetable")?);

        let _ = syscall::write(
            *new_filetable_sel_fd,
            &usize::to_ne_bytes(*cur_filetable_fd),
        )?;
    }

    copy_env_regs(*cur_pid_fd, *new_pid_fd)?;

    // Unblock context.
    syscall::kill(new_pid, SIGCONT)?;
    let _ = syscall::waitpid(
        new_pid,
        &mut 0,
        syscall::WUNTRACED | syscall::WCONTINUED | syscall::WNOHANG,
    );

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
    ldp x0, x8, [sp], #16
    ldp x2, x1, [sp], #16
    ldp x4, x3, [sp], #16
    ldr x5, [sp], #16

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
