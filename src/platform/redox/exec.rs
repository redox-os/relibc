use core::convert::TryFrom;

use alloc::{
    collections::{btree_map::Entry, BTreeMap},
    vec::Vec,
};

use syscall::{
    data::ExecMemRange,
    error::{Error, Result, ENOEXEC, ENOMEM},
    flag::{AT_ENTRY, AT_NULL, AT_PHDR, AT_PHENT, AT_PHNUM, MapFlags},
};

fn read_all(fd: usize, offset: u64, buf: &mut [u8]) -> Result<()> {
    syscall::lseek(fd, offset as isize, syscall::SEEK_SET).unwrap();

    let mut total_bytes_read = 0;

    while total_bytes_read < buf.len() {
        total_bytes_read += match syscall::read(fd, &mut buf[total_bytes_read..])? {
            0 => return Err(Error::new(ENOEXEC)),
            bytes_read => bytes_read,
        }
    }
    Ok(())
}

fn find_free_target_addr(tree: &BTreeMap<usize, TreeEntry>, size: usize) -> Option<usize> {
    let mut iterator = tree.iter().peekable();

    // Ignore the space between zero and the first region, to avoid null pointers.
    while let Some((cur_address, entry)) = iterator.next() {
        let end = *cur_address + entry.size;

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
struct TreeEntry {
    size: usize, // always a page-size multiple
    flags: MapFlags,
    accessible_addr: *mut u8, // also always a page-size multiple
}
impl Drop for TreeEntry {
    fn drop(&mut self) {
        unsafe {
            if !self.accessible_addr.is_null() {
                let _ = syscall::funmap(self.accessible_addr as usize, self.size);
            }
        }
    }
}

#[cfg(target_arch = "x86_64")]
const PAGE_SIZE: usize = 4096;

const FD_ANONYMOUS: usize = !0;

pub fn fexec_impl(fd: usize, path: &[u8], args: &[&[u8]], envs: &[&[u8]], args_envs_size_without_nul: usize) -> Result<usize> {
    let total_args_envs_size = args_envs_size_without_nul + args.len() + envs.len();

    // Here, we do the minimum part of loading an application, which is what the kernel used to do.
    // We load the executable into memory (albeit at different offsets in this executable), fix
    // some misalignments, and then execute the SYS_EXEC syscall to replace the program memory
    // entirely.

    // TODO: setuid/setgid
    // TODO: Introduce RAII guards to all owned allocations so that no leaks occur in case of
    // errors.

    use goblin::elf::header::header64::Header;

    let mut header_bytes = [0_u8; core::mem::size_of::<Header>()];

    read_all(fd, 0, &mut header_bytes)?;

    let header = Header::from_bytes(&header_bytes);

    let instruction_ptr = usize::try_from(header.e_entry).map_err(|_| Error::new(ENOEXEC))?;

    let mut tree = BTreeMap::<usize, TreeEntry>::new();

    use goblin::elf64::program_header::{self, ProgramHeader};

    let phdrs_size = (header.e_phnum as usize) * (header.e_phentsize as usize);
    let phdrs_size_aligned = (phdrs_size + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE;
    let phdrs_mem = unsafe { syscall::fmap(FD_ANONYMOUS, &syscall::Map { offset: 0, size: phdrs_size_aligned, address: 0, flags: MapFlags::PROT_WRITE | MapFlags::MAP_PRIVATE })? };
    read_all(fd, header.e_phoff, unsafe { core::slice::from_raw_parts_mut(phdrs_mem as *mut u8, phdrs_size) })?;

    let phdrs = unsafe { core::slice::from_raw_parts(phdrs_mem as *const ProgramHeader, header.e_phnum as usize) };

    for segment in phdrs {
        let mut flags = syscall::PROT_READ;

        // W ^ X. If it is executable, do not allow it to be writable, even if requested
        if segment.p_flags & program_header::PF_X == program_header::PF_X {
            flags |= syscall::PROT_EXEC;
        } else if segment.p_flags & program_header::PF_W == program_header::PF_W {
            flags |= syscall::PROT_WRITE;
        }

        match segment.p_type {
            program_header::PT_LOAD => {
                let voff = segment.p_vaddr as usize % PAGE_SIZE;
                let vaddr = segment.p_vaddr as usize - voff;
                let size =
                    (segment.p_memsz as usize + voff + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE;

                if segment.p_filesz > segment.p_memsz {
                    return Err(Error::new(ENOEXEC));
                }

                let mem = match tree
                    .range_mut(..=vaddr)
                    .next_back()
                    .filter(|(other_vaddr, entry)| **other_vaddr + entry.size > vaddr)
                {
                    None => unsafe {
                        let mem = syscall::fmap(
                            FD_ANONYMOUS,
                            &syscall::Map {
                                offset: 0,
                                address: 0,
                                size,
                                flags: syscall::PROT_WRITE,
                            },
                        )
                        .map_err(|_| Error::new(ENOMEM))?
                            as *mut u8;
                        tree.insert(
                            vaddr,
                            TreeEntry {
                                size,
                                flags,
                                accessible_addr: mem,
                            },
                        );
                        mem
                    },
                    Some((
                        _,
                        &mut TreeEntry {
                            flags: ref mut f,
                            accessible_addr,
                            ..
                        },
                    )) => {
                        *f |= flags;
                        accessible_addr
                    }
                };
                read_all(fd, segment.p_offset, unsafe {
                    core::slice::from_raw_parts_mut(mem.add(voff), segment.p_filesz as usize)
                })?;
            }
            _ => (),
        }
    }
    let (stack_base, mut stack_mem) = unsafe {
        let stack_base = syscall::fmap(FD_ANONYMOUS, &syscall::Map { offset: 0, size: STACK_SIZE, address: 0, flags: MapFlags::PROT_WRITE | MapFlags::PROT_READ | MapFlags::MAP_PRIVATE })? as *mut u8;
        let stack_mem = stack_base.add(STACK_SIZE).sub(256);

        (stack_base, stack_mem)
    };

    tree.insert(STACK_TOP - STACK_SIZE, TreeEntry {
        size: STACK_SIZE,
        flags: MapFlags::PROT_READ | MapFlags::PROT_WRITE | MapFlags::MAP_PRIVATE,
        accessible_addr: stack_base,
    });
    let mut stack_mem = stack_mem.cast::<usize>();

    let target_phdr_address = find_free_target_addr(&tree, phdrs_size_aligned).ok_or(Error::new(ENOMEM))?;
    tree.insert(target_phdr_address, TreeEntry {
        size: phdrs_size_aligned,
        accessible_addr: phdrs_mem as *mut u8,
        flags: MapFlags::PROT_READ | MapFlags::MAP_PRIVATE,
    });

    let mut sp = STACK_TOP - 256;

    let mut push = |word: usize| unsafe {
        sp -= core::mem::size_of::<usize>();
        stack_mem = stack_mem.sub(1);
        stack_mem.write(word);
    };

    push(0);
    push(AT_NULL);
    push(instruction_ptr);
    push(AT_ENTRY);
    push(target_phdr_address);
    push(AT_PHDR);
    push(header.e_phnum as usize);
    push(AT_PHNUM);
    push(header.e_phentsize as usize);
    push(AT_PHENT);

    let args_envs_size_aligned = (total_args_envs_size+PAGE_SIZE-1)/PAGE_SIZE*PAGE_SIZE;
    let target_args_env_address = find_free_target_addr(&tree, args_envs_size_aligned).ok_or(Error::new(ENOMEM))?;

    unsafe {
        let map = syscall::Map {
            offset: 0,
            flags: MapFlags::PROT_READ | MapFlags::PROT_WRITE | MapFlags::MAP_PRIVATE,
            address: 0,
            size: args_envs_size_aligned,
        };
        let ptr = syscall::fmap(FD_ANONYMOUS, &map)? as *mut u8;
        let args_envs_region = core::slice::from_raw_parts_mut(ptr, total_args_envs_size);
        let mut offset = 0;

        for collection in &[envs, args] {
            push(0);

            for source_slice in collection.iter().rev().copied() {
                push(target_args_env_address + offset);
                args_envs_region[offset..offset + source_slice.len()].copy_from_slice(source_slice);
                offset += source_slice.len() + 1;
            }
        }

        tree.insert(target_args_env_address, TreeEntry {
            accessible_addr: ptr,
            size: args_envs_size_aligned,
            flags: MapFlags::PROT_READ | MapFlags::MAP_PRIVATE,
        });
    }
    push(args.len());

    const STACK_TOP: usize = (1 << 47);
    const STACK_SIZE: usize = 1024 * 1024;

    let memranges = tree
        .into_iter()
        .map(|(address, mut tree_entry)| {
            // Prevent use-after-free
            let old_address = core::mem::replace(&mut tree_entry.accessible_addr, core::ptr::null_mut()) as usize;

            ExecMemRange {
                address,
                size: tree_entry.size,
                flags: tree_entry.flags.bits(),
                old_address,
            }
        })
        .collect::<Vec<_>>();

    /*unsafe {
        let stack = &*(stack_mem as *const crate::start::Stack);

    }*/

    unsafe { crate::ld_so::tcb::Tcb::deactivate(); }

    // TODO: Restore old name if exec failed?
    if let Ok(fd) = syscall::open("thisproc:current/name", syscall::O_WRONLY) {
        let _ = syscall::write(fd, path);
        let _ = syscall::close(fd);
    }

    syscall::exec(&memranges, instruction_ptr, sp)?;
    unreachable!();
}
