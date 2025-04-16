use core::{
    cell::SyncUnsafeCell,
    fmt::Debug,
    mem::{size_of, MaybeUninit},
};

use crate::{
    arch::*,
    auxv_defs::*,
    protocol::{ProcCall, ThreadCall},
    read_proc_meta, static_proc_info,
    sys::{proc_call, thread_call},
    RtTcb, StaticProcInfo, DYNAMIC_PROC_INFO,
};

use alloc::{boxed::Box, collections::BTreeMap, vec};

//TODO: allow use of either 32-bit or 64-bit programs
#[cfg(target_pointer_width = "32")]
use goblin::elf32::{
    header::Header,
    program_header::program_header32::{ProgramHeader, PF_R, PF_W, PF_X, PT_INTERP, PT_LOAD},
};
#[cfg(target_pointer_width = "64")]
use goblin::elf64::{
    header::Header,
    program_header::program_header64::{ProgramHeader, PF_R, PF_W, PF_X, PT_INTERP, PT_LOAD},
};

use syscall::{
    error::*,
    flag::{MapFlags, SEEK_SET},
    CallFlags, GrantDesc, GrantFlags, Map, ProcSchemeAttrs, SetSighandlerData, MAP_FIXED_NOREPLACE,
    MAP_SHARED, O_CLOEXEC, PAGE_SIZE, PROT_EXEC, PROT_READ, PROT_WRITE,
};

pub enum FexecResult {
    Normal {
        addrspace_handle: FdGuard,
    },
    Interp {
        path: Box<[u8]>,
        image_file: FdGuard,
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
    // Default scheme for the process
    pub default_scheme: Option<&'a [u8]>,
    // POSIX states that while sigactions are reset, ignored sigactions will remain ignored.
    pub sigignmask: u64,
    // POSIX also states that the sigprocmask must be preserved across execs.
    pub sigprocmask: u64,
    /// File mode creation mask (POSIX)
    pub umask: u32,
    /// Thread handle
    pub thr_fd: usize,
    /// Process handle
    pub proc_fd: usize,
}

pub fn fexec_impl<A, E>(
    image_file: FdGuard,
    thread_fd: &FdGuard,
    proc_fd: &FdGuard,
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
    // some misalignments, and then switch address space.

    let mut header_bytes = [0_u8; size_of::<Header>()];
    pread_all(*image_file, 0, &mut header_bytes)?;
    let header = Header::from_bytes(&header_bytes);

    let grants_fd = {
        let current_addrspace_fd = FdGuard::new(syscall::dup(**thread_fd, b"addrspace")?);
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

    pread_all(*image_file as usize, u64::from(header.e_phoff), phs).map_err(|_| Error::new(EIO))?;

    for ph_idx in 0..phnum {
        let ph_bytes = &phs[ph_idx * phentsize..(ph_idx + 1) * phentsize];
        let segment: &ProgramHeader =
            plain::from_bytes(ph_bytes).map_err(|_| Error::new(EINVAL))?;
        let mut flags = if segment.p_flags & PF_R == PF_R {
            syscall::PROT_READ
        } else {
            syscall::PROT_NONE
        };

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
                pread_all(
                    *image_file as usize,
                    u64::from(segment.p_offset),
                    &mut interp,
                )?;

                return Ok(FexecResult::Interp {
                    path: interp.into_boxed_slice(),
                    image_file,
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

                // TODO: Attempt to mmap with MAP_PRIVATE directly from the image file instead.

                if filesz > 0 {
                    let (_guard, dst_memory) = unsafe {
                        MmapGuard::map_mut_anywhere(
                            *grants_fd,
                            vaddr,                                       // offset
                            (voff + filesz).next_multiple_of(PAGE_SIZE), // size
                        )?
                    };
                    pread_all(
                        *image_file,
                        u64::from(segment.p_offset),
                        &mut dst_memory[voff..voff + filesz],
                    )?;
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

        let page = if let Some(ref mut page) = stack_page
            && old_page_no == new_page_no
        {
            page
        } else if let Some(ref mut stack_page) = stack_page {
            stack_page.remap(new_page_no * PAGE_SIZE, PROT_READ | PROT_WRITE)?;
            stack_page
        } else {
            let new = MmapGuard::map(
                *grants_fd,
                &Map {
                    offset: new_page_no * PAGE_SIZE,
                    size: PAGE_SIZE,
                    flags: PROT_READ | PROT_WRITE,
                    address: 0, // let kernel decide
                },
            )?;

            stack_page.insert(new)
        };

        unsafe {
            page.as_mut_ptr_slice()
                .as_mut_ptr()
                .add(new_page_off)
                .cast::<usize>()
                .write(word);
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
        let (_guard, memory) =
            MmapGuard::map_mut_anywhere(*grants_fd, pheaders, pheaders_size_aligned)?;

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

    let total_args_envs_auxvpointee_size = total_args_envs_size
        + extrainfo.cwd.map_or(0, |s| s.len() + 1)
        + extrainfo.default_scheme.map_or(0, |s| s.len() + 1);
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
                    MmapGuard::map_mut_anywhere(*grants_fd, containing_page, aligned_size)?
                };
                memory[displacement..][..source_slice.len()].copy_from_slice(source_slice);
            }

            offset += source_slice.len() + 1;
            Ok(address)
        };

        if let Some(cwd) = extrainfo.cwd {
            push(append(cwd)?)?;
            push(AT_REDOX_INITIAL_CWD_PTR)?;
            push(cwd.len())?;
            push(AT_REDOX_INITIAL_CWD_LEN)?;
        }

        if let Some(default_scheme) = extrainfo.default_scheme {
            push(append(default_scheme)?)?;
            push(AT_REDOX_INITIAL_DEFAULT_SCHEME_PTR)?;
            push(default_scheme.len())?;
            push(AT_REDOX_INITIAL_DEFAULT_SCHEME_LEN)?;
        }

        #[cfg(target_pointer_width = "32")]
        {
            push((extrainfo.sigignmask >> 32) as usize)?;
            push(AT_REDOX_INHERITED_SIGIGNMASK_HI)?;
        }
        push(extrainfo.sigignmask as usize)?;
        push(AT_REDOX_INHERITED_SIGIGNMASK)?;
        #[cfg(target_pointer_width = "32")]
        {
            push((extrainfo.sigprocmask >> 32) as usize)?;
            push(AT_REDOX_INHERITED_SIGPROCMASK_HI)?;
        }
        push(extrainfo.sigprocmask as usize)?;
        push(AT_REDOX_INHERITED_SIGPROCMASK)?;

        push(extrainfo.umask as usize)?;
        push(AT_REDOX_UMASK)?;

        push(extrainfo.thr_fd as usize)?;
        push(AT_REDOX_THR_FD)?;
        push(extrainfo.proc_fd as usize)?;
        push(AT_REDOX_PROC_FD)?;

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

    if let Ok(sighandler_fd) = syscall::dup(**thread_fd, b"sighandler").map(FdGuard::new) {
        let _ = syscall::write(
            *sighandler_fd,
            &SetSighandlerData {
                user_handler: 0,
                excp_handler: 0,
                thread_control_addr: 0,
                proc_control_addr: 0,
            },
        );
        // TODO: sync with procmgr
    }

    unsafe {
        deactivate_tcb(**thread_fd)?;
    }

    // TODO: Restore old name if exec failed?
    if let Ok(name_fd) = syscall::dup(**thread_fd, b"name").map(FdGuard::new) {
        let _ = syscall::write(*name_fd, interp_override.as_ref().map_or(path, |o| &o.name));
    }
    if interp_override.is_some() {
        let mmap_min_fd = FdGuard::new(syscall::dup(*grants_fd, b"mmap-min-addr")?);
        let last_addr = tree.iter().rev().nth(1).map_or(0, |(off, len)| *off + *len);
        let aligned_last_addr = last_addr.next_multiple_of(PAGE_SIZE);
        let _ = syscall::write(*mmap_min_fd, &usize::to_ne_bytes(aligned_last_addr));
    }

    let addrspace_selection_fd = FdGuard::new(syscall::dup(**thread_fd, b"current-addrspace")?);

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
fn pread_all(fd: usize, offset: u64, buf: &mut [u8]) -> Result<()> {
    syscall::lseek(fd, offset as isize, SEEK_SET)?;

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
    fd: usize,
    base: usize,
    size: usize,
}
impl MmapGuard {
    pub fn map(fd: usize, map: &Map) -> Result<Self> {
        Ok(Self {
            fd,
            size: map.size,
            base: unsafe { syscall::fmap(fd, map)? },
        })
    }
    pub fn remap(&mut self, offset: usize, mut flags: MapFlags) -> Result<()> {
        flags.remove(MapFlags::MAP_FIXED_NOREPLACE);
        flags.insert(MapFlags::MAP_FIXED);

        let _new_base = unsafe {
            syscall::fmap(
                self.fd,
                &Map {
                    offset,
                    size: self.size,
                    flags,
                    address: self.base,
                },
            )?
        };

        Ok(())
    }
    pub unsafe fn map_mut_anywhere<'a>(
        fd: usize,
        offset: usize,
        size: usize,
    ) -> Result<(Self, &'a mut [u8])> {
        let mut this = Self::map(
            fd,
            &Map {
                size,
                offset,
                address: 0,
                flags: PROT_READ | PROT_WRITE,
            },
        )?;
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
    pub fn take(mut self) {
        self.size = 0;
    }
}
impl Drop for MmapGuard {
    fn drop(&mut self) {
        if self.size != 0 {
            let _ = unsafe { syscall::funmap(self.base, self.size) };
        }
    }
}

#[repr(transparent)]
pub struct FdGuard {
    fd: usize,
}
impl FdGuard {
    #[inline]
    pub const fn new(fd: usize) -> Self {
        Self { fd }
    }
    #[inline]
    pub fn take(self) -> usize {
        let fd = self.fd;
        core::mem::forget(self);
        fd
    }
}
impl core::ops::Deref for FdGuard {
    type Target = usize;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.fd
    }
}

impl Drop for FdGuard {
    #[inline]
    fn drop(&mut self) {
        let _ = syscall::close(self.fd);
    }
}
impl Debug for FdGuard {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[fd {}]", self.fd)
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

/// Spawns a new context which will not share the same address space as the current one. File
/// descriptors from other schemes are reobtained with `dup`, and grants referencing such file
/// descriptors are reobtained through `fmap`. Other mappings are kept but duplicated using CoW.
pub fn fork_impl(args: &ForkArgs<'_>) -> Result<usize> {
    let old_mask = crate::signal::get_sigmask()?;
    let pid = unsafe {
        Error::demux(__relibc_internal_fork_wrapper(
            args as *const ForkArgs as usize,
        ))?
    };

    if pid == 0 {
        crate::signal::set_sigmask(Some(old_mask), None)?;
    }
    Ok(pid)
}

pub enum ForkArgs<'a> {
    Init {
        this_thr_fd: &'a FdGuard,
        auth: &'a FdGuard,
    },
    Managed,
}

pub fn fork_inner(initial_rsp: *mut usize, args: &ForkArgs) -> Result<usize> {
    let (cur_filetable_fd, new_proc_fd, new_thr_fd, new_pid);

    {
        let cur_thr_fd = match args {
            ForkArgs::Init { this_thr_fd, .. } => this_thr_fd,
            ForkArgs::Managed => RtTcb::current().thread_fd(),
        };
        let NewChildProc {
            proc_fd,
            thr_fd,
            pid,
        } = new_child_process(args)?;
        new_proc_fd = proc_fd;
        new_thr_fd = thr_fd;
        new_pid = pid;

        copy_str(**cur_thr_fd, *new_thr_fd, "name")?;

        // Copy existing files into new file table, but do not reuse the same file table (i.e. new
        // parent FDs will not show up for the child).
        {
            cur_filetable_fd = FdGuard::new(syscall::dup(**cur_thr_fd, b"filetable")?);

            // This must be done before the address space is copied.
            unsafe {
                let proc_fd = new_proc_fd.as_ref().map_or(usize::MAX, |p| **p);
                //let _ = syscall::write(1, alloc::format!("P{}Q{}R{}\n", *cur_filetable_fd, proc_fd, *new_thr_fd).as_bytes());
                initial_rsp.write(*cur_filetable_fd);
                initial_rsp.add(1).write(proc_fd);
                initial_rsp.add(2).write(*new_thr_fd);
            }
        }

        // CoW-duplicate address space.
        {
            let new_addr_space_sel_fd =
                FdGuard::new(syscall::dup(*new_thr_fd, b"current-addrspace")?);

            let cur_addr_space_fd = FdGuard::new(syscall::dup(**cur_thr_fd, b"addrspace")?);
            let new_addr_space_fd = FdGuard::new(syscall::dup(*cur_addr_space_fd, b"exclusive")?);

            let mut grant_desc_buf = [GrantDesc::default(); 16];
            loop {
                let bytes_read = {
                    let buf = unsafe {
                        core::slice::from_raw_parts_mut(
                            grant_desc_buf.as_mut_ptr().cast(),
                            grant_desc_buf.len() * size_of::<GrantDesc>(),
                        )
                    };
                    syscall::read(*cur_addr_space_fd, buf)?
                };
                if bytes_read == 0 {
                    break;
                }

                let grants = &grant_desc_buf[..bytes_read / size_of::<GrantDesc>()];

                for grant in grants {
                    if !grant.flags.contains(GrantFlags::GRANT_SCHEME)
                        || !grant.flags.contains(GrantFlags::GRANT_SHARED)
                    {
                        continue;
                    }

                    let buf;

                    // TODO: write! using some #![no_std] Cursor type (tracking the length)?
                    #[cfg(target_pointer_width = "64")]
                    {
                        //buf = *b"grant-fd-AAAABBBBCCCCDDDD";
                        //write!(&mut buf, "grant-fd-{:>016x}", grant.base).unwrap();
                        buf = alloc::format!("grant-fd-{:>016x}", grant.base).into_bytes();
                    }

                    #[cfg(target_pointer_width = "32")]
                    {
                        //buf = *b"grant-fd-AAAABBBB";
                        //write!(&mut buf[..], "grant-fd-{:>08x}", grant.base).unwrap();
                        buf = alloc::format!("grant-fd-{:>08x}", grant.base).into_bytes();
                    }

                    let grant_fd = FdGuard::new(syscall::dup(*cur_addr_space_fd, &buf)?);

                    let mut flags = MAP_SHARED | MAP_FIXED_NOREPLACE;

                    flags.set(PROT_READ, grant.flags.contains(GrantFlags::GRANT_READ));
                    flags.set(PROT_WRITE, grant.flags.contains(GrantFlags::GRANT_WRITE));
                    flags.set(PROT_EXEC, grant.flags.contains(GrantFlags::GRANT_EXEC));

                    mmap_remote(
                        &new_addr_space_fd,
                        &grant_fd,
                        grant.offset as usize,
                        grant.base,
                        grant.size,
                        flags,
                    )?;
                }
            }

            let buf = create_set_addr_space_buf(
                *new_addr_space_fd,
                __relibc_internal_fork_ret as usize,
                initial_rsp as usize,
            );
            let _ = syscall::write(*new_addr_space_sel_fd, &buf)?;
        }
        {
            // Reuse the same sigaltstack and signal entry (all memory will be re-mapped CoW later).
            //
            // Do this after the address space is cloned, since the kernel will get a shared
            // reference to the TCB and whatever pages stores the signal proc control struct.
            {
                let new_sighandler_fd = FdGuard::new(syscall::dup(*new_thr_fd, b"sighandler")?);
                let _ = syscall::write(
                    *new_sighandler_fd,
                    &crate::signal::current_setsighandler_struct(),
                )?;
            }
            if let Some(ref proc_fd) = new_proc_fd {
                proc_call(
                    **proc_fd,
                    &mut [],
                    CallFlags::empty(),
                    &[ProcCall::SyncSigPctl as u64],
                )?;
                thread_call(
                    *new_thr_fd,
                    &mut [],
                    CallFlags::empty(),
                    &[ThreadCall::SyncSigTctl as u64],
                )?;
            }
        }
        copy_env_regs(**cur_thr_fd, *new_thr_fd)?;
    }
    // Copy the file table. We do this last to ensure that all previously used file descriptors are
    // closed. The only exception -- the filetable selection fd and the current filetable fd --
    // will be closed by the child process.
    {
        // TODO: Use file descriptor forwarding or something similar to avoid copying the file
        // table in the kernel.
        let new_filetable_fd = FdGuard::new(syscall::dup(*cur_filetable_fd, b"copy")?);
        let new_filetable_sel_fd = FdGuard::new(syscall::dup(*new_thr_fd, b"current-filetable")?);
        let _ = syscall::write(
            *new_filetable_sel_fd,
            &usize::to_ne_bytes(*new_filetable_fd),
        )?;
    }
    let start_fd = FdGuard::new(syscall::dup(*new_thr_fd, b"start")?);
    let _ = syscall::write(*start_fd, &[0])?;

    Ok(new_pid)
}

struct NewChildProc {
    proc_fd: Option<FdGuard>,

    thr_fd: FdGuard,
    pid: usize,
}

pub fn new_child_process(args: &ForkArgs<'_>) -> Result<NewChildProc> {
    match *args {
        ForkArgs::Managed => {
            let proc_info = crate::static_proc_info();
            assert!(
                proc_info.has_proc_fd,
                "cannot use ForkArgs::Managed without an existing proc info"
            );
            let this_proc_fd = unsafe { proc_info.proc_fd.assume_init_ref() };
            let child_proc_fd = FdGuard::new(syscall::dup(**this_proc_fd, b"fork")?);
            let only_thread_fd = FdGuard::new(syscall::dup(*child_proc_fd, b"thread-0")?);
            let meta = read_proc_meta(&child_proc_fd)?;
            Ok(NewChildProc {
                proc_fd: Some(child_proc_fd),
                thr_fd: only_thread_fd,
                pid: meta.pid as usize,
            })
        }
        #[cfg(feature = "proc")]
        ForkArgs::Init { .. } => unreachable!(),

        #[cfg(not(feature = "proc"))]
        ForkArgs::Init { this_thr_fd, auth } => {
            let thr_fd = FdGuard::new(syscall::dup(**auth, b"new-context")?);
            let buf = ProcSchemeAttrs {
                pid: 0,
                euid: 0,
                egid: 0,
                ens: 1,
            };
            let attr_fd = FdGuard::new(syscall::dup(
                *thr_fd,
                alloc::format!("auth-{}-attrs", **auth).as_bytes(),
            )?);
            let _ = syscall::write(*attr_fd, &buf)?;
            Ok(NewChildProc {
                thr_fd,
                pid: 1, // dummy fd to distinguish child from parent
                proc_fd: None,
            })
        }
    }
}

pub fn copy_str(cur_pid_fd: usize, new_proc_fd: usize, key: &str) -> Result<()> {
    let cur_name_fd = FdGuard::new(syscall::dup(cur_pid_fd, key.as_bytes())?);
    let new_name_fd = FdGuard::new(syscall::dup(new_proc_fd, key.as_bytes())?);

    // TODO: Max path size?
    let mut buf = [0_u8; 256];
    let len = syscall::read(*cur_name_fd, &mut buf)?;
    let buf = buf.get(..len).ok_or(Error::new(ENAMETOOLONG))?;

    syscall::write(*new_name_fd, &buf)?;

    Ok(())
}

pub unsafe fn make_init() -> [&'static FdGuard; 2] {
    let proc_fd = FdGuard::new(
        syscall::open("/scheme/proc/init", syscall::O_CLOEXEC).expect("failed to create init"),
    );
    syscall::sendfd(
        *proc_fd,
        syscall::dup(**RtTcb::current().thread_fd(), &[]).unwrap(),
        0,
        0,
    )
    .expect("failed to assign current thread to init process");

    let managed_thr_fd = FdGuard::new(
        syscall::dup(*proc_fd, b"thread-0").expect("failed to get managed thread for init"),
    );

    let managed_thr_fd = (*RtTcb::current().thr_fd.get()).insert(managed_thr_fd);

    STATIC_PROC_INFO.get().write(crate::StaticProcInfo {
        pid: 1,
        ppid: 1,
        proc_fd: MaybeUninit::new(proc_fd),
        has_proc_fd: true,
    });
    *DYNAMIC_PROC_INFO.lock() = crate::DynamicProcInfo {
        pgid: 1,
        ruid: 0,
        euid: 0,
        suid: 0,
        rgid: 0,
        egid: 0,
        sgid: 0,
    };
    [
        (*STATIC_PROC_INFO.get()).proc_fd.assume_init_ref(),
        managed_thr_fd,
    ]
}
pub(crate) static STATIC_PROC_INFO: SyncUnsafeCell<StaticProcInfo> =
    SyncUnsafeCell::new(StaticProcInfo {
        pid: 0,
        ppid: 0,
        proc_fd: MaybeUninit::zeroed(),
        has_proc_fd: false,
    });
