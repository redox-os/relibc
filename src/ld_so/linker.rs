use alloc::{
    collections::BTreeMap,
    rc::Rc,
    string::{String, ToString},
    vec::Vec,
};
use core::{cell::RefCell, mem::transmute, ptr};
use goblin::{
    elf::{program_header, reloc, sym::STT_TLS, Elf},
    error::{Error, Result},
};

use crate::{
    c_str::CString,
    fs::File,
    header::{
        dl_tls::{__tls_get_addr, dl_tls_index},
        fcntl, sys_mman,
        unistd::F_OK,
    },
    io::Read,
    platform::types::c_void,
};

use super::{
    access::accessible,
    callbacks::LinkerCallbacks,
    debug::{RTLDState, _dl_debug_state, _r_debug},
    dso::{is_pie_enabled, DSO},
    tcb::{round_up, Master, Tcb},
    PATH_SEP,
};

#[derive(Clone, Copy, Debug)]
pub struct Symbol {
    pub value: usize,
    pub base: usize,
    pub size: usize,
    pub sym_type: u8,
}

impl Symbol {
    pub fn as_ptr(self) -> *mut c_void {
        (self.base + self.value) as *mut c_void
    }
}

pub struct Linker {
    ld_library_path: Option<String>,
    next_object_id: usize,
    next_tls_module_id: usize,
    tls_size: usize,
    objects: BTreeMap<usize, DSO>,
    name_to_object_id_map: BTreeMap<String, usize>,
    pub cbs: Rc<RefCell<LinkerCallbacks>>,
}

const root_id: usize = 1;

impl Linker {
    pub fn new(ld_library_path: Option<String>) -> Self {
        Self {
            ld_library_path: ld_library_path,
            next_object_id: root_id,
            next_tls_module_id: 0,
            tls_size: 0,
            objects: BTreeMap::new(),
            name_to_object_id_map: BTreeMap::new(),
            cbs: Rc::new(RefCell::new(LinkerCallbacks::new())),
        }
    }

    pub fn load_program(&mut self, path: &str, base_addr: Option<usize>) -> Result<usize> {
        self.load_object(path, &None, base_addr, false)?;
        return Ok(self.objects.get(&root_id).unwrap().entry_point);
    }

    pub fn load_library(&mut self, name: Option<&str>) -> Result<usize> {
        match name {
            Some(name) => {
                if let Some(id) = self.name_to_object_id_map.get(name) {
                    let obj = self.objects.get_mut(id).unwrap();
                    obj.use_count += 1;
                    return Ok(*id);
                } else {
                    let parent_runpath = &self.objects.get(&root_id).unwrap().runpath.clone();
                    let lib_id = self.next_object_id;
                    self.load_object(name, parent_runpath, None, true)?;

                    return Ok(lib_id);
                }
            }
            None => return Ok(root_id),
        }
    }

    pub fn get_sym(&self, lib_id: usize, name: &str) -> Option<*mut c_void> {
        match self.objects.get(&lib_id) {
            Some(obj) => {
                return obj.get_sym(name).map(|(s, strong)| {
                    if s.sym_type != STT_TLS {
                        s.as_ptr()
                    } else {
                        unsafe {
                            let mut tls_index = dl_tls_index {
                                ti_module: obj.tls_module_id as u64,
                                ti_offset: s.value as u64,
                            };
                            __tls_get_addr(&mut tls_index)
                        }
                    }
                });
            }
            _ => {
                return None;
            }
        }
    }

    pub fn unload(&mut self, lib_id: usize) {
        if let Some(obj) = self.objects.get_mut(&lib_id) {
            if obj.dlopened {
                if obj.use_count == 1 {
                    let obj = self.objects.remove(&lib_id).unwrap();
                    for dep in obj.dependencies.iter() {
                        self.unload(*self.name_to_object_id_map.get(dep).unwrap());
                    }
                    self.name_to_object_id_map.remove(&obj.name);
                    drop(obj);
                } else {
                    obj.use_count -= 1;
                }
            }
        }
    }

    pub fn fini(&self) {
        for obj in self.objects.values() {
            obj.run_fini();
        }
    }

    fn load_object(
        &mut self,
        path: &str,
        runpath: &Option<String>,
        base_addr: Option<usize>,
        dlopened: bool,
    ) -> Result<()> {
        unsafe { _r_debug.state = RTLDState::RT_ADD };
        _dl_debug_state();

        let mut new_objects = Vec::new();
        let mut objects_data = Vec::new();
        let mut tcb_masters = Vec::new();
        self.load_objects_recursive(
            path,
            runpath,
            base_addr,
            dlopened,
            &mut new_objects,
            &mut objects_data,
            &mut tcb_masters,
        )?;

        unsafe {
            let tcb = if self.objects.len() == 0 {
                Tcb::new(self.tls_size)?
            } else {
                Tcb::current().unwrap()
            };
            tcb.append_masters(tcb_masters);
            tcb.copy_masters()?;
            tcb.activate();
        }

        self.relocate(&new_objects, &objects_data)?;
        self.run_init(&new_objects);

        for obj in new_objects.into_iter() {
            self.name_to_object_id_map.insert(obj.name.clone(), obj.id);
            self.objects.insert(obj.id, obj);
        }

        unsafe { _r_debug.state = RTLDState::RT_CONSISTENT };
        _dl_debug_state();

        return Ok(());
    }

    fn load_objects_recursive(
        &mut self,
        name: &str,
        parent_runpath: &Option<String>,
        base_addr: Option<usize>,
        dlopened: bool,
        new_objects: &mut Vec<DSO>,
        objects_data: &mut Vec<Vec<u8>>,
        tcb_masters: &mut Vec<Master>,
    ) -> Result<()> {
        if let Some(obj) = {
            if let Some(id) = self.name_to_object_id_map.get(name) {
                self.objects.get_mut(id)
            } else {
                new_objects.iter_mut().find(|o| o.name == name)
            }
        } {
            obj.use_count += 1;
            return Ok(());
        }

        let path = Linker::search_object(name, &self.ld_library_path, parent_runpath)?;
        let data = Linker::read_file(&path)?;
        let (obj, tcb_master) = DSO::new(
            &path,
            &data,
            base_addr,
            dlopened,
            self.next_object_id,
            self.next_tls_module_id,
            self.tls_size,
        )?;
        new_objects.push(obj);
        objects_data.push(data);
        self.next_object_id += 1;

        if let Some(master) = tcb_master {
            self.next_tls_module_id += 1;
            self.tls_size = master.offset;
            tcb_masters.push(master);
        }

        let (runpath, dependencies) = {
            let parent = new_objects.last().unwrap();
            (parent.runpath.clone(), parent.dependencies.clone())
        };
        for dep_name in dependencies.iter() {
            self.load_objects_recursive(
                dep_name,
                &runpath,
                None,
                dlopened,
                new_objects,
                objects_data,
                tcb_masters,
            )?;
        }

        return Ok(());
    }

    fn search_object(
        name: &str,
        ld_library_path: &Option<String>,
        parent_runpath: &Option<String>,
    ) -> Result<String> {
        let mut full_path = name.to_string();
        if accessible(&full_path, F_OK) == 0 {
            return Ok(full_path);
        } else {
            let mut search_paths = Vec::new();
            if let Some(runpath) = parent_runpath {
                search_paths.extend(runpath.split(PATH_SEP));
            }
            if let Some(ld_path) = ld_library_path {
                search_paths.extend(ld_path.split(PATH_SEP));
            }
            search_paths.push("/lib");
            for part in search_paths.iter() {
                full_path = format!("{}/{}", part, name);
                trace!("trying path {}", full_path);
                if accessible(&full_path, F_OK) == 0 {
                    return Ok(full_path);
                }
            }
        }
        return Err(Error::Malformed(format!("failed to locate '{}'", name)));
    }

    fn read_file(path: &str) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let path_c = CString::new(path)
            .map_err(|err| Error::Malformed(format!("invalid path '{}': {}", path, err)))?;
        let flags = fcntl::O_RDONLY | fcntl::O_CLOEXEC;
        let mut file = File::open(&path_c, flags)
            .map_err(|err| Error::Malformed(format!("failed to open '{}': {}", path, err)))?;
        file.read_to_end(&mut data)
            .map_err(|err| Error::Malformed(format!("failed to read '{}': {}", path, err)))?;

        return Ok(data);
    }

    fn relocate(&self, new_objects: &Vec<DSO>, objects_data: &Vec<Vec<u8>>) -> Result<()> {
        let symbols_lookup_objects: Vec<&DSO> =
            self.objects.values().chain(new_objects.iter()).collect();

        // Perform relocations
        for i in (0..new_objects.len()).rev() {
            let elf = Elf::parse(&objects_data[i])?;
            let obj = &new_objects[i];

            trace!("link {}", obj.name);

            let mmap = &obj.mmap;
            let b = mmap.as_ptr() as usize;

            // Relocate
            for rel in elf
                .dynrelas
                .iter()
                .chain(elf.dynrels.iter())
                .chain(elf.pltrelocs.iter())
            {
                trace!(
                    "  rel {}: {:x?}",
                    reloc::r_to_str(rel.r_type, elf.header.e_machine),
                    rel
                );
                let (symbol, t) = if rel.r_sym > 0 {
                    let sym = elf.dynsyms.get(rel.r_sym).ok_or(Error::Malformed(format!(
                        "missing symbol for relocation {:?}",
                        rel
                    )))?;

                    let mut t = 0;
                    let name =
                        elf.dynstrtab
                            .get(sym.st_name)
                            .ok_or(Error::Malformed(format!(
                                "missing name for symbol {:?}",
                                sym
                            )))??;
                    let mut symbol = None;
                    let mut found = false;
                    let lookup_start = match rel.r_type {
                        reloc::R_X86_64_COPY => 1,
                        _ => 0,
                    };
                    for lookup_id in lookup_start..symbols_lookup_objects.len() {
                        let lookup_obj = &symbols_lookup_objects[lookup_id];
                        if let Some((s, strong)) = lookup_obj.get_sym(name) {
                            trace!(
                                "symbol {} from {} found in {} ({})",
                                name, obj.name, lookup_obj.name, if strong { "strong" } else { "weak" }
                            );
                            symbol = Some(s);
                            t = lookup_obj.tls_offset;
                            found = true;
                            // Stop looking if any strong symbol is found
                            if strong {
                                break;
                            }
                        }
                    }
                    // TODO: below doesn't work because of missing __preinit_array_{start,end} and __init_array_{start,end} symbols in dynamic linked programs
                    /*
                    if !found {
                        return Err(Error::Malformed(format!("missing symbol for name {}", name)));
                    }
                    */
                    (symbol, t)
                } else {
                    (None, 0)
                };

                let s = symbol
                    .as_ref()
                    .map(|sym| sym.as_ptr() as usize)
                    .unwrap_or(0);

                let a = rel.r_addend.unwrap_or(0) as usize;

                let ptr = if is_pie_enabled(&elf) {
                    (b + rel.r_offset as usize) as *mut u8
                } else {
                    rel.r_offset as *mut u8
                };
                let set_u64 = |value| {
                    trace!("    set_u64 {:#x}", value);
                    unsafe {
                        *(ptr as *mut u64) = value;
                    }
                };

                match rel.r_type {
                    reloc::R_X86_64_64 => {
                        set_u64((s + a) as u64);
                    }
                    reloc::R_X86_64_DTPMOD64 => {
                        set_u64(obj.tls_module_id as u64);
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
                        if rel.r_sym > 0 {
                            let sym = symbol
                                .as_ref()
                                .expect("R_X86_64_TPOFF64 called without valid symbol");
                            set_u64((sym.value + a).wrapping_sub(t) as u64);
                        } else {
                            set_u64(a.wrapping_sub(t) as u64);
                        }
                    }
                    reloc::R_X86_64_IRELATIVE => unsafe {
                        let f: unsafe extern "C" fn() -> u64 = transmute(b + a);
                        set_u64(f());
                    },
                    reloc::R_X86_64_COPY => unsafe {
                        let sym = symbol
                            .as_ref()
                            .expect("R_X86_64_COPY called without valid symbol");
                        ptr::copy_nonoverlapping(sym.as_ptr() as *const u8, ptr, sym.size as usize);
                    },
                    _ => {
                        panic!(
                            "    {} unsupported",
                            reloc::r_to_str(rel.r_type, elf.header.e_machine)
                        );
                    }
                }
            }

            // Protect pages
            for ph in elf
                .program_headers
                .iter()
                .filter(|ph| ph.p_type == program_header::PT_LOAD)
            {
                let voff = ph.p_vaddr % ph.p_align;
                let vaddr = (ph.p_vaddr - voff) as usize;
                let vsize = round_up((ph.p_memsz + voff) as usize, ph.p_align as usize);
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
                        mmap.as_ptr().add(vaddr)
                    } else {
                        vaddr as *const u8
                    };
                    trace!("  prot {:#x}, {:#x}: {:p}, {:#x}", vaddr, vsize, ptr, prot);
                    sys_mman::mprotect(ptr as *mut c_void, vsize, prot)
                };

                if res < 0 {
                    return Err(Error::Malformed(format!("failed to mprotect {}", obj.name)));
                }
            }
        }

        return Ok(());
    }

    fn run_init(&self, objects: &Vec<DSO>) {
        for obj in objects.iter().rev() {
            obj.run_init();
        }
    }
}
