use core::convert::TryFrom;
use super::extra::{create_set_addr_space_buf, FdGuard};

use alloc::{
    collections::{btree_map::Entry, BTreeMap},
    vec::Vec,
};

use syscall::{
    data::ExecMemRange,
    error::*,
    flag::{AT_ENTRY, AT_NULL, AT_PHDR, AT_PHENT, AT_PHNUM, MapFlags, O_WRONLY, SEEK_SET},
};

use crate::fs::File;

fn read_all(fd: usize, offset: Option<u64>, buf: &mut [u8]) -> Result<()> {
    if let Some(offset) = offset {
        syscall::lseek(fd, offset as isize, SEEK_SET)?;
    }

    let mut total_bytes_read = 0;

    while total_bytes_read < buf.len() {
        total_bytes_read += match syscall::read(fd, &mut buf[total_bytes_read..])? {
            0 => return Err(Error::new(ENOEXEC)),
            bytes_read => bytes_read,
        }
    }
    Ok(())
}
fn write_all(fd: usize, offset: Option<u64>, buf: &[u8]) -> Result<()> {
    if let Some(offset) = offset {
        syscall::lseek(fd, offset as isize, SEEK_SET)?;
    }

    let mut total_bytes_written = 0;

    while total_bytes_written < buf.len() {
        total_bytes_written += match syscall::write(fd, &buf[total_bytes_written..])? {
            0 => return Err(Error::new(EIO)),
            bytes_written => bytes_written,
        }
    }
    Ok(())
}

fn find_free_target_addr(tree: &BTreeMap<usize, usize>, size: usize) -> Option<usize> {
    let mut iterator = tree.iter().peekable();

    // Ignore the space between zero and the first region, to avoid null pointers.
    while let Some((cur_address, entry_size)) = iterator.next() {
        let end = *cur_address + entry_size;

        if let Some((next_address, _)) = iterator.peek() {
            if **next_address - end > size {
                return Some(end);
            }
        }
        // No need to check last entry, since the stack will always be put at the highest
        // possible address.
    }

    None
}

#[cfg(target_arch = "x86_64")]
const PAGE_SIZE: usize = 4096;

const FD_ANONYMOUS: usize = !0;

pub fn fexec_impl(file: File, path: &[u8], args: &[&[u8]], envs: &[&[u8]], args_envs_size_without_nul: usize) -> Result<usize> {
    let addrspace_selection_fd = fexec_impl_inner(file, path, args, envs, args_envs_size_without_nul)?;

    // Dropping this FD will cause the address space switch.
    drop(addrspace_selection_fd);

    unreachable!();
}
fn fexec_impl_inner(file: File, path: &[u8], args: &[&[u8]], envs: &[&[u8]], args_envs_size_without_nul: usize) -> Result<FdGuard> {
    use goblin::elf64::{header::Header, program_header::program_header64::{ProgramHeader, PT_LOAD, PF_W, PF_X}};

    let fd = *file as usize;
    let total_args_envs_size = args_envs_size_without_nul + args.len() + envs.len();

    // Here, we do the minimum part of loading an application, which is what the kernel used to do.
    // We load the executable into memory (albeit at different offsets in this executable), fix
    // some misalignments, and then execute the SYS_EXEC syscall to replace the program memory
    // entirely.

    let mut header_bytes = [0_u8; core::mem::size_of::<Header>()];
    read_all(fd, Some(0), &mut header_bytes)?;
    let header = Header::from_bytes(&header_bytes);

    let grants_fd = {
        let current_addrspace_fd = FdGuard::new(syscall::open("thisproc:current/addrspace", 0)?);
        FdGuard::new(syscall::dup(*current_addrspace_fd, b"empty")?)
    };
    let memory_fd = FdGuard::new(syscall::dup(*grants_fd, b"mem")?);

    let instruction_ptr = usize::try_from(header.e_entry).map_err(|_| Error::new(ENOEXEC))?;

    // Never allow more than 1 MiB of program headers. TODO: Capabilities again?
    const MAX_PH_SIZE: usize = 1024 * 1024;
    let phentsize = u64::from(header.e_phentsize) as usize;
    let phnum = u64::from(header.e_phnum) as usize;
    let pheaders_size = phentsize.saturating_mul(phnum);

    if pheaders_size > MAX_PH_SIZE {
        return Err(Error::new(E2BIG));
    }
    let mut phs = vec! [0_u8; pheaders_size];

    let mut tree = BTreeMap::new();
    tree.insert(0, PAGE_SIZE);

    const BUFSZ: usize = 16384;
    let mut buf = vec! [0_u8; BUFSZ];

    read_all(*file as usize, Some(header.e_phoff), &mut phs).map_err(|_| Error::new(EIO))?;

    for ph_idx in 0..phnum {
        let ph_bytes = &phs[ph_idx * phentsize..(ph_idx + 1) * phentsize];
        let segment: &ProgramHeader = plain::from_bytes(ph_bytes).map_err(|_| Error::new(EINVAL))?;
        let mut flags = syscall::PROT_READ;

        // W ^ X. If it is executable, do not allow it to be writable, even if requested
        if segment.p_flags & PF_X == PF_X {
            flags |= syscall::PROT_EXEC;
        } else if segment.p_flags & PF_W == PF_W {
            flags |= syscall::PROT_WRITE;
        }

        let voff = segment.p_vaddr as usize % PAGE_SIZE;
        let vaddr = segment.p_vaddr as usize - voff;
        let size =
            (segment.p_memsz as usize + voff + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE;

        if segment.p_filesz > segment.p_memsz {
            return Err(Error::new(ENOEXEC));
        }
        if segment.p_type == PT_LOAD {
            mprotect_remote(*grants_fd, vaddr, size, flags)?;
            syscall::lseek(*file as usize, segment.p_offset as isize, SEEK_SET).map_err(|_| Error::new(EIO))?;
            syscall::lseek(*memory_fd, segment.p_vaddr as isize, SEEK_SET).map_err(|_| Error::new(EIO))?;

            for size in core::iter::repeat(BUFSZ).take((segment.p_filesz as usize) / BUFSZ).chain(Some((segment.p_filesz as usize) % BUFSZ)) {
                read_all(*file as usize, None, &mut buf[..size]).map_err(|_| Error::new(EIO))?;
                let _ = syscall::write(*memory_fd, &buf[..size]).map_err(|_| Error::new(EIO))?;
            }

            if !tree.range(..=vaddr).next_back().filter(|(start, size)| **start + **size > vaddr).is_some() {
                tree.insert(vaddr, size);
            }
        }
    }
    // Setup a stack starting from the very end of the address space, and then growing downwards.
    const STACK_TOP: usize = 1 << 47;
    const STACK_SIZE: usize = 1024 * 1024;

    mprotect_remote(*grants_fd, STACK_TOP - STACK_SIZE, STACK_SIZE, MapFlags::PROT_READ | MapFlags::PROT_WRITE)?;
    tree.insert(STACK_TOP - STACK_SIZE, STACK_SIZE);

    let mut sp = STACK_TOP - 256;

    let mut push = |word: usize| {
        sp -= core::mem::size_of::<usize>();
        write_all(*memory_fd, Some(sp as u64), &usize::to_ne_bytes(word))
    };

    let pheaders_size_aligned = (pheaders_size+PAGE_SIZE-1)/PAGE_SIZE*PAGE_SIZE;
    let pheaders = find_free_target_addr(&tree, pheaders_size_aligned).ok_or(Error::new(ENOMEM))?;
    tree.insert(pheaders, pheaders_size_aligned);
    mprotect_remote(*grants_fd, pheaders, pheaders_size_aligned, MapFlags::PROT_READ)?;

    write_all(*memory_fd, Some(pheaders as u64), &phs)?;

    push(0)?;
    push(AT_NULL)?;
    push(header.e_entry as usize)?;
    push(AT_ENTRY)?;
    push(pheaders)?;
    push(AT_PHDR)?;
    push(header.e_phnum as usize)?;
    push(AT_PHNUM)?;
    push(header.e_phentsize as usize)?;
    push(AT_PHENT)?;

    let args_envs_size_aligned = (total_args_envs_size+PAGE_SIZE-1)/PAGE_SIZE*PAGE_SIZE;
    let target_args_env_address = find_free_target_addr(&tree, args_envs_size_aligned).ok_or(Error::new(ENOMEM))?;
    mprotect_remote(*grants_fd, target_args_env_address, args_envs_size_aligned, MapFlags::PROT_READ | MapFlags::PROT_WRITE)?;
    tree.insert(target_args_env_address, args_envs_size_aligned);

    let mut offset = 0;

    let mut argc = 0;

    for (collection, is_args) in [(envs, false), (args, true)] {
        push(0)?;

        for source_slice in collection.iter().rev() {
            if is_args { argc += 1; }
            push(target_args_env_address + offset)?;

            write_all(*memory_fd, Some((target_args_env_address + offset) as u64), source_slice)?;
            offset += source_slice.len() + 1;
        }
    }

    push(argc)?;

    unsafe { crate::ld_so::tcb::Tcb::deactivate(); }

    // TODO: Restore old name if exec failed?
    if let Ok(name_fd) = syscall::open("thisproc:current/name", O_WRONLY).map(FdGuard::new) {
        let _ = syscall::write(*name_fd, path);
    }

    let addrspace_selection_fd = FdGuard::new(syscall::open("thisproc:current/current-addrspace", O_WRONLY)?);

    let _ = syscall::write(*addrspace_selection_fd, &create_set_addr_space_buf(*grants_fd, header.e_entry as usize, sp));

    Ok(addrspace_selection_fd)
}
fn mprotect_remote(socket: usize, addr: usize, len: usize, flags: MapFlags) -> Result<()> {
    let mut grants_buf = [0_u8; 24];
    grants_buf[..8].copy_from_slice(&usize::to_ne_bytes(addr));
    grants_buf[8..16].copy_from_slice(&usize::to_ne_bytes(len));
    grants_buf[16..24].copy_from_slice(&usize::to_ne_bytes(flags.bits()));
    syscall::write(socket, &grants_buf)?;
    Ok(())
}
