use core::mem::size_of;

use syscall::Result;

use crate::{RtTcb, arch::*, proc::*, signal::tmp_disable_signals, static_proc_info};

/// Spawns a new context sharing the same address space as the current one (i.e. a new thread).
pub unsafe fn rlct_clone_impl(stack: *mut usize, tcb: &RtTcb) -> Result<usize> {
    let proc_info = static_proc_info();
    let cur_proc_fd = proc_info.proc_fd.as_ref().unwrap();

    let cur_thr_fd = RtTcb::current().thread_fd();
    let new_thr_fd = cur_proc_fd.dup(b"new-thread")?.to_upper().unwrap();

    // Inherit existing address space
    {
        let cur_addr_space_fd = cur_thr_fd.dup(b"addrspace")?;
        let new_addr_space_sel_fd = new_thr_fd.dup(b"current-addrspace")?;

        let buf = create_set_addr_space_buf(
            cur_addr_space_fd.as_raw_fd(),
            __relibc_internal_rlct_clone_ret as *const () as usize,
            stack as usize,
        );
        new_addr_space_sel_fd.write(&buf)?;
    }

    // Inherit reference to file table
    {
        let cur_filetable_fd = cur_thr_fd.dup(b"filetable")?;
        let new_filetable_sel_fd = new_thr_fd.dup(b"current-filetable")?;

        new_filetable_sel_fd.write(&usize::to_ne_bytes(cur_filetable_fd.as_raw_fd()))?;
    }

    // Since the signal handler is not yet initialized, signals specifically targeting the thread
    // (relibc is only required to implement thread-specific signals that already originate from
    // the same process) will be discarded. Process-specific signals will ignore this new thread,
    // until it has initialized its own signal handler.

    let start_fd = new_thr_fd.dup(b"start")?;

    let fd = new_thr_fd.as_raw_fd();
    unsafe {
        tcb.thr_fd.get().write(Some(new_thr_fd));
    }

    // Unblock context.
    start_fd.write(&[0])?;

    Ok(fd)
}

pub unsafe fn exit_this_thread(stack_base: *mut (), stack_size: usize) -> ! {
    let _guard = tmp_disable_signals();

    let tcb = RtTcb::current();
    // TODO: modify interface so it writes directly to the thread fd?
    let status_fd = tcb.thread_fd().dup(b"status").unwrap();

    let _ = unsafe { syscall::funmap(tcb as *const RtTcb as usize, syscall::PAGE_SIZE) };

    let mut buf = [0; size_of::<usize>() * 3];
    plain::slice_from_mut_bytes(&mut buf)
        .unwrap()
        .copy_from_slice(&[usize::MAX, stack_base as usize, stack_size]);
    // TODO: SYS_CALL w/CONSUME
    status_fd.write(&buf).unwrap();
    unreachable!()
}
