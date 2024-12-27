use core::mem::size_of;

use syscall::{Result, O_CLOEXEC};

use crate::{arch::*, proc::*, signal::tmp_disable_signals, RtTcb};

/// Spawns a new context sharing the same address space as the current one (i.e. a new thread).
pub unsafe fn rlct_clone_impl(stack: *mut usize) -> Result<FdGuard> {
    // FIXME
    let cur_thr_fd = RtTcb::current().thread_fd();
    let new_thr_fd = FdGuard::new(syscall::open(
        "/scheme/thisproc/new-thread/open_via_dup",
        O_CLOEXEC,
    )?);

    copy_str(**cur_thr_fd, *new_thr_fd, "name")?;

    // Inherit existing address space
    {
        let cur_addr_space_fd = FdGuard::new(syscall::dup(**cur_thr_fd, b"addrspace")?);
        let new_addr_space_sel_fd = FdGuard::new(syscall::dup(*new_thr_fd, b"current-addrspace")?);

        let buf = create_set_addr_space_buf(
            *cur_addr_space_fd,
            __relibc_internal_rlct_clone_ret as usize,
            stack as usize,
        );
        let _ = syscall::write(*new_addr_space_sel_fd, &buf)?;
    }

    // Inherit reference to file table
    {
        let cur_filetable_fd = FdGuard::new(syscall::dup(**cur_thr_fd, b"filetable")?);
        let new_filetable_sel_fd = FdGuard::new(syscall::dup(*new_thr_fd, b"current-filetable")?);

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
    let start_fd = FdGuard::new(syscall::dup(*new_thr_fd, b"start")?);
    let _ = syscall::write(*start_fd, &[0])?;

    Ok(new_thr_fd)
}

pub unsafe fn exit_this_thread(stack_base: *mut (), stack_size: usize) -> ! {
    let _guard = tmp_disable_signals();

    let tcb = RtTcb::current();
    let thread_fd = tcb.thread_fd();

    let _ = syscall::funmap(tcb as *const RtTcb as usize, syscall::PAGE_SIZE);

    // TODO: modify interface so it writes directly to the thread fd?
    let status_fd = syscall::dup(**thread_fd, b"status").unwrap();
    let mut buf = [0; size_of::<usize>() * 3];
    plain::slice_from_mut_bytes(&mut buf)
        .unwrap()
        .copy_from_slice(&[usize::MAX, stack_base as usize, stack_size]);
    syscall::write(status_fd, &buf).unwrap();
    unreachable!()
}
