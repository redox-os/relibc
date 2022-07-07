use core::arch::global_asm;
use core::mem::size_of;

use alloc::boxed::Box;
use alloc::vec::Vec;

use syscall::data::Map;
use syscall::flag::{MapFlags, O_CLOEXEC};
use syscall::error::{Error, Result, EINVAL, ENAMETOOLONG};
use syscall::SIGCONT;

use super::extra::{create_set_addr_space_buf, FdGuard};

fn new_context() -> Result<(FdGuard, usize)> {
    // Create a new context (fields such as uid/gid will be inherited from the current context).
    let fd = FdGuard::new(syscall::open("thisproc:new/open_via_dup", O_CLOEXEC)?);

    // Extract pid.
    let mut buffer = [0_u8; 64];
    let len = syscall::fpath(*fd, &mut buffer)?;
    let buffer = buffer.get(..len).ok_or(Error::new(ENAMETOOLONG))?;

    let colon_idx = buffer.iter().position(|c| *c == b':').ok_or(Error::new(EINVAL))?;
    let slash_idx = buffer.iter().skip(colon_idx).position(|c| *c == b'/').ok_or(Error::new(EINVAL))? + colon_idx;
    let pid_bytes = buffer.get(colon_idx + 1..slash_idx).ok_or(Error::new(EINVAL))?;
    let pid_str = core::str::from_utf8(pid_bytes).map_err(|_| Error::new(EINVAL))?;
    let pid = pid_str.parse::<usize>().map_err(|_| Error::new(EINVAL))?;

    Ok((fd, pid))
}

fn copy_str(cur_pid_fd: usize, new_pid_fd: usize, key: &str) -> Result<()> {
    let cur_name_fd = FdGuard::new(syscall::dup(cur_pid_fd, key.as_bytes())?);
    let new_name_fd = FdGuard::new(syscall::dup(new_pid_fd, key.as_bytes())?);

    let mut buf = [0_u8; 256];
    let len = syscall::read(*cur_name_fd, &mut buf)?;
    let buf = buf.get(..len).ok_or(Error::new(ENAMETOOLONG))?;

    syscall::write(*new_name_fd, &buf)?;

    Ok(())
}
#[cfg(target_arch = "x86_64")]
fn copy_float_env_regs(cur_pid_fd: usize, new_pid_fd: usize) -> Result<()> {
    // Copy environment registers.
    {
        let cur_env_regs_fd = FdGuard::new(syscall::dup(cur_pid_fd, b"regs/env")?);
        let new_env_regs_fd = FdGuard::new(syscall::dup(new_pid_fd, b"regs/env")?);

        let mut env_regs = syscall::EnvRegisters::default();
        let _ = syscall::read(*cur_env_regs_fd, &mut env_regs)?;
        let _ = syscall::write(*new_env_regs_fd, &env_regs)?;
    }
    // Copy float registers.
    {
        let cur_float_regs_fd = FdGuard::new(syscall::dup(cur_pid_fd, b"regs/float")?);
        let new_float_regs_fd = FdGuard::new(syscall::dup(new_pid_fd, b"regs/float")?);

        let mut float_regs = syscall::FloatRegisters::default();
        let _ = syscall::read(*cur_float_regs_fd, &mut float_regs)?;
        let _ = syscall::write(*new_float_regs_fd, &float_regs)?;
    }

    Ok(())
}

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

        let buf = create_set_addr_space_buf(*cur_addr_space_fd, pte_clone_ret as usize, stack as usize);
        let _ = syscall::write(*new_addr_space_sel_fd, &buf)?;
    }

    // Reuse file table
    {
        let cur_filetable_fd = FdGuard::new(syscall::dup(*cur_pid_fd, b"filetable")?);
        let new_filetable_sel_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"current-filetable")?);

        let _ = syscall::write(*new_filetable_sel_fd, &usize::to_ne_bytes(*cur_filetable_fd))?;
    }

    copy_float_env_regs(*cur_pid_fd, *new_pid_fd)?;

    // Unblock context. 
    syscall::kill(new_pid, SIGCONT)?;

    Ok(0)
}
/// Spawns a new context which will not share the same address space as the current one. File
/// descriptors from other schemes are reobtained with `dup`, and grants referencing such file
/// descriptors are reobtained through `fmap`. Other mappings are kept but duplicated using CoW.
pub fn fork_impl() -> Result<usize> {
    unsafe {
        Error::demux(fork_wrapper())
    }
}

fn fork_inner(initial_rsp: *mut usize) -> Result<usize> {
    let (cur_filetable_fd, new_pid_fd, new_pid);

    {
        let cur_pid_fd = FdGuard::new(syscall::open("thisproc:current/open_via_dup", O_CLOEXEC)?);
        (new_pid_fd, new_pid) = new_context()?;

        // Do not allocate new signal stack, but copy existing address (all memory will be re-mapped
        // CoW later).
        {
            let cur_sigstack_fd = FdGuard::new(syscall::dup(*cur_pid_fd, b"sigstack")?);
            let new_sigstack_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"sigstack")?);

            let mut sigstack_buf = usize::to_ne_bytes(0);

            let _ = syscall::read(*cur_sigstack_fd, &mut sigstack_buf);
            let _ = syscall::write(*new_sigstack_fd, &sigstack_buf);
        }

        copy_str(*cur_pid_fd, *new_pid_fd, "name")?;
        copy_str(*cur_pid_fd, *new_pid_fd, "cwd")?;

        // Copy existing files into new file table, but do not reuse the same file table (i.e. new
        // parent FDs will not show up for the child).
        {
            cur_filetable_fd = FdGuard::new(syscall::dup(*cur_pid_fd, b"filetable")?);

            // This must be done before the address space is copied.
            unsafe {
                initial_rsp.write(*cur_filetable_fd);
                initial_rsp.add(1).write(*new_pid_fd);
            }
        }

        // CoW-duplicate address space.
        {
            let cur_addr_space_fd = FdGuard::new(syscall::dup(*cur_pid_fd, b"addrspace")?);

            // FIXME: Find mappings which use external file descriptors

            let new_addr_space_fd = FdGuard::new(syscall::dup(*cur_addr_space_fd, b"exclusive")?);

            let mut buf = vec! [0_u8; 4096];
            let mut bytes_read = 0;

            loop {
                let new_bytes_read = syscall::read(*cur_addr_space_fd, &mut buf[bytes_read..])?;

                if new_bytes_read == 0 { break }

                bytes_read += new_bytes_read;
            }
            let bytes = &buf[..bytes_read];

            for struct_bytes in bytes.array_chunks::<{size_of::<usize>() * 4}>() {
                let mut words = struct_bytes.array_chunks::<{size_of::<usize>()}>().copied().map(usize::from_ne_bytes);

                let addr = words.next().unwrap();
                let size = words.next().unwrap();
                let flags = words.next().unwrap();
                let offset = words.next().unwrap();

                if flags & 0x8000_0000 == 0 {
                    continue;
                }
                let map_flags = MapFlags::from_bits_truncate(flags);

                let mapped_address = unsafe {
                    let fd = FdGuard::new(syscall::dup(*cur_addr_space_fd, format!("grant-{:x}", addr).as_bytes())?);
                    syscall::fmap(*fd, &syscall::Map { address: 0, size, flags: map_flags, offset })?
                };

                let mut buf = [0_u8; size_of::<usize>() * 4];
                let mut chunks = buf.array_chunks_mut::<{size_of::<usize>()}>();
                *chunks.next().unwrap() = usize::to_ne_bytes(addr);
                *chunks.next().unwrap() = usize::to_ne_bytes(size);
                *chunks.next().unwrap() = usize::to_ne_bytes(map_flags.bits());
                *chunks.next().unwrap() = usize::to_ne_bytes(mapped_address);

                let _ = syscall::write(*new_addr_space_fd, &buf)?;
            }
            let new_addr_space_sel_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"current-addrspace")?);

            let buf = create_set_addr_space_buf(*new_addr_space_fd, fork_ret as usize, initial_rsp as usize);
            let _ = syscall::write(*new_addr_space_sel_fd, &buf)?;
        }
        copy_float_env_regs(*cur_pid_fd, *new_pid_fd)?;
    }
    // Copy the file table. We do this last to ensure that all previously used file descriptors are
    // closed. The only exception -- the filetable selection fd and the current filetable fd --
    // will be closed by the child process.
    {
        // TODO: Use cross_scheme_links or something similar to avoid copying the file table in the
        // kernel.
        let new_filetable_fd = FdGuard::new(syscall::dup(*cur_filetable_fd, b"copy")?);
        let new_filetable_sel_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"current-filetable")?);
        let _ = syscall::write(*new_filetable_sel_fd, &usize::to_ne_bytes(*new_filetable_fd));
    }

    // Unblock context.
    syscall::kill(new_pid, SIGCONT)?;

    Ok(new_pid)
}
#[no_mangle]
unsafe extern "sysv64" fn __relibc_internal_fork_impl(initial_rsp: *mut usize) -> usize {
    Error::mux(fork_inner(initial_rsp))
}
#[no_mangle]
unsafe extern "sysv64" fn __relibc_internal_fork_hook(cur_filetable_fd: usize, new_pid_fd: usize) {
    let _ = syscall::close(cur_filetable_fd);
    let _ = syscall::close(new_pid_fd);
}

#[no_mangle]
core::arch::global_asm!("
    .p2align 6
    .globl fork_wrapper
    .type fork_wrapper, @function
fork_wrapper:
    push rbp
    mov rbp, rsp

    push rbx
    push rbp
    push r12
    push r13
    push r14
    push r15

    sub rsp, 16

    mov rdi, rsp
    call __relibc_internal_fork_impl
    jmp 2f

fork_ret:
    mov rdi, [rsp]
    mov rsi, [rsp + 8]
    call __relibc_internal_fork_hook
    xor rax, rax
2:
    add rsp, 16
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx

    pop rbp
    ret
    .size fork_wrapper, . - fork_wrapper

    .globl pte_clone_ret
    .type pte_clone_ret, @function
pte_clone_ret:

    # Load registers
    pop rax
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop r8
    pop r9

    # Call entry point
    call rax

    ret
    .size pte_clone_ret, . - pte_clone_ret
");

extern "sysv64" {
    fn fork_wrapper() -> usize;
    fn fork_ret();
    fn pte_clone_ret();
}
