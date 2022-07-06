use core::{mem, ptr, slice};
use core::arch::global_asm;

use syscall::data::Map;
use syscall::flag::{MapFlags, O_CLOEXEC};
use syscall::error::{Error, Result, EINVAL, ENAMETOOLONG};
use syscall::SIGCONT;

use crate::platform::{sys::e, types::*};

#[no_mangle]
pub unsafe extern "C" fn redox_fpath(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    e(syscall::fpath(
        fd as usize,
        slice::from_raw_parts_mut(buf as *mut u8, count),
    )) as ssize_t
}

#[no_mangle]
pub unsafe extern "C" fn redox_physalloc(size: size_t) -> *mut c_void {
    let res = e(syscall::physalloc(size));
    if res == !0 {
        return ptr::null_mut();
    } else {
        return res as *mut c_void;
    }
}

#[no_mangle]
pub unsafe extern "C" fn redox_physfree(physical_address: *mut c_void, size: size_t) -> c_int {
    e(syscall::physfree(physical_address as usize, size)) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn redox_physmap(
    physical_address: *mut c_void,
    size: size_t,
    flags: c_int,
) -> *mut c_void {
    let res = e(syscall::physmap(
        physical_address as usize,
        size,
        syscall::PhysmapFlags::from_bits(flags as usize).expect("physmap: invalid bit pattern"),
    ));
    if res == !0 {
        return ptr::null_mut();
    } else {
        return res as *mut c_void;
    }
}

#[no_mangle]
pub unsafe extern "C" fn redox_physunmap(virtual_address: *mut c_void) -> c_int {
    e(syscall::physunmap(virtual_address as usize)) as c_int
}

pub struct FdGuard {
    fd: usize,
    taken: bool,
}
impl FdGuard {
    pub fn new(fd: usize) -> Self {
        Self {
            fd, taken: false,
        }
    }
    pub fn take(&mut self) -> usize {
        self.taken = true;
        self.fd
    }
}
impl core::ops::Deref for FdGuard {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.fd
    }
}

impl Drop for FdGuard {
    fn drop(&mut self) {
        if !self.taken {
            let _ = syscall::close(self.fd);
        }
    }
}

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
    syscall::kill(new_pid, SIGCONT);

    Ok(0)
}
fn create_set_addr_space_buf(space: usize, ip: usize, sp: usize) -> [u8; mem::size_of::<usize>() * 3] {
    let mut buf = [0_u8; 3 * mem::size_of::<usize>()];
    let mut chunks = buf.array_chunks_mut::<{mem::size_of::<usize>()}>();
    *chunks.next().unwrap() = usize::to_ne_bytes(space);
    *chunks.next().unwrap() = usize::to_ne_bytes(sp);
    *chunks.next().unwrap() = usize::to_ne_bytes(ip);
    buf
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
    let new_pid = {
        let cur_pid_fd = FdGuard::new(syscall::open("thisproc:current/open_via_dup", O_CLOEXEC)?);
        let (new_pid_fd, new_pid) = new_context()?;

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

        // CoW-duplicate address space.
        {
            let cur_addr_space_fd = FdGuard::new(syscall::dup(*cur_pid_fd, b"addrspace")?);

            // FIXME: Find mappings which use external file descriptors

            let new_addr_space_fd = FdGuard::new(syscall::dup(*cur_addr_space_fd, b"exclusive")?);
            let new_addr_space_sel_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"current-addrspace")?);

            let buf = create_set_addr_space_buf(*new_addr_space_fd, fork_ret as usize, initial_rsp as usize);
            let _ = syscall::write(*new_addr_space_sel_fd, &buf)?;
        }

        // Copy existing files into new file table, but do not reuse the same file table (i.e. new
        // parent FDs will not show up for the child).
        {
            let cur_filetable_fd = FdGuard::new(syscall::dup(*cur_pid_fd, b"filetable")?);
            // TODO: Use cross_scheme_links or something similar to avoid copying the file table in the
            // kernel.
            let new_filetable_fd = FdGuard::new(syscall::dup(*cur_filetable_fd, b"copy")?);
            let new_filetable_sel_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"current-filetable")?);

            let _ = syscall::write(*new_filetable_sel_fd, &usize::to_ne_bytes(*new_filetable_fd));
        }
        copy_float_env_regs(*cur_pid_fd, *new_pid_fd)?;

        new_pid
    };

    // Unblock context.
    syscall::kill(new_pid, SIGCONT);

    Ok(new_pid)
}
#[no_mangle]
unsafe extern "sysv64" fn __relibc_internal_fork_impl(initial_rsp: *mut usize) -> usize {
    Error::mux(fork_inner(initial_rsp))
}

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

    mov rdi, rsp
    call __relibc_internal_fork_impl
    jmp 2f

fork_ret:
    xor rax, rax
2:
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
