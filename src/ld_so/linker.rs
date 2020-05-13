use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    string::{String, ToString},
    vec::Vec,
};
use core::{
    mem::{size_of, transmute},
    ptr, slice,
};
use goblin::{
    elf::{
        program_header,
        r#dyn::{Dyn, DT_DEBUG},
        reloc, sym, Elf,
    },
    error::{Error, Result},
};

use crate::{
    c_str::CString,
    fs::File,
    header::{fcntl, sys_mman, unistd},
    io::Read,
    platform::types::c_void,
};

use super::{
    debug::{RTLDDebug, RTLDState, _dl_debug_state, _r_debug},
    tcb::{Master, Tcb},
    PAGE_SIZE,
};

#[cfg(target_os = "redox")]
const PATH_SEP: char = ';';

#[cfg(target_os = "linux")]
const PATH_SEP: char = ':';

pub struct DSO {
    pub name: String,
    pub base_addr: usize,
    pub entry_point: usize,
}
#[derive(Default, Debug)]
pub struct DepTree {
    pub name: String,
    pub deps: Vec<DepTree>,
}

impl DepTree {
    fn new(name: String) -> DepTree {
        DepTree {
            name,
            deps: Vec::new(),
        }
    }
}
pub struct Linker {
    // Used by load
    /// Library path to search when loading library by name
    library_path: String,
    /// Loaded library raw data
    objects: BTreeMap<String, Box<[u8]>>,

    // Used by link
    /// Global symbols
    globals: BTreeMap<String, usize>,
    /// Weak symbols
    weak_syms: BTreeMap<String, usize>,
    /// Loaded library in-memory data
    mmaps: BTreeMap<String, &'static mut [u8]>,
    verbose: bool,
    tls_index_offset: usize,
    /// A set used to detect circular dependencies in the Linker::load function
    cir_dep: BTreeSet<String>,
    /// Each object will have its children callec once with no repetition.
    dep_tree: DepTree,
}

impl Linker {
    pub fn new(library_path: &str, verbose: bool) -> Self {
        Self {
            library_path: library_path.to_string(),
            objects: BTreeMap::new(),
            globals: BTreeMap::new(),
            weak_syms: BTreeMap::new(),
            mmaps: BTreeMap::new(),
            verbose,
            tls_index_offset: 0,
            cir_dep: BTreeSet::new(),
            dep_tree: Default::default(),
        }
    }

    pub fn load(&mut self, name: &str, path: &str) -> Result<()> {
        self.dep_tree = self.load_recursive(name, path)?;
        if self.verbose {
            println!("Dep tree: {:#?}", self.dep_tree);
        }
        return Ok(());
    }

    fn load_recursive(&mut self, name: &str, path: &str) -> Result<DepTree> {
        if self.verbose {
            println!("load {}: {}", name, path);
        }
        if self.cir_dep.contains(name) {
            return Err(Error::Malformed(format!(
                "Circular dependency: {} is a dependency of itself",
                name
            )));
        }

        let mut deps = DepTree::new(name.to_string());
        let mut data = Vec::new();
        self.cir_dep.insert(name.to_string());
        let path_c = CString::new(path)
            .map_err(|err| Error::Malformed(format!("invalid path '{}': {}", path, err)))?;

        {
            let flags = fcntl::O_RDONLY | fcntl::O_CLOEXEC;
            let mut file = File::open(&path_c, flags)
                .map_err(|err| Error::Malformed(format!("failed to open '{}': {}", path, err)))?;

            file.read_to_end(&mut data)
                .map_err(|err| Error::Malformed(format!("failed to read '{}': {}", path, err)))?;
        }
        deps.deps = self.load_data(name, data.into_boxed_slice())?;
        self.cir_dep.remove(name);
        Ok(deps)
    }

    pub fn load_data(&mut self, name: &str, data: Box<[u8]>) -> Result<Vec<DepTree>> {
        let elf = Elf::parse(&data)?;
        //println!("{:#?}", elf);
        let mut deps = Vec::new();
        for library in elf.libraries.iter() {
            if let Some(dep) = self.load_library(library)? {
                deps.push(dep);
            }
        }

        self.objects.insert(name.to_string(), data);

        return Ok(deps);
    }

    pub fn load_library(&mut self, name: &str) -> Result<Option<DepTree>> {
        if self.objects.contains_key(name) {
            // It should be previously resolved so we don't need to worry about it
            Ok(None)
        } else if name.contains('/') {
            Ok(Some(self.load_recursive(name, name)?))
        } else {
            let library_path = self.library_path.clone();
            for part in library_path.split(PATH_SEP) {
                let path = if part.is_empty() {
                    format!("./{}", name)
                } else {
                    format!("{}/{}", part, name)
                };
                if self.verbose {
                    println!("check {}", path);
                }
                let access = unsafe {
                    let path_c = CString::new(path.as_bytes()).map_err(|err| {
                        Error::Malformed(format!("invalid path '{}': {}", path, err))
                    })?;

                    // TODO: Use R_OK | X_OK
                    unistd::access(path_c.as_ptr(), unistd::F_OK) == 0
                };

                if access {
                    return Ok(Some(self.load_recursive(name, &path)?));
                }
            }

            Err(Error::Malformed(format!("failed to locate '{}'", name)))
        }
    }

    fn collect_syms(
        elf: &Elf,
        mmap: &[u8],
        verbose: bool,
    ) -> Result<(BTreeMap<String, usize>, BTreeMap<String, usize>)> {
        let mut globals = BTreeMap::new();
        let mut weak_syms = BTreeMap::new();
        for sym in elf.dynsyms.iter() {
            let bind = sym.st_bind();
            if sym.st_value == 0 || ![sym::STB_GLOBAL, sym::STB_WEAK].contains(&bind) {
                continue;
            }
            let name: String;
            let value: usize;
            if let Some(name_res) = elf.dynstrtab.get(sym.st_name) {
                name = name_res?.to_string();
                value = mmap.as_ptr() as usize + sym.st_value as usize;
            } else {
                continue;
            }
            match sym.st_bind() {
                sym::STB_GLOBAL => {
                    if verbose {
                        println!("  global {}: {:x?} = {:#x}", &name, sym, value);
                    }
                    globals.insert(name, value);
                }
                sym::STB_WEAK => {
                    if verbose {
                        println!("  weak {}: {:x?} = {:#x}", &name, sym, value);
                    }
                    weak_syms.insert(name, value);
                }
                _ => unreachable!(),
            }
        }
        return Ok((globals, weak_syms));
    }

    pub fn get_sym(&self, name: &str) -> Option<usize> {
        if let Some(value) = self.globals.get(name) {
            if self.verbose {
                println!("    sym {} = {:#x}", name, value);
            }
            Some(*value)
        } else if let Some(value) = self.weak_syms.get(name) {
            if self.verbose {
                println!("    sym {} = {:#x}", name, value);
            }
            Some(*value)
        } else {
            if self.verbose {
                println!("    sym {} = undefined", name);
            }
            None
        }
    }
    pub fn run_init(&self) -> Result<()> {
        self.run_init_tree(&self.dep_tree)
    }

    fn run_init_tree(&self, root: &DepTree) -> Result<()> {
        for node in root.deps.iter() {
            self.run_init_tree(node)?;
        }
        if self.verbose {
            println!("init {}", &root.name);
        }
        let mmap = match self.mmaps.get(&root.name) {
            Some(some) => some,
            None => return Ok(()),
        };
        let elf = Elf::parse(self.objects.get(&root.name).unwrap())?;
        for section in elf.section_headers {
            let name = match elf.shdr_strtab.get(section.sh_name) {
                Some(x) => match x {
                    Ok(y) => y,
                    _ => continue,
                },
                _ => continue,
            };
            if name == ".init_array" {
                let addr = mmap.as_ptr() as usize + section.vm_range().start;
                for i in (0..section.sh_size).step_by(8) {
                    unsafe { call_inits_finis(addr + i as usize) };
                }
            }
        }
        return Ok(());
    }

    pub fn run_fini(&self) -> Result<()> {
        self.run_fini_tree(&self.dep_tree)
    }

    fn run_fini_tree(&self, root: &DepTree) -> Result<()> {
        if self.verbose {
            println!("init {}", &root.name);
        }
        let mmap = match self.mmaps.get(&root.name) {
            Some(some) => some,
            None => return Ok(()),
        };
        let elf = Elf::parse(self.objects.get(&root.name).unwrap())?;
        for section in elf.section_headers {
            let name = match elf.shdr_strtab.get(section.sh_name) {
                Some(x) => match x {
                    Ok(y) => y,
                    _ => continue,
                },
                _ => continue,
            };
            if name == ".fini_array" {
                let addr = mmap.as_ptr() as usize + section.vm_range().start;
                for i in (0..section.sh_size).step_by(8) {
                    unsafe { call_inits_finis(addr + i as usize) };
                }
            }
        }
        for node in root.deps.iter() {
            self.run_fini_tree(node)?;
        }
        return Ok(());
    }

    pub fn link(&mut self, primary_opt: Option<&str>, dso: Option<DSO>) -> Result<Option<usize>> {
        unsafe { _r_debug.state = RTLDState::RT_ADD };
        _dl_debug_state();
        let elfs = {
            let mut elfs = BTreeMap::new();
            for (name, data) in self.objects.iter() {
                // Skip already linked libraries
                if !self.mmaps.contains_key(&*name) {
                    elfs.insert(name.as_str(), Elf::parse(&data)?);
                }
            }
            elfs
        };

        // Load all ELF files into memory and find all globals
        let mut tls_primary = 0;
        let mut tls_size = 0;
        for (elf_name, elf) in elfs.iter() {
            if self.verbose {
                println!("map {}", elf_name);
            }
            let object = match self.objects.get(*elf_name) {
                Some(some) => some,
                None => continue,
            };
            // data for struct LinkMap
            let mut l_ld = 0;
            // Calculate virtual memory bounds
            let bounds = {
                let mut bounds_opt: Option<(usize, usize)> = None;
                for ph in elf.program_headers.iter() {
                    let voff = ph.p_vaddr as usize % PAGE_SIZE;
                    let vaddr = ph.p_vaddr as usize - voff;
                    let vsize =
                        ((ph.p_memsz as usize + voff + PAGE_SIZE - 1) / PAGE_SIZE) * PAGE_SIZE;

                    match ph.p_type {
                        program_header::PT_DYNAMIC => {
                            l_ld = ph.p_vaddr;
                        }
                        program_header::PT_LOAD => {
                            if self.verbose {
                                println!("  load {:#x}, {:#x}: {:x?}", vaddr, vsize, ph);
                            }
                            if let Some(ref mut bounds) = bounds_opt {
                                if vaddr < bounds.0 {
                                    bounds.0 = vaddr;
                                }
                                if vaddr + vsize > bounds.1 {
                                    bounds.1 = vaddr + vsize;
                                }
                            } else {
                                bounds_opt = Some((vaddr, vaddr + vsize));
                            }
                        }
                        program_header::PT_TLS => {
                            if self.verbose {
                                println!("  load tls {:#x}: {:x?}", vsize, ph);
                            }
                            tls_size += vsize;
                            if Some(*elf_name) == primary_opt {
                                tls_primary += vsize;
                            }
                        }
                        _ => (),
                    }
                }
                match bounds_opt {
                    Some(some) => some,
                    None => continue,
                }
            };
            if self.verbose {
                println!("  bounds {:#x}, {:#x}", bounds.0, bounds.1);
            }
            // Allocate memory
            let mmap = unsafe {
                let same_elf = if let Some(prog) = dso.as_ref() {
                    if prog.name == *elf_name {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                if same_elf {
                    let addr = dso.as_ref().unwrap().base_addr;
                    let size = bounds.1;
                    // Fill the gaps i the binary
                    let mut ranges = Vec::new();
                    for ph in elf.program_headers.iter() {
                        if ph.p_type == program_header::PT_LOAD {
                            let voff = ph.p_vaddr as usize % PAGE_SIZE;
                            let vaddr = ph.p_vaddr as usize - voff;
                            let vsize = ((ph.p_memsz as usize + voff + PAGE_SIZE - 1) / PAGE_SIZE)
                                * PAGE_SIZE;
                            ranges.push((vaddr, vsize));
                        }
                    }
                    ranges.sort();
                    let mut start = addr;
                    for (vaddr, vsize) in ranges.iter() {
                        if start < addr + vaddr {
                            sys_mman::mmap(
                                start as *mut c_void,
                                addr + vaddr - start,
                                //TODO: Make it possible to not specify PROT_EXEC on Redox
                                sys_mman::PROT_READ | sys_mman::PROT_WRITE,
                                sys_mman::MAP_ANONYMOUS | sys_mman::MAP_PRIVATE,
                                -1,
                                0,
                            );
                        }
                        start = addr + vaddr + vsize
                    }
                    sys_mman::mprotect(
                        addr as *mut c_void,
                        size,
                        sys_mman::PROT_READ | sys_mman::PROT_WRITE,
                    );
                    _r_debug.insert_first(addr as usize, &elf_name, addr + l_ld as usize);
                    slice::from_raw_parts_mut(addr as *mut u8, size)
                } else {
                    let size = bounds.1;
                    let ptr = sys_mman::mmap(
                        ptr::null_mut(),
                        size,
                        //TODO: Make it possible to not specify PROT_EXEC on Redox
                        sys_mman::PROT_READ | sys_mman::PROT_WRITE,
                        sys_mman::MAP_ANONYMOUS | sys_mman::MAP_PRIVATE,
                        -1,
                        0,
                    );
                    if ptr as usize == !0
                    /* MAP_FAILED */
                    {
                        return Err(Error::Malformed(format!("failed to map {}", elf_name)));
                    }
                    ptr::write_bytes(ptr as *mut u8, 0, size);
                    _r_debug.insert(ptr as usize, &elf_name, ptr as usize + l_ld as usize);
                    slice::from_raw_parts_mut(ptr as *mut u8, size)
                }
            };
            if self.verbose {
                println!("  mmap {:p}, {:#x}", mmap.as_mut_ptr(), mmap.len());
            }
            let (globals, weak_syms) = Linker::collect_syms(&elf, &mmap, self.verbose)?;
            self.globals.extend(globals.into_iter());
            self.weak_syms.extend(weak_syms.into_iter());
            self.mmaps.insert(elf_name.to_string(), mmap);
        }

        // Allocate TLS
        let mut tcb_opt = if primary_opt.is_some() {
            Some(unsafe { Tcb::new(tls_size)? })
        } else {
            None
        };
        if self.verbose {
            println!("tcb {:x?}", tcb_opt);
        }
        // Copy data
        let mut tls_offset = tls_primary;
        let mut tcb_masters = Vec::new();
        // Insert main image master
        tcb_masters.push(Master {
            ptr: ptr::null_mut(),
            len: 0,
            offset: 0,
        });
        let mut tls_ranges = BTreeMap::new();
        for (elf_name, elf) in elfs.iter() {
            let same_elf = if let Some(prog) = dso.as_ref() {
                if prog.name == *elf_name {
                    true
                } else {
                    false
                }
            } else {
                false
            };
            if same_elf {
                continue;
            }
            let object = match self.objects.get(*elf_name) {
                Some(some) => some,
                None => continue,
            };

            let mmap = match self.mmaps.get_mut(*elf_name) {
                Some(some) => some,
                None => continue,
            };
            if self.verbose {
                println!("load {}", elf_name);
            }
            // Copy data
            for ph in elf.program_headers.iter() {
                let voff = ph.p_vaddr as usize % PAGE_SIZE;
                let vaddr = ph.p_vaddr as usize - voff;
                let vsize = ((ph.p_memsz as usize + voff + PAGE_SIZE - 1) / PAGE_SIZE) * PAGE_SIZE;

                match ph.p_type {
                    program_header::PT_LOAD => {
                        let obj_data = {
                            let range = ph.file_range();
                            match object.get(range.clone()) {
                                Some(some) => some,
                                None => {
                                    return Err(Error::Malformed(format!(
                                        "failed to read {:?}",
                                        range
                                    )))
                                }
                            }
                        };

                        let mmap_data = {
                            let range = ph.p_vaddr as usize..ph.p_vaddr as usize + obj_data.len();
                            match mmap.get_mut(range.clone()) {
                                Some(some) => some,
                                None => {
                                    return Err(Error::Malformed(format!(
                                        "failed to write {:?}",
                                        range
                                    )))
                                }
                            }
                        };
                        if self.verbose {
                            println!(
                                "  copy {:#x}, {:#x}: {:#x}, {:#x}",
                                vaddr,
                                vsize,
                                voff,
                                obj_data.len()
                            );
                        }
                        mmap_data.copy_from_slice(obj_data);
                    }
                    program_header::PT_TLS => {
                        let valign = if ph.p_align > 0 {
                            ((ph.p_memsz + (ph.p_align - 1)) / ph.p_align) * ph.p_align
                        } else {
                            ph.p_memsz
                        } as usize;

                        let mut tcb_master = Master {
                            ptr: unsafe { mmap.as_ptr().add(ph.p_vaddr as usize) },
                            len: ph.p_filesz as usize,
                            offset: tls_size - valign,
                        };
                        if self.verbose {
                            println!(
                                "  tls master {:p}, {:#x}: {:#x}, {:#x}",
                                tcb_master.ptr, tcb_master.len, tcb_master.offset, valign,
                            );
                        }
                        if Some(*elf_name) == primary_opt {
                            tls_ranges.insert(
                                elf_name.to_string(),
                                (self.tls_index_offset, tcb_master.range()),
                            );
                            tcb_masters[0] = tcb_master;
                        } else {
                            tcb_master.offset -= tls_offset;
                            tls_offset += vsize;
                            tls_ranges.insert(
                                elf_name.to_string(),
                                (
                                    self.tls_index_offset + tcb_masters.len(),
                                    tcb_master.range(),
                                ),
                            );
                            tcb_masters.push(tcb_master);
                        }
                    }
                    _ => (),
                }
            }
        }

        self.tls_index_offset += tcb_masters.len();

        // Set master images for TLS and copy TLS data
        if let Some(ref mut tcb) = tcb_opt {
            unsafe {
                tcb.set_masters(tcb_masters.into_boxed_slice());
                tcb.copy_masters()?;
            }
        }

        // Perform relocations, and protect pages
        for (elf_name, elf) in elfs.iter() {
            if self.verbose {
                println!("link {}", elf_name);
            }
            // Relocate
            for rel in elf
                .dynrelas
                .iter()
                .chain(elf.dynrels.iter())
                .chain(elf.pltrelocs.iter())
            {
                // println!("  rel {}: {:x?}",
                //     reloc::r_to_str(rel.r_type, elf.header.e_machine),
                //     rel
                // );

                let s = if rel.r_sym > 0 {
                    let sym = elf.dynsyms.get(rel.r_sym).ok_or(Error::Malformed(format!(
                        "missing symbol for relocation {:?}",
                        rel
                    )))?;

                    let name =
                        elf.dynstrtab
                            .get(sym.st_name)
                            .ok_or(Error::Malformed(format!(
                                "missing name for symbol {:?}",
                                sym
                            )))??;
                    self.get_sym(name).unwrap_or(0)
                } else {
                    0
                };

                let a = rel.r_addend.unwrap_or(0) as usize;

                let mmap = match self.mmaps.get_mut(*elf_name) {
                    Some(some) => some,
                    None => continue,
                };

                let b = mmap.as_mut_ptr() as usize;

                let (tm, t) = if let Some((tls_index, tls_range)) = tls_ranges.get(*elf_name) {
                    (*tls_index, tls_range.start)
                } else {
                    (0, 0)
                };

                let ptr = unsafe { mmap.as_mut_ptr().add(rel.r_offset as usize) };

                let set_u64 = |value| {
                    // println!("    set_u64 {:#x}", value);
                    unsafe {
                        *(ptr as *mut u64) = value;
                    }
                };

                match rel.r_type {
                    reloc::R_X86_64_64 => {
                        set_u64((s + a) as u64);
                    }
                    reloc::R_X86_64_DTPMOD64 => {
                        set_u64(tm as u64);
                    }
                    reloc::R_X86_64_DTPOFF64 => {
                        set_u64(rel.r_offset as u64);
                    }
                    reloc::R_X86_64_GLOB_DAT | reloc::R_X86_64_JUMP_SLOT => {
                        set_u64(s as u64);
                    }
                    reloc::R_X86_64_RELATIVE => {
                        set_u64((b + a) as u64);
                    }
                    reloc::R_X86_64_TPOFF64 => {
                        set_u64((s + a).wrapping_sub(t) as u64);
                    }
                    reloc::R_X86_64_IRELATIVE => (), // Handled below
                    _ => {
                        panic!(
                            "    {} unsupported",
                            reloc::r_to_str(rel.r_type, elf.header.e_machine)
                        );
                    }
                }
            }

            // overwrite DT_DEBUG if exist in DYNAMIC segment
            // first we identify the location of DYNAMIC segment
            let mut dyn_start = None;
            let mut debug_start = None;
            for ph in elf.program_headers.iter() {
                if ph.p_type == program_header::PT_DYNAMIC {
                    dyn_start = Some(ph.p_vaddr as usize);
                }
            }
            // next we identify the location of DT_DEBUG in .dynamic section
            if let Some(dynamic) = elf.dynamic.as_ref() {
                let mut i = 0;
                for entry in &dynamic.dyns {
                    if entry.d_tag == DT_DEBUG {
                        debug_start = Some(i as usize);
                        break;
                    }
                    i += 1;
                }
            }
            if let Some(dyn_start_addr) = dyn_start {
                if let Some(i) = debug_start {
                    let mmap = match self.mmaps.get_mut(*elf_name) {
                        Some(some) => some,
                        None => continue,
                    };
                    let bytes: [u8; size_of::<Dyn>() / 2] =
                        unsafe { transmute((&_r_debug) as *const RTLDDebug as usize) };
                    let start = dyn_start_addr + i * size_of::<Dyn>() + size_of::<Dyn>() / 2;
                    mmap[start..start + size_of::<Dyn>() / 2].clone_from_slice(&bytes);
                }
            }

            // Protect pages
            for ph in elf.program_headers.iter() {
                if ph.p_type == program_header::PT_LOAD {
                    let voff = ph.p_vaddr as usize % PAGE_SIZE;
                    let vaddr = ph.p_vaddr as usize - voff;
                    let vsize =
                        ((ph.p_memsz as usize + voff + PAGE_SIZE - 1) / PAGE_SIZE) * PAGE_SIZE;

                    let mut prot = 0;

                    if ph.p_flags & program_header::PF_R == program_header::PF_R {
                        prot |= sys_mman::PROT_READ;
                    }

                    // W ^ X. If it is executable, do not allow it to be writable, even if requested
                    if ph.p_flags & program_header::PF_X == program_header::PF_X {
                        prot |= sys_mman::PROT_EXEC;
                    } else if ph.p_flags & program_header::PF_W == program_header::PF_W {
                        prot |= sys_mman::PROT_WRITE;
                    }

                    let mmap = match self.mmaps.get_mut(*elf_name) {
                        Some(some) => some,
                        None => continue,
                    };
                    let res = unsafe {
                        let ptr = mmap.as_mut_ptr().add(vaddr);
                        if self.verbose {
                            println!("  prot {:#x}, {:#x}: {:p}, {:#x}", vaddr, vsize, ptr, prot);
                        }
                        sys_mman::mprotect(ptr as *mut c_void, vsize, prot)
                    };

                    if res < 0 {
                        return Err(Error::Malformed(format!("failed to mprotect {}", elf_name)));
                    }
                }
            }
        }

        // Activate TLS
        if let Some(ref mut tcb) = tcb_opt {
            unsafe {
                tcb.activate();
            }
        }

        // Perform indirect relocations (necessary evil), gather entry point
        let mut entry_opt = None;
        for (elf_name, elf) in elfs.iter() {
            let mmap = match self.mmaps.get_mut(*elf_name) {
                Some(some) => some,
                None => continue,
            };
            if self.verbose {
                println!("entry {}", elf_name);
            }
            if Some(*elf_name) == primary_opt {
                entry_opt = Some(mmap.as_mut_ptr() as usize + elf.header.e_entry as usize);
            }

            // Relocate
            for rel in elf
                .dynrelas
                .iter()
                .chain(elf.dynrels.iter())
                .chain(elf.pltrelocs.iter())
            {
                // println!("  rel {}: {:x?}",
                //     reloc::r_to_str(rel.r_type, elf.header.e_machine),
                //     rel
                // );

                let a = rel.r_addend.unwrap_or(0) as usize;

                let b = mmap.as_mut_ptr() as usize;

                let ptr = unsafe { mmap.as_mut_ptr().add(rel.r_offset as usize) };

                let set_u64 = |value| {
                    // println!("    set_u64 {:#x}", value);
                    unsafe {
                        *(ptr as *mut u64) = value;
                    }
                };

                if rel.r_type == reloc::R_X86_64_IRELATIVE {
                    unsafe {
                        let f: unsafe extern "C" fn() -> u64 = transmute(b + a);
                        set_u64(f());
                    }
                }
            }

            // Protect pages
            for ph in elf.program_headers.iter() {
                if let program_header::PT_LOAD = ph.p_type {
                    let voff = ph.p_vaddr as usize % PAGE_SIZE;
                    let vaddr = ph.p_vaddr as usize - voff;
                    let vsize =
                        ((ph.p_memsz as usize + voff + PAGE_SIZE - 1) / PAGE_SIZE) * PAGE_SIZE;

                    let mut prot = 0;

                    if ph.p_flags & program_header::PF_R == program_header::PF_R {
                        prot |= sys_mman::PROT_READ;
                    }

                    // W ^ X. If it is executable, do not allow it to be writable, even if requested
                    if ph.p_flags & program_header::PF_X == program_header::PF_X {
                        prot |= sys_mman::PROT_EXEC;
                    } else if ph.p_flags & program_header::PF_W == program_header::PF_W {
                        prot |= sys_mman::PROT_WRITE;
                    }

                    let res = unsafe {
                        let ptr = mmap.as_mut_ptr().add(vaddr);
                        if self.verbose {
                            println!("  prot {:#x}, {:#x}: {:p}, {:#x}", vaddr, vsize, ptr, prot);
                        }
                        sys_mman::mprotect(ptr as *mut c_void, vsize, prot)
                    };

                    if res < 0 {
                        return Err(Error::Malformed(format!("failed to mprotect {}", elf_name)));
                    }
                }
            }
        }
        unsafe { _r_debug.state = RTLDState::RT_CONSISTENT };
        _dl_debug_state();
        Ok(entry_opt)
    }
}

unsafe extern "C" fn call_inits_finis(addr: usize) {
    #[cfg(target_arch = "x86_64")]
    asm!("
        cmp qword ptr [rdi], 0
        je end
        call [rdi]
end:    nop
        "
        :
        :
        :
        : "intel", "volatile"
    );
}
