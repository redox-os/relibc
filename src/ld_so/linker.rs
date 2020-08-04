use alloc::{
    boxed::Box,
    collections::BTreeMap,
    rc::Rc,
    string::{String, ToString},
    vec::Vec,
};
use core::{
    cell::RefCell,
    mem::{size_of, swap, transmute},
    ptr, slice,
};
use goblin::{
    elf::{
        header::ET_DYN,
        program_header,
        r#dyn::{Dyn, DT_DEBUG},
        reloc,
        sym,
        Elf,
    },
    error::{Error, Result},
};

use crate::{
    c_str::CString,
    fs::File,
    header::{fcntl, sys_mman, unistd, errno::STR_ERROR},
    io::Read,
    platform::{errno, types::c_void},
};

use super::{
    access::access,
    callbacks::LinkerCallbacks,
    debug::{RTLDDebug, RTLDState, _dl_debug_state, _r_debug},
    library::{DepTree, Library},
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

#[derive(Clone, Copy, Debug)]
pub struct Symbol {
    pub value: usize,
    pub base: usize,
    pub size: usize,
}
impl Symbol {
    pub fn as_ptr(self) -> *mut c_void {
        (self.base + self.value) as *mut c_void
    }
}

pub struct Linker {
    // Used by load
    /// Library path to search when loading library by name
    library_path: String,
    root: Library,
    verbose: bool,
    tls_index_offset: usize,
    lib_spaces: BTreeMap<usize, Library>,
    counter: usize,
    pub cbs: Rc<RefCell<LinkerCallbacks>>,
}

impl Linker {
    pub fn new(library_path: &str, verbose: bool) -> Self {
        Self {
            library_path: library_path.to_string(),
            root: Library::new(),
            verbose,
            tls_index_offset: 0,
            lib_spaces: BTreeMap::new(),
            counter: 1,
            cbs: Rc::new(RefCell::new(LinkerCallbacks::new())),
        }
    }
    pub fn load(&mut self, name: &str, path: &str) -> Result<()> {
        let mut lib: Library = Library::new();
        swap(&mut lib, &mut self.root);
        lib.dep_tree = self.load_recursive(name, path, &mut lib)?;
        swap(&mut lib, &mut self.root);
        if self.verbose {
            println!("Dep tree: {:#?}", self.root.dep_tree);
        }
        return Ok(());
    }
    pub fn unload(&mut self, libspace: usize) {
        if let Some(lib) = self.lib_spaces.remove(&libspace) {
            for (_, (_, mmap)) in lib.mmaps {
                unsafe { sys_mman::munmap(mmap.as_mut_ptr() as *mut c_void, mmap.len()) };
            }
        }
    }
    fn load_recursive(&mut self, name: &str, path: &str, lib: &mut Library) -> Result<DepTree> {
        if self.verbose {
            println!("load {}: {}", name, path);
        }
        if lib.cir_dep.contains(name) {
            return Err(Error::Malformed(format!(
                "Circular dependency: {} is a dependency of itself",
                name
            )));
        }

        let mut deps = DepTree::new(name.to_string());
        let mut data = Vec::new();
        lib.cir_dep.insert(name.to_string());
        let path_c = CString::new(path)
            .map_err(|err| Error::Malformed(format!("invalid path '{}': {}", path, err)))?;

        {
            let flags = fcntl::O_RDONLY | fcntl::O_CLOEXEC;
            let mut file = File::open(&path_c, flags)
                .map_err(|err| Error::Malformed(format!("failed to open '{}': {}", path, err)))?;

            file.read_to_end(&mut data)
                .map_err(|err| Error::Malformed(format!("failed to read '{}': {}", path, err)))?;
        }
        deps.deps = self.load_data(name, data.into_boxed_slice(), lib)?;
        lib.cir_dep.remove(name);
        Ok(deps)
    }

    fn load_data(
        &mut self,
        name: &str,
        data: Box<[u8]>,
        lib: &mut Library,
    ) -> Result<Vec<DepTree>> {
        let elf = Elf::parse(&data)?;
        //println!("{:#?}", elf);
        let mut deps = Vec::new();
        for library in elf.libraries.iter() {
            if let Some(dep) = self._load_library(library, lib)? {
                deps.push(dep);
            }
        }

        lib.objects.insert(name.to_string(), data);

        return Ok(deps);
    }

    pub fn load_library(&mut self, name: &str) -> Result<usize> {
        let mut lib = Library::new();
        self._load_library(name, &mut lib)?;
        let ret = self.counter;
        self.lib_spaces.insert(ret, lib);
        self.counter += 1;
        return Ok(ret);
    }
    fn _load_library(&mut self, name: &str, lib: &mut Library) -> Result<Option<DepTree>> {
        if lib.objects.contains_key(name) || self.root.objects.contains_key(name) {
            // It should be previously resolved so we don't need to worry about it
            Ok(None)
        } else if name.contains('/') {
            Ok(Some(self.load_recursive(name, name, lib)?))
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
                    // We cannot use unix stdlib because errno is thead local variable
                    // and fs:[0] is not set yet.
                    access(path_c.as_ptr(), unistd::F_OK) == 0
                };

                if access {
                    return Ok(Some(self.load_recursive(name, &path, lib)?));
                }
            }

            Err(Error::Malformed(format!("failed to locate '{}'", name)))
        }
    }

    fn collect_syms(
        elf: &Elf,
        mmap: &[u8],
        verbose: bool,
    ) -> Result<(BTreeMap<String, Symbol>, BTreeMap<String, Symbol>)> {
        let mut globals = BTreeMap::new();
        let mut weak_syms = BTreeMap::new();
        for sym in elf.dynsyms.iter() {
            let bind = sym.st_bind();
            if sym.st_value == 0 || ![sym::STB_GLOBAL, sym::STB_WEAK].contains(&bind) {
                continue;
            }
            let name: String;
            let value: Symbol;
            if let Some(name_res) = elf.dynstrtab.get(sym.st_name) {
                name = name_res?.to_string();
                value = if is_pie_enabled(elf) {
                    Symbol {
                        base: mmap.as_ptr() as usize,
                        value: sym.st_value as usize,
                        size: sym.st_size as usize,
                    }
                } else {
                    Symbol {
                        base: 0,
                        value: sym.st_value as usize,
                        size: sym.st_size as usize,
                    }
                };
            } else {
                continue;
            }
            match sym.st_bind() {
                sym::STB_GLOBAL => {
                    if verbose {
                        println!("  global {}: {:x?} = {:p}", &name, sym, value.as_ptr());
                    }
                    globals.insert(name, value);
                }
                sym::STB_WEAK => {
                    if verbose {
                        println!("  weak {}: {:x?} = {:p}", &name, sym, value.as_ptr());
                    }
                    weak_syms.insert(name, value);
                }
                _ => unreachable!(),
            }
        }
        return Ok((globals, weak_syms));
    }

    pub fn get_sym(&self, name: &str, libspace: Option<usize>) -> Option<Symbol> {
        match libspace {
            Some(id) => {
                let lib = self.lib_spaces.get(&id)?;
                lib.get_sym(name)
            }
            None => self.root.get_sym(name),
        }
    }

    pub fn run_init(&self, libspace: Option<usize>) -> Result<()> {
        match libspace {
            Some(id) => {
                let lib = self.lib_spaces.get(&id).unwrap();
                self.run_tree(&lib, &lib.dep_tree, ".init_array")
            }
            None => self.run_tree(&self.root, &self.root.dep_tree, ".init_array"),
        }
    }

    pub fn run_fini(&self, libspace: Option<usize>) -> Result<()> {
        match libspace {
            Some(id) => {
                let lib = self.lib_spaces.get(&id).unwrap();
                self.run_tree(&lib, &lib.dep_tree, ".fini_array")
            }
            None => {
                //TODO we first need to deinitialize all the loaded libraries first!
                self.run_tree(&self.root, &self.root.dep_tree, ".fini_array")
            }
        }
    }

    fn run_tree(&self, lib: &Library, root: &DepTree, tree_name: &str) -> Result<()> {
        for node in root.deps.iter() {
            self.run_tree(lib, node, tree_name)?;
        }
        if self.verbose {
            println!("running {} {}", tree_name, &root.name);
        }
        let (_, mmap) = match lib.mmaps.get(&root.name) {
            Some(some) => some,
            None => return Ok(()),
        };
        let elf = Elf::parse(lib.objects.get(&root.name).unwrap())?;
        for section in &elf.section_headers {
            let name = match elf.shdr_strtab.get(section.sh_name) {
                Some(x) => match x {
                    Ok(y) => y,
                    _ => continue,
                },
                _ => continue,
            };
            if name == tree_name {
                let addr = if is_pie_enabled(&elf) {
                    mmap.as_ptr() as usize + section.vm_range().start
                } else {
                    section.vm_range().start
                };
                for i in (0..section.sh_size).step_by(8) {
                    unsafe { call_inits_finis(addr + i as usize) };
                }
            }
        }
        return Ok(());
    }

    pub fn link(
        &mut self,
        primary_opt: Option<&str>,
        dso: Option<DSO>,
        libspace: Option<usize>,
    ) -> Result<Option<usize>> {
        match libspace {
            Some(id) => {
                let mut lib = self.lib_spaces.remove(&id).unwrap();
                let res = self._link(primary_opt, dso, &mut lib);
                self.lib_spaces.insert(id, lib);
                res
            }
            None => {
                let mut lib = Library::new();
                swap(&mut lib, &mut self.root);
                let res = self._link(primary_opt, dso, &mut lib);
                swap(&mut lib, &mut self.root);
                res
            }
        }
    }

    pub fn _link(
        &mut self,
        primary_opt: Option<&str>,
        dso: Option<DSO>,
        lib: &mut Library,
    ) -> Result<Option<usize>> {
        unsafe { _r_debug.state = RTLDState::RT_ADD };
        _dl_debug_state();
        let elfs = {
            let mut elfs = BTreeMap::new();
            for (name, data) in lib.objects.iter() {
                // Skip already linked libraries
                if !lib.mmaps.contains_key(&*name) && !self.root.mmaps.contains_key(&*name) {
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
            let object = match lib.objects.get(*elf_name) {
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
                    let size = if is_pie_enabled(&elf) {
                        bounds.1
                    } else {
                        bounds.1 - bounds.0
                    };

                    // Fill the gaps i the binary
                    let mut ranges = Vec::new();
                    for ph in elf.program_headers.iter() {
                        if ph.p_type == program_header::PT_LOAD {
                            let voff = ph.p_vaddr as usize % PAGE_SIZE;
                            let vaddr = ph.p_vaddr as usize - voff;
                            let vsize = ((ph.p_memsz as usize + voff + PAGE_SIZE - 1) / PAGE_SIZE)
                                * PAGE_SIZE;
                            if is_pie_enabled(&elf) {
                                ranges.push((vaddr, vsize));
                            } else {
                                ranges.push((vaddr - addr, vsize));
                            }
                        }
                    }
                    ranges.sort();
                    let mut start = addr;
                    for (vaddr, vsize) in ranges.iter() {
                        if start < addr + vaddr {
                            println!("mmap({:#x}, {})", start, addr + vaddr - start);
                            let mut flags = sys_mman::MAP_ANONYMOUS | sys_mman::MAP_PRIVATE;
                            if start != 0 {
                                flags |= sys_mman::MAP_FIXED_NOREPLACE;
                            }
                            let ptr = sys_mman::mmap(
                                start as *mut c_void,
                                addr + vaddr - start,
                                //TODO: Make it possible to not specify PROT_EXEC on Redox
                                sys_mman::PROT_READ | sys_mman::PROT_WRITE,
                                flags,
                                -1,
                                0,
                            );
                            if ptr as usize == !0 /* MAP_FAILED */ {
                                return Err(Error::Malformed(format!("failed to map {}. errno: {}", elf_name, STR_ERROR[errno as usize])));
                            }
                            if start as *mut c_void != ptr::null_mut() {
                                assert_eq!(ptr, start as *mut c_void, "mmap must always map on the destination we requested");
                            }
                        }
                        start = addr + vaddr + vsize
                    }
                    sys_mman::mprotect(
                        addr as *mut c_void,
                        size,
                        sys_mman::PROT_READ | sys_mman::PROT_WRITE,
                    );
                    _r_debug.insert_first(addr as usize, &elf_name, addr + l_ld as usize);
                    (addr as usize, slice::from_raw_parts_mut(addr as *mut u8, size))
                } else {
                    let (start, end) = bounds;
                    let size = end - start;
                    println!("mmap({:#x}, {})", start, size);
                    let mut flags = sys_mman::MAP_ANONYMOUS | sys_mman::MAP_PRIVATE;
                    if start != 0 {
                        flags |= sys_mman::MAP_FIXED_NOREPLACE;
                    }
                    let ptr = sys_mman::mmap(
                        start as *mut c_void,
                        size,
                        //TODO: Make it possible to not specify PROT_EXEC on Redox
                        sys_mman::PROT_READ | sys_mman::PROT_WRITE,
                        flags,
                        -1,
                        0,
                    );
                    if ptr as usize == !0 /* MAP_FAILED */ {
                        return Err(Error::Malformed(format!("failed to map {}. errno: {}", elf_name, STR_ERROR[errno as usize])));
                    }
                    if start as *mut c_void != ptr::null_mut() {
                        assert_eq!(ptr, start as *mut c_void, "mmap must always map on the destination we requested");
                    }
                    ptr::write_bytes(ptr as *mut u8, 0, size);
                    _r_debug.insert(ptr as usize, &elf_name, ptr as usize + l_ld as usize);
                    (start, slice::from_raw_parts_mut(ptr as *mut u8, size))
                }
            };
            if self.verbose {
                println!("  mmap {:p}, {:#x}", mmap.1.as_mut_ptr(), mmap.1.len());
            }
            let (globals, weak_syms) = Linker::collect_syms(&elf, &mmap.1, self.verbose)?;
            lib.globals.extend(globals.into_iter());
            lib.weak_syms.extend(weak_syms.into_iter());
            lib.mmaps.insert(elf_name.to_string(), mmap);
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
            let object = match lib.objects.get(*elf_name) {
                Some(some) => some,
                None => continue,
            };

            let &mut (base_addr, ref mut mmap) = match lib.mmaps.get_mut(*elf_name) {
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
                                        "failed to read {:x?}",
                                        range
                                    )))
                                }
                            }
                        };

                        let mmap_data = {
                            let range = ph.p_vaddr as usize - base_addr..ph.p_vaddr as usize + obj_data.len() - base_addr;
                            match mmap.get_mut(range.clone()) {
                                Some(some) => some,
                                None => {
                                    println!("mmap: {}", mmap.len());
                                    return Err(Error::Malformed(format!(
                                        "failed to write {:x?}",
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

                let symbol = if rel.r_sym > 0 {
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
                    lib.get_sym(name)
                       .or_else(|| self.root.get_sym(name))
                } else {
                    None
                };

                let s = symbol.as_ref().map(|sym| sym.as_ptr() as usize).unwrap_or(0);

                let a = rel.r_addend.unwrap_or(0) as usize;

                let (_, mmap) = match lib.mmaps.get_mut(*elf_name) {
                    Some(some) => some,
                    None => continue,
                };

                let b = mmap.as_mut_ptr() as usize;

                let (tm, t) = if let Some((tls_index, tls_range)) = tls_ranges.get(*elf_name) {
                    (*tls_index, tls_range.start)
                } else {
                    (0, 0)
                };

                let ptr = if is_pie_enabled(&elf) {
                    unsafe { mmap.as_mut_ptr().add(rel.r_offset as usize) }
                } else {
                    rel.r_offset as *mut u8
                };

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
                        if s != 0 {
                            set_u64((s - b) as u64);
                        } else {
                            set_u64(s as u64);
                        }
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
                    reloc::R_X86_64_COPY => unsafe {
                        // TODO: Make this work
                        let sym = symbol.as_ref().expect("R_X86_64_COPY called without valid symbol");
                        ptr::copy_nonoverlapping(sym.as_ptr() as *const u8, ptr, sym.size as usize);
                    }
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
                    let (_, mmap) = match lib.mmaps.get_mut(*elf_name) {
                        Some(some) => some,
                        None => continue,
                    };
                    let bytes: [u8; size_of::<Dyn>() / 2] =
                        unsafe { transmute((&_r_debug) as *const RTLDDebug as usize) };
                    let start = if is_pie_enabled(elf) {
                        dyn_start_addr + i * size_of::<Dyn>() + size_of::<Dyn>() / 2
                    } else {
                        dyn_start_addr + i * size_of::<Dyn>() + size_of::<Dyn>() / 2
                            - mmap.as_mut_ptr() as usize
                    };
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

                    let (_, mmap) = match lib.mmaps.get_mut(*elf_name) {
                        Some(some) => some,
                        None => continue,
                    };
                    let res = unsafe {
                        let ptr = if is_pie_enabled(elf) {
                            mmap.as_mut_ptr().add(vaddr)
                        } else {
                            vaddr as *const u8
                        };
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
            let (_, mmap) = match lib.mmaps.get_mut(*elf_name) {
                Some(some) => some,
                None => continue,
            };
            if self.verbose {
                println!("entry {}", elf_name);
            }
            if Some(*elf_name) == primary_opt {
                if is_pie_enabled(&elf) {
                    entry_opt = Some(mmap.as_mut_ptr() as usize + elf.header.e_entry as usize);
                } else {
                    entry_opt = Some(elf.header.e_entry as usize);
                }
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
                        let ptr = if is_pie_enabled(&elf) {
                            mmap.as_mut_ptr().add(vaddr)
                        } else {
                            vaddr as *const u8
                        };
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

unsafe fn call_inits_finis(addr: usize) {
    let func = transmute::<usize, *const Option<extern "C" fn()>>(addr);
    (*func).map(|x| x());
}

fn is_pie_enabled(elf: &Elf) -> bool {
    if elf.header.e_type == ET_DYN {
        true
    } else {
        false
    }
}
