use syscall::SetSighandlerData;
use syscall::{Result, O_CLOEXEC};

use crate::arch::*;
use crate::proc::*;
use crate::signal::sighandler_function;

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

    // Inherit reference to file table
    {
        let cur_filetable_fd = FdGuard::new(syscall::dup(*cur_pid_fd, b"filetable")?);
        let new_filetable_sel_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"current-filetable")?);

        let _ = syscall::write(
            *new_filetable_sel_fd,
            &usize::to_ne_bytes(*cur_filetable_fd),
        )?;
    }

    // Since the signal handler is not yet initialized, signals specifically targeting the thread
    // (relibc is only required to implement thread-specific signals that already originate from
    // the same process) will be discarded. Process-specific signals will ignore this new thread,
    // until it has initialized its own signal handler.

    // Unblock context.
    let start_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"start")?);
    let _ = syscall::write(*start_fd, &[0])?;

    Ok(new_pid)
}
