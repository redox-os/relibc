#![no_std]
#![feature(array_chunks, int_roundings, slice_ptr_get)]
#![forbid(unreachable_patterns)]

extern crate alloc;

use core::mem::size_of;

use alloc::{boxed::Box, collections::BTreeMap, vec};

//TODO: allow use of either 32-bit or 64-bit programs
#[cfg(target_pointer_width = "32")]
use goblin::elf32::{
    header::Header,
    program_header::program_header32::{ProgramHeader, PF_W, PF_X, PT_INTERP, PT_LOAD},
};
#[cfg(target_pointer_width = "64")]
use goblin::elf64::{
    header::Header,
    program_header::program_header64::{ProgramHeader, PF_W, PF_X, PT_INTERP, PT_LOAD},
};

use syscall::{
    error::*,
    flag::{MapFlags, SEEK_SET},
    PAGE_SIZE, Map, PROT_WRITE, O_CLOEXEC,
};

pub use self::arch::*;
mod arch;

pub enum FexecResult {
    Normal {
        addrspace_handle: FdGuard,
    },
    Interp {
        path: Box<[u8]>,
        image_file: FdGuard,
        open_via_dup: FdGuard,
        interp_override: InterpOverride,
    },
}
pub struct InterpOverride {
    phs: Box<[u8]>,
    at_entry: usize,
    at_phnum: usize,
    at_phent: usize,
    name: Box<[u8]>,
    tree: BTreeMap<usize, usize>,
}

pub struct ExtraInfo<'a> {
    pub cwd: Option<&'a [u8]>,
}

pub fn fexec_impl<A, E>(
    image_file: FdGuard,
    open_via_dup: FdGuard,
    memory_scheme_fd: &FdGuard,
    path: &[u8],
    args: A,
    envs: E,
    total_args_envs_size: usize,
    extrainfo: &ExtraInfo,
    mut interp_override: Option<InterpOverride>,
) -> Result<FexecResult>
where
    A: IntoIterator,
    E: IntoIterator,
    A::Item: AsRef<[u8]>,
    E::Item: AsRef<[u8]>,
{
    // Here, we do the minimum part of loading an application, which is what the kernel used to do.
    // We load the executable into memory (albeit at different offsets in this executable), fix
    // some misalignments, and then execute the SYS_EXEC syscall to replace the program memory
    // entirely.

    let mut header_bytes = [0_u8; size_of::<Header>()];
    read_all(*image_file, Some(0), &mut header_bytes)?;
    let header = Header::from_bytes(&header_bytes);

    let grants_fd = {
        let current_addrspace_fd = FdGuard::new(syscall::dup(*open_via_dup, b"addrspace")?);
        FdGuard::new(syscall::dup(*current_addrspace_fd, b"empty")?)
    };

    // Never allow more than 1 MiB of program headers.
    const MAX_PH_SIZE: usize = 1024 * 1024;
    let phentsize = u64::from(header.e_phentsize) as usize;
    let phnum = u64::from(header.e_phnum) as usize;
    let pheaders_size = phentsize
        .saturating_mul(phnum)
        .saturating_add(size_of::<Header>());

    if pheaders_size > MAX_PH_SIZE {
        return Err(Error::new(E2BIG));
    }
    let mut phs_raw = vec![0_u8; pheaders_size];
    phs_raw[..size_of::<Header>()].copy_from_slice(&header_bytes);
    let phs = &mut phs_raw[size_of::<Header>()..];

    // TODO: Remove clone, but this would require more as_refs and as_muts
    let mut tree = interp_override.as_mut().map_or_else(
        || core::iter::once((0, PAGE_SIZE)).collect::<BTreeMap<_, _>>(),
        |o| core::mem::take(&mut o.tree),
    );

    read_all(*image_file as usize, Some(header.e_phoff as u64), phs)
        .map_err(|_| Error::new(EIO))?;

    for ph_idx in 0..phnum {
        let ph_bytes = &phs[ph_idx * phentsize..(ph_idx + 1) * phentsize];
        let segment: &ProgramHeader =
            plain::from_bytes(ph_bytes).map_err(|_| Error::new(EINVAL))?;
        let mut flags = syscall::PROT_READ;

        // W ^ X. If it is executable, do not allow it to be writable, even if requested
        if segment.p_flags & PF_X == PF_X {
            flags |= syscall::PROT_EXEC;
        } else if segment.p_flags & PF_W == PF_W {
            flags |= syscall::PROT_WRITE;
        }

        match segment.p_type {
            // PT_INTERP must come before any PT_LOAD, so we don't have to iterate twice.
            PT_INTERP => {
                let mut interp = vec![0_u8; segment.p_filesz as usize];
                read_all(
                    *image_file as usize,
                    Some(segment.p_offset as u64),
                    &mut interp,
                )?;

                return Ok(FexecResult::Interp {
                    path: interp.into_boxed_slice(),
                    image_file,
                    open_via_dup,
                    interp_override: InterpOverride {
                        at_entry: header.e_entry as usize,
                        at_phnum: phnum,
                        at_phent: phentsize,
                        phs: phs_raw.into_boxed_slice(),
                        name: path.into(),
                        tree,
                    },
                });
            }
            PT_LOAD => {
                let voff = segment.p_vaddr as usize % PAGE_SIZE;
                let vaddr = segment.p_vaddr as usize - voff;
                let filesz = segment.p_filesz as usize;

                let total_page_count = (segment.p_memsz as usize + voff).div_ceil(PAGE_SIZE);

                // The case where segments overlap so that they share one page, is not handled.
                // TODO: Should it be?

                if segment.p_filesz > segment.p_memsz {
                    return Err(Error::new(ENOEXEC));
                }

                allocate_remote(
                    &grants_fd,
                    memory_scheme_fd,
                    vaddr,
                    total_page_count * PAGE_SIZE,
                    flags,
                )?;
                syscall::lseek(*image_file, segment.p_offset as isize, SEEK_SET)
                    .map_err(|_| Error::new(EIO))?;

                // If unaligned, read the head page separately.
                let (first_aligned_page, remaining_filesz) = if voff > 0 {
                    let bytes_to_next_page = PAGE_SIZE - voff;

                    let (_guard, dst_page) = unsafe {
                        MmapGuard::map_mut_anywhere(
                            *grants_fd,
                            vaddr,
                            PAGE_SIZE,
                        )?
                    };

                    let length = core::cmp::min(bytes_to_next_page, filesz);

                    read_all(*image_file, None, &mut dst_page[voff..][..length])?;

                    (vaddr + PAGE_SIZE, filesz - length)
                } else {
                    (vaddr, filesz)
                };

                let remaining_page_count = remaining_filesz.div_floor(PAGE_SIZE);
                let tail_bytes = remaining_filesz % PAGE_SIZE;

                // TODO: Unless the calling process if *very* memory-constrained, the max amount of
                // pages per iteration has no limit other than the time it takes to setup page
                // tables.
                //
                // TODO: Reserve PAGES_PER_ITER "scratch pages" of virtual memory for that type of
                // situation?
                const PAGES_PER_ITER: usize = 64;

                // TODO: Before this loop, attempt to mmap with MAP_PRIVATE directly from the image
                // file.

                for page_idx in (0..remaining_page_count).step_by(PAGES_PER_ITER) {
                    // Use commented out lines to trigger kernel bug (FIXME).

                    //let pages_in_this_group = core::cmp::min(PAGES_PER_ITER, file_page_count - page_idx * PAGES_PER_ITER);
                    let pages_in_this_group = core::cmp::min(PAGES_PER_ITER, remaining_page_count - page_idx);

                    if pages_in_this_group == 0 { break }

                    let _ = syscall::write(1, alloc::format!("idx {} reading (segment {:#?})\n", page_idx, segment).as_bytes());

                    // TODO: MAP_FIXED to optimize away funmap?
                    let (_guard, dst_memory) = unsafe {
                        MmapGuard::map_mut_anywhere(
                            *grants_fd,
                            first_aligned_page + page_idx * PAGE_SIZE, // offset
                            pages_in_this_group * PAGE_SIZE, // size
                        )?
                    };

                    // TODO: Are &mut [u8] and &mut [[u8; PAGE_SIZE]] interchangeable (if the
                    // lengths are aligned, obviously)?

                    read_all(*image_file, None, dst_memory)?;
                }

                if tail_bytes > 0 {
                    let (_guard, dst_page) = unsafe {
                        MmapGuard::map_mut_anywhere(
                            *grants_fd,
                            first_aligned_page + remaining_page_count * PAGE_SIZE,
                            PAGE_SIZE,
                        )?
                    };
                    read_all(*image_file, None, &mut dst_page[..tail_bytes])?;
                }

                // file_page_count..file_page_count + zero_page_count are already zero-initialized
                // by the kernel.

                if !tree
                    .range(..=vaddr)
                    .next_back()
                    .filter(|(start, size)| **start + **size > vaddr)
                    .is_some()
                {
                    tree.insert(vaddr, total_page_count * PAGE_SIZE);
                }
            }
            _ => continue,
        }
    }

    allocate_remote(
        &grants_fd,
        memory_scheme_fd,
        STACK_TOP - STACK_SIZE,
        STACK_SIZE,
        MapFlags::PROT_READ | MapFlags::PROT_WRITE,
    )?;
    tree.insert(STACK_TOP - STACK_SIZE, STACK_SIZE);

    let mut sp = STACK_TOP;
    let mut stack_page = Option::<MmapGuard>::None;

    let mut push = |word: usize| {
        let old_page_no = sp / PAGE_SIZE;
        sp -= size_of::<usize>();
        let new_page_no = sp / PAGE_SIZE;
        let new_page_off = sp % PAGE_SIZE;

        if old_page_no != new_page_no {
            stack_page = None;
        }

        let page = if let Some(ref mut page) = stack_page {
            page.as_mut_ptr_slice()
        } else {
            let new = MmapGuard::map(*grants_fd, &Map {
                offset: new_page_no * PAGE_SIZE,
                size: PAGE_SIZE,
                flags: MapFlags::PROT_WRITE,
                address: 0, // let kernel decide
            })?;

            stack_page.insert(new).as_mut_ptr_slice()
        };

        unsafe {
            page.as_mut_ptr().add(new_page_off).cast::<usize>().write(word);
        }

        Ok(())
    };

    let pheaders_to_convey = if let Some(ref r#override) = interp_override {
        &*r#override.phs
    } else {
        &*phs_raw
    };
    let pheaders_size_aligned = pheaders_to_convey.len().next_multiple_of(PAGE_SIZE);
    let pheaders = find_free_target_addr(&tree, pheaders_size_aligned).ok_or(Error::new(ENOMEM))?;
    tree.insert(pheaders, pheaders_size_aligned);
    allocate_remote(
        &grants_fd,
        memory_scheme_fd,
        pheaders,
        pheaders_size_aligned,
        MapFlags::PROT_READ | MapFlags::PROT_WRITE,
    )?;
    unsafe {
        let (_guard, memory) = MmapGuard::map_mut_anywhere(
            *grants_fd,
            pheaders,
            pheaders_size_aligned,
        )?;

        memory[..pheaders_to_convey.len()].copy_from_slice(pheaders_to_convey);
    }
    mprotect_remote(
        &grants_fd,
        pheaders,
        pheaders_size_aligned,
        MapFlags::PROT_READ,
    )?;

    push(0)?;
    push(AT_NULL)?;
    push(header.e_entry as usize)?;
    if let Some(ref r#override) = interp_override {
        push(AT_BASE)?;
        push(r#override.at_entry)?;
    }
    push(AT_ENTRY)?;
    push(pheaders + size_of::<Header>())?;
    push(AT_PHDR)?;
    push(
        interp_override
            .as_ref()
            .map_or(header.e_phnum as usize, |o| o.at_phnum),
    )?;
    push(AT_PHNUM)?;
    push(
        interp_override
            .as_ref()
            .map_or(header.e_phentsize as usize, |o| o.at_phent),
    )?;
    push(AT_PHENT)?;

    let total_args_envs_auxvpointee_size =
        total_args_envs_size + extrainfo.cwd.map_or(0, |s| s.len() + 1);
    let args_envs_size_aligned = total_args_envs_auxvpointee_size.next_multiple_of(PAGE_SIZE);
    let target_args_env_address =
        find_free_target_addr(&tree, args_envs_size_aligned).ok_or(Error::new(ENOMEM))?;
    allocate_remote(
        &grants_fd,
        memory_scheme_fd,
        target_args_env_address,
        args_envs_size_aligned,
        MapFlags::PROT_READ | MapFlags::PROT_WRITE,
    )?;
    tree.insert(target_args_env_address, args_envs_size_aligned);

    let mut offset = 0;

    let mut argc = 0;

    {
        let mut append = |source_slice: &[u8]| {
            // TODO
            let address = target_args_env_address + offset;

            if !source_slice.is_empty() {
                let containing_page = address.div_floor(PAGE_SIZE) * PAGE_SIZE;
                let displacement = address - containing_page;
                let size = source_slice.len() + displacement;
                let aligned_size = size.next_multiple_of(PAGE_SIZE);

                let (_guard, memory) = unsafe {
                    MmapGuard::map_mut_anywhere(
                        *grants_fd, 
                        containing_page,
                        aligned_size,
                    )?
                };
                memory[displacement..][..source_slice.len()].copy_from_slice(source_slice);
            }

            offset += source_slice.len() + 1;
            Ok(address)
        };

        if let Some(cwd) = extrainfo.cwd {
            push(append(cwd)?)?;
            push(AT_REDOX_INITIALCWD_PTR)?;
            push(cwd.len())?;
            push(AT_REDOX_INITIALCWD_LEN)?;
        }

        push(0)?;

        for env in envs {
            push(append(env.as_ref())?)?;
        }

        push(0)?;

        for arg in args {
            push(append(arg.as_ref())?)?;
            argc += 1;
        }
    }

    push(argc)?;

    unsafe {
        deactivate_tcb(*open_via_dup)?;
    }

    {
        let current_sigaction_fd = FdGuard::new(syscall::dup(*open_via_dup, b"sigactions")?);
        let empty_sigaction_fd = FdGuard::new(syscall::dup(*current_sigaction_fd, b"empty")?);
        let sigaction_selection_fd =
            FdGuard::new(syscall::dup(*open_via_dup, b"current-sigactions")?);

        let _ = syscall::write(
            *sigaction_selection_fd,
            &usize::to_ne_bytes(*empty_sigaction_fd),
        )?;
    }

    // TODO: Restore old name if exec failed?
    if let Ok(name_fd) = syscall::dup(*open_via_dup, b"name").map(FdGuard::new) {
        let _ = syscall::write(*name_fd, interp_override.as_ref().map_or(path, |o| &o.name));
    }
    if interp_override.is_some() {
        let mmap_min_fd = FdGuard::new(syscall::dup(*grants_fd, b"mmap-min-addr")?);
        let last_addr = tree.iter().rev().nth(1).map_or(0, |(off, len)| *off + *len);
        let aligned_last_addr = last_addr.next_multiple_of(PAGE_SIZE);
        let _ = syscall::write(*mmap_min_fd, &usize::to_ne_bytes(aligned_last_addr));
    }

    let addrspace_selection_fd = FdGuard::new(syscall::dup(*open_via_dup, b"current-addrspace")?);
            let _ = syscall::write(1, alloc::format!("ENTER {:p} {:p}\n", header.e_entry as *const u8, sp as *const u8).as_bytes());

    let _ = syscall::write(
        *addrspace_selection_fd,
        &create_set_addr_space_buf(*grants_fd, header.e_entry as usize, sp),
    );

    Ok(FexecResult::Normal {
        addrspace_handle: addrspace_selection_fd,
    })
}
fn write_usizes<const N: usize>(fd: &FdGuard, usizes: [usize; N]) -> Result<()> {
    let _ = syscall::write(**fd, unsafe { plain::as_bytes(&usizes) });
    Ok(())
}
fn allocate_remote(
    addrspace_fd: &FdGuard,
    memory_scheme_fd: &FdGuard,
    dst_addr: usize,
    len: usize,
    flags: MapFlags,
) -> Result<()> {
    mmap_remote(addrspace_fd, memory_scheme_fd, 0, dst_addr, len, flags)
}
pub fn mmap_remote(
    addrspace_fd: &FdGuard,
    fd: &FdGuard,
    offset: usize,
    dst_addr: usize,
    len: usize,
    flags: MapFlags,
) -> Result<()> {
    write_usizes(
        addrspace_fd,
        [
            // op
            syscall::flag::ADDRSPACE_OP_MMAP,
            // fd
            **fd,
            // "offset"
            offset,
            // address
            dst_addr,
            // size
            len,
            // flags
            (flags | MapFlags::MAP_FIXED_NOREPLACE).bits(),
        ],
    )
}
pub fn mprotect_remote(
    addrspace_fd: &FdGuard,
    addr: usize,
    len: usize,
    flags: MapFlags,
) -> Result<()> {
    write_usizes(
        addrspace_fd,
        [
            // op
            syscall::flag::ADDRSPACE_OP_MPROTECT,
            // address
            addr,
            // size
            len,
            // flags
            flags.bits(),
        ],
    )
}
pub fn munmap_remote(addrspace_fd: &FdGuard, addr: usize, len: usize) -> Result<()> {
    write_usizes(
        addrspace_fd,
        [
            // op
            syscall::flag::ADDRSPACE_OP_MUNMAP,
            // address
            addr,
            // size
            len,
        ],
    )
}
pub fn munmap_transfer(
    src: &FdGuard,
    dst: &FdGuard,
    src_addr: usize,
    dst_addr: usize,
    len: usize,
    flags: MapFlags,
) -> Result<()> {
    write_usizes(
        dst,
        [
            // op
            syscall::flag::ADDRSPACE_OP_TRANSFER,
            // fd
            **src,
            // "offset" (source address)
            src_addr,
            // address
            dst_addr,
            // size
            len,
            // flags
            (flags | MapFlags::MAP_FIXED_NOREPLACE).bits(),
        ],
    )
}
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

// TODO: With the introduction of remote mmaps, remove this and let the kernel handle address
// allocation.
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

pub struct MmapGuard {
    base: usize,
    size: usize,
    taken: bool,
}
impl MmapGuard {
    pub fn map(fd: usize, map: &Map) -> Result<Self> {
        Ok(Self {
            size: map.size,
            taken: false,

            base: unsafe { syscall::fmap(fd, map)? },
        })
    }
    pub unsafe fn map_mut_anywhere<'a>(fd: usize, offset: usize, size: usize) -> Result<(Self, &'a mut [u8])> {
        let mut this = Self::map(fd, &Map {
            size,
            offset,
            address: 0,
            flags: PROT_WRITE,
        })?;
        let slice = &mut *this.as_mut_ptr_slice();

        Ok((this, slice))
    }
    pub fn addr(&self) -> usize {
        self.base
    }
    pub fn len(&self) -> usize {
        self.size
    }
    pub fn as_mut_ptr_slice(&mut self) -> *mut [u8] {
        core::ptr::slice_from_raw_parts_mut(self.base as *mut u8, self.size)
    }
    pub fn take(&mut self) {
        self.taken = true;
    }
}
impl Drop for MmapGuard {
    fn drop(&mut self) {
        if !self.taken {
            let _ = unsafe { syscall::funmap(self.base, self.size) };
        }
    }
}

pub struct FdGuard {
    fd: usize,
    taken: bool,
}
impl FdGuard {
    pub fn new(fd: usize) -> Self {
        Self { fd, taken: false }
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
pub fn create_set_addr_space_buf(
    space: usize,
    ip: usize,
    sp: usize,
) -> [u8; size_of::<usize>() * 3] {
    let mut buf = [0_u8; 3 * size_of::<usize>()];
    let mut chunks = buf.array_chunks_mut::<{ size_of::<usize>() }>();
    *chunks.next().unwrap() = usize::to_ne_bytes(space);
    *chunks.next().unwrap() = usize::to_ne_bytes(sp);
    *chunks.next().unwrap() = usize::to_ne_bytes(ip);
    buf
}

#[path = "../../../auxv_defs.rs"]
pub mod auxv_defs;

use auxv_defs::*;

/// Spawns a new context which will not share the same address space as the current one. File
/// descriptors from other schemes are reobtained with `dup`, and grants referencing such file
/// descriptors are reobtained through `fmap`. Other mappings are kept but duplicated using CoW.
pub fn fork_impl() -> Result<usize> {
    unsafe { Error::demux(__relibc_internal_fork_wrapper()) }
}

fn fork_inner(initial_rsp: *mut usize) -> Result<usize> {
    let (cur_filetable_fd, new_pid_fd, new_pid);

    {
        let cur_pid_fd = FdGuard::new(syscall::open(
            "thisproc:current/open_via_dup",
            O_CLOEXEC,
        )?);
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

        {
            let cur_sigaction_fd = FdGuard::new(syscall::dup(*cur_pid_fd, b"sigactions")?);
            let new_sigaction_fd = FdGuard::new(syscall::dup(*cur_sigaction_fd, b"copy")?);
            let new_sigaction_sel_fd =
                FdGuard::new(syscall::dup(*new_pid_fd, b"current-sigactions")?);

            let _ = syscall::write(
                *new_sigaction_sel_fd,
                &usize::to_ne_bytes(*new_sigaction_fd),
            )?;
        }

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
            let new_addr_space_sel_fd =
                FdGuard::new(syscall::dup(*new_pid_fd, b"current-addrspace")?);

            let cur_addr_space_fd = FdGuard::new(syscall::dup(*cur_pid_fd, b"addrspace")?);
            let new_addr_space_fd = FdGuard::new(syscall::dup(*cur_addr_space_fd, b"exclusive")?);

            let buf = create_set_addr_space_buf(
                *new_addr_space_fd,
                __relibc_internal_fork_ret as usize,
                initial_rsp as usize,
            );
            let _ = syscall::write(*new_addr_space_sel_fd, &buf)?;
        }
        copy_env_regs(*cur_pid_fd, *new_pid_fd)?;
    }
    // Copy the file table. We do this last to ensure that all previously used file descriptors are
    // closed. The only exception -- the filetable selection fd and the current filetable fd --
    // will be closed by the child process.
    {
        // TODO: Use cross_scheme_links or something similar to avoid copying the file table in the
        // kernel.
        let new_filetable_fd = FdGuard::new(syscall::dup(*cur_filetable_fd, b"copy")?);
        let new_filetable_sel_fd = FdGuard::new(syscall::dup(*new_pid_fd, b"current-filetable")?);
        let _ = syscall::write(
            *new_filetable_sel_fd,
            &usize::to_ne_bytes(*new_filetable_fd),
        );
    }

    // Unblock context.
    syscall::kill(new_pid, syscall::SIGCONT)?;

    // XXX: Killing with SIGCONT will put (pid, 65536) at key (pid, pgid) into the waitpid of this
    // context. This means that if pgid is changed (as it is in ion for example), the pgid message
    // in syscall::exit() will not be inserted as the key comparator thinks they're equal as their
    // PIDs are. So, we have to call this to clear the waitpid queue to prevent deadlocks.
    let _ = syscall::waitpid(new_pid, &mut 0, syscall::WUNTRACED | syscall::WCONTINUED);

    Ok(new_pid)
}

pub fn new_context() -> Result<(FdGuard, usize)> {
    // Create a new context (fields such as uid/gid will be inherited from the current context).
    let fd = FdGuard::new(syscall::open(
        "thisproc:new/open_via_dup",
        O_CLOEXEC,
    )?);

    // Extract pid.
    let mut buffer = [0_u8; 64];
    let len = syscall::fpath(*fd, &mut buffer)?;
    let buffer = buffer.get(..len).ok_or(Error::new(ENAMETOOLONG))?;

    let colon_idx = buffer
        .iter()
        .position(|c| *c == b':')
        .ok_or(Error::new(EINVAL))?;
    let slash_idx = buffer
        .iter()
        .skip(colon_idx)
        .position(|c| *c == b'/')
        .ok_or(Error::new(EINVAL))?
        + colon_idx;
    let pid_bytes = buffer
        .get(colon_idx + 1..slash_idx)
        .ok_or(Error::new(EINVAL))?;
    let pid_str = core::str::from_utf8(pid_bytes).map_err(|_| Error::new(EINVAL))?;
    let pid = pid_str.parse::<usize>().map_err(|_| Error::new(EINVAL))?;

    Ok((fd, pid))
}

pub fn copy_str(cur_pid_fd: usize, new_pid_fd: usize, key: &str) -> Result<()> {
    let cur_name_fd = FdGuard::new(syscall::dup(cur_pid_fd, key.as_bytes())?);
    let new_name_fd = FdGuard::new(syscall::dup(new_pid_fd, key.as_bytes())?);

    // TODO: Max path size?
    let mut buf = [0_u8; 256];
    let len = syscall::read(*cur_name_fd, &mut buf)?;
    let buf = buf.get(..len).ok_or(Error::new(ENAMETOOLONG))?;

    syscall::write(*new_name_fd, &buf)?;

    Ok(())
}
