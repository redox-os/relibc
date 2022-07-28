use core::arch::global_asm;
use core::mem::size_of;

use alloc::boxed::Box;
use alloc::vec::Vec;

use syscall::data::Map;
use syscall::flag::{MapFlags, O_CLOEXEC};
use syscall::error::{Error, Result, EINVAL, ENAMETOOLONG};
use syscall::SIGCONT;

use super::extra::{create_set_addr_space_buf, FdGuard};

pub use redox_exec::*;

/// Spawns a new context sharing the same address space as the current one (i.e. a new thread).
pub unsafe fn pte_clone_impl(stack: *mut usize) -> Result<usize> {
    let cur_pid_fd = FdGuard::new(syscall::open("thisproc:current/open_via_dup", O_CLOEXEC)?);
    let (new_pid_fd, new_pid) = new_context()?;

    // Allocate a new signal stack.
    {
        let sigstack_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"sigstack")?);

        const SIGSTACK_SIZE: usize = 1024 * 256;

        // TODO: Put sigstack at high addresses?
        let target_sigstack = syscall::fmap(!0, &Map { address: 0, flags: MapFlags::PROT_READ | MapFlags::PROT_WRITE | MapFlags::MAP_PRIVATE, offset: 0, size: SIGSTACK_SIZE })? + SIGSTACK_SIZE;

        let _ = syscall::write(*sigstack_fd, &usize::to_ne_bytes(target_sigstack))?;
    }

    copy_str(*cur_pid_fd, *new_pid_fd, "name")?;
    copy_str(*cur_pid_fd, *new_pid_fd, "cwd")?;

    // Reuse existing address space
    {
        let cur_addr_space_fd = FdGuard::new(syscall::dup(*cur_pid_fd, b"addrspace")?);
        let new_addr_space_sel_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"current-addrspace")?);

        let buf = create_set_addr_space_buf(*cur_addr_space_fd, __relibc_internal_pte_clone_ret as usize, stack as usize);
        let _ = syscall::write(*new_addr_space_sel_fd, &buf)?;
    }

    // Reuse file table
    {
        let cur_filetable_fd = FdGuard::new(syscall::dup(*cur_pid_fd, b"filetable")?);
        let new_filetable_sel_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"current-filetable")?);

        let _ = syscall::write(*new_filetable_sel_fd, &usize::to_ne_bytes(*cur_filetable_fd))?;
    }

    // Reuse sigactions (on Linux, CLONE_THREAD requires CLONE_SIGHAND which implies the sigactions
    // table is reused).
    {
        let cur_sigaction_fd = FdGuard::new(syscall::dup(*cur_pid_fd, b"sigactions")?);
        let new_sigaction_sel_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"current-sigactions")?);

        let _ = syscall::write(*new_sigaction_sel_fd, &usize::to_ne_bytes(*cur_sigaction_fd))?;
    }

    copy_env_regs(*cur_pid_fd, *new_pid_fd)?;

    // Unblock context.
    syscall::kill(new_pid, SIGCONT)?;
    let _ = syscall::waitpid(new_pid, &mut 0, syscall::WUNTRACED | syscall::WCONTINUED);

    Ok(0)
}

#[cfg(target_arch = "x86_64")]
core::arch::global_asm!("
    .globl __relibc_internal_pte_clone_ret
    .type __relibc_internal_pte_clone_ret, @function
    .p2align 6
__relibc_internal_pte_clone_ret:
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
    mov WORD PTR [rsp], 0x031F
    fldcw [rsp]

    add rsp, 8

    # Call entry point
    call rax

    ret
    .size __relibc_internal_pte_clone_ret, . - __relibc_internal_pte_clone_ret
");

#[cfg(target_arch = "x86_64")]
extern "sysv64" {
    fn __relibc_internal_pte_clone_ret();
}
