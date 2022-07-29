use syscall::error::*;

use crate::{FdGuard, fork_inner};

// Setup a stack starting from the very end of the address space, and then growing downwards.
pub(crate) const STACK_TOP: usize = 1 << 47;
pub(crate) const STACK_SIZE: usize = 1024 * 1024;

/// Deactive TLS, used before exec() on Redox to not trick target executable into thinking TLS
/// is already initialized as if it was a thread.
pub unsafe fn deactivate_tcb(open_via_dup: usize) -> Result<()> {
    //TODO: aarch64
    Err(Error::new(ENOSYS))
}

pub fn copy_env_regs(cur_pid_fd: usize, new_pid_fd: usize) -> Result<()> {
    //TODO: aarch64
    Err(Error::new(ENOSYS))
}

#[no_mangle]
unsafe extern "C" fn __relibc_internal_fork_impl(initial_rsp: *mut usize) -> usize {
    Error::mux(fork_inner(initial_rsp))
}

#[no_mangle]
unsafe extern "C" fn __relibc_internal_fork_hook(cur_filetable_fd: usize, new_pid_fd: usize) {
    let _ = syscall::close(cur_filetable_fd);
    let _ = syscall::close(new_pid_fd);
}

//TODO: aarch64
core::arch::global_asm!("
    .p2align 6
    .globl __relibc_internal_fork_wrapper
    .type __relibc_internal_fork_wrapper, @function
__relibc_internal_fork_wrapper:
        b __relibc_internal_fork_wrapper

    .size __relibc_internal_fork_wrapper, . - __relibc_internal_fork_wrapper

    .p2align 6
    .globl __relibc_internal_fork_ret
    .type __relibc_internal_fork_ret, @function
__relibc_internal_fork_ret:
        b __relibc_internal_fork_ret

    .size __relibc_internal_fork_ret, . - __relibc_internal_fork_ret"
);

extern "C" {
    pub(crate) fn __relibc_internal_fork_wrapper() -> usize;
    pub(crate) fn __relibc_internal_fork_ret();
}
