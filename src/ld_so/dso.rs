use super::{
    debug::{RTLDDebug, _r_debug},
    linker::Symbol,
    tcb::{round_up, Master},
};
use crate::{
    header::{errno::STR_ERROR, sys_mman},
    platform::{errno, types::c_void},
};
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};
use core::{
    mem::{size_of, transmute},
    ptr, slice,
};
use goblin::{
    elf::{
        header::ET_DYN,
        program_header,
        r#dyn::{Dyn, DT_DEBUG, DT_RUNPATH},
        section_header::{SHN_UNDEF, SHT_FINI_ARRAY, SHT_INIT_ARRAY},
        sym, Elf,
    },
    error::{Error, Result},
};

/// Use to represent a library as well as all the symbols that is loaded withen it.
#[derive(Default)]
pub struct DSO {
    pub name: String,
    pub id: usize,
    pub dlopened: bool,
    pub entry_point: usize,
    pub runpath: Option<String>,
    /// Loaded library in-memory data
    pub mmap: &'static mut [u8],
    pub global_syms: BTreeMap<String, Symbol>,
    pub weak_syms: BTreeMap<String, Symbol>,
    pub dependencies: Vec<String>,
    /// .init_array addr and len
    pub init_array: (usize, usize),
    /// .fini_array addr and len
    pub fini_array: (usize, usize),
    pub tls_module_id: usize,
    pub tls_offset: usize,
    pub use_count: usize,
}

impl DSO {
    pub fn new(
        path: &str,
        data: &Vec<u8>,
        base_addr: Option<usize>,
        dlopened: bool,
        id: usize,
        tls_module_id: usize,
        tls_offset: usize,
    ) -> Result<(DSO, Option<Master>)> {
        let elf = Elf::parse(data)?;
        let (mmap, tcb_master) = DSO::mmap_and_copy(&path, &elf, &data, base_addr, tls_offset)?;
        let (global_syms, weak_syms) = DSO::collect_syms(&elf, &mmap)?;
        let (init_array, fini_array) = DSO::init_fini_arrays(&elf, mmap.as_ptr() as usize);

        let name = match elf.soname {
            Some(soname) => soname.to_string(),
            _ => basename(&path),
        };
        let tls_offset = match tcb_master {
            Some(ref master) => master.offset,
            _ => 0,
        };
        let entry_point = if is_pie_enabled(&elf) {
            mmap.as_ptr() as usize + elf.header.e_entry as usize
        } else {
            elf.header.e_entry as usize
        };
        let dso = DSO {
            name: name,
            id: id,
            use_count: 1,
            dlopened: dlopened,
            entry_point: entry_point,
            runpath: DSO::get_runpath(&path, &elf)?,
            mmap: mmap,
            global_syms: global_syms,
            weak_syms: weak_syms,
            dependencies: elf.libraries.iter().map(|s| s.to_string()).collect(),
            init_array: init_array,
            fini_array: fini_array,
            tls_module_id: tls_module_id,
            tls_offset: tls_offset,
        };
        return Ok((dso, tcb_master));
    }

    pub fn get_sym(&self, name: &str) -> Option<(Symbol, bool)> {
        if let Some(value) = self.global_syms.get(name) {
            Some((*value, true))
        } else if let Some(value) = self.weak_syms.get(name) {
            Some((*value, false))
        } else {
            None
        }
    }

    pub fn run_init(&self) {
        unsafe {
            let (addr, size) = self.init_array;
            for i in (0..size).step_by(8) {
                let func = transmute::<usize, *const Option<extern "C" fn()>>(addr + i);
                (*func).map(|x| x());
            }
        }
    }

    pub fn run_fini(&self) {
        unsafe {
            let (addr, size) = self.fini_array;
            for i in (0..size).step_by(8).rev() {
                let func = transmute::<usize, *const Option<extern "C" fn()>>(addr + i);
                (*func).map(|x| x());
            }
        }
    }

    fn get_runpath(path: &str, elf: &Elf) -> Result<Option<String>> {
        if let Some(dynamic) = &elf.dynamic {
            let entry = dynamic.dyns.iter().find(|d| d.d_tag == DT_RUNPATH);
            match entry {
                Some(entry) => {
                    let runpath = elf
                        .dynstrtab
                        .get(entry.d_val as usize)
                        .ok_or(Error::Malformed("Missing RUNPATH in dynstrtab".to_string()))??;
                    let base = dirname(path);
                    return Ok(Some(runpath.replace("$ORIGIN", &base)));
                }
                _ => return Ok(None),
            }
        }
        return Ok(None);
    }

    fn mmap_and_copy(
        path: &str,
        elf: &Elf,
        data: &Vec<u8>,
        base_addr: Option<usize>,
        tls_offset: usize,
    ) -> Result<(&'static mut [u8], Option<Master>)> {
        trace!("# {}", path);
        // data for struct LinkMap
        let mut l_ld = 0;
        // Calculate virtual memory bounds
        let bounds = {
            let mut bounds_opt: Option<(usize, usize)> = None;
            for ph in elf.program_headers.iter() {
                let voff = ph.p_vaddr % ph.p_align;
                let vaddr = (ph.p_vaddr - voff) as usize;
                let vsize = round_up((ph.p_memsz + voff) as usize, ph.p_align as usize);

                match ph.p_type {
                    program_header::PT_DYNAMIC => {
                        l_ld = ph.p_vaddr;
                    }
                    program_header::PT_LOAD => {
                        trace!("  load {:#x}, {:#x}: {:x?}", vaddr, vsize, ph);
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
                    _ => (),
                }
            }
            bounds_opt.ok_or(Error::Malformed(
                "Unable to find PT_LOAD section".to_string(),
            ))?
        };
        trace!("  bounds {:#x}, {:#x}", bounds.0, bounds.1);
        // Allocate memory
        let mmap = unsafe {
            if let Some(addr) = base_addr {
                let size = if is_pie_enabled(&elf) {
                    bounds.1
                } else {
                    bounds.1 - bounds.0
                };
                _r_debug.insert_first(addr as usize, path, addr + l_ld as usize);
                slice::from_raw_parts_mut(addr as *mut u8, size)
            } else {
                let (start, end) = bounds;
                let size = end - start;
                let mut flags = sys_mman::MAP_ANONYMOUS | sys_mman::MAP_PRIVATE;
                if start != 0 {
                    flags |= sys_mman::MAP_FIXED_NOREPLACE;
                }
                trace!("  mmap({:#x}, {:x}, {:x})", start, size, flags);
                let ptr = sys_mman::mmap(
                    start as *mut c_void,
                    size,
                    //TODO: Make it possible to not specify PROT_EXEC on Redox
                    sys_mman::PROT_READ | sys_mman::PROT_WRITE,
                    flags,
                    -1,
                    0,
                );
                if ptr as usize == !0
                /* MAP_FAILED */
                {
                    return Err(Error::Malformed(format!(
                        "failed to map {}. errno: {}",
                        path, STR_ERROR[errno as usize]
                    )));
                }
                if start as *mut c_void != ptr::null_mut() {
                    assert_eq!(
                        ptr, start as *mut c_void,
                        "mmap must always map on the destination we requested"
                    );
                }
                trace!("    = {:p}", ptr);
                ptr::write_bytes(ptr as *mut u8, 0, size);
                _r_debug.insert(ptr as usize, path, ptr as usize + l_ld as usize);
                slice::from_raw_parts_mut(ptr as *mut u8, size)
            }
        };

        let skip_load_segment_copy = base_addr.is_some();
        let mut tcb_master = None;

        // Copy data
        for ph in elf.program_headers.iter() {
            let voff = ph.p_vaddr % ph.p_align;
            let vaddr = (ph.p_vaddr - voff) as usize;
            let vsize = round_up((ph.p_memsz + voff) as usize, ph.p_align as usize);

            match ph.p_type {
                program_header::PT_LOAD => {
                    if skip_load_segment_copy {
                        continue;
                    }
                    let obj_data = {
                        let range = ph.file_range();
                        match data.get(range.clone()) {
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
                        let range = if is_pie_enabled(elf) {
                            let addr = ph.p_vaddr as usize;
                            addr..addr + obj_data.len()
                        } else {
                            let addr = ph.p_vaddr as usize - mmap.as_ptr() as usize;
                            addr..addr + obj_data.len()
                        };
                        match mmap.get_mut(range.clone()) {
                            Some(some) => some,
                            None => {
                                return Err(Error::Malformed(format!(
                                    "failed to write {:x?}",
                                    range
                                )));
                            }
                        }
                    };
                    trace!(
                        "  copy {:#x}, {:#x}: {:#x}, {:#x}",
                        vaddr,
                        vsize,
                        voff,
                        obj_data.len()
                    );
                    mmap_data.copy_from_slice(obj_data);
                }
                program_header::PT_TLS => {
                    let ptr = unsafe {
                        if is_pie_enabled(elf) {
                            mmap.as_ptr().add(ph.p_vaddr as usize)
                        } else {
                            ph.p_vaddr as *const u8
                        }
                    };
                    tcb_master = Some(Master {
                        ptr: ptr,
                        len: ph.p_filesz as usize,
                        offset: tls_offset + vsize,
                    });
                    trace!("  tcb master {:x?}", tcb_master);
                }
                program_header::PT_DYNAMIC => {
                    // overwrite DT_DEBUG if exist in DYNAMIC segment
                    // first we identify the location of DYNAMIC segment
                    let dyn_start = ph.p_vaddr as usize;
                    let mut debug_start = None;
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
                    if let Some(i) = debug_start {
                        let bytes: [u8; size_of::<Dyn>() / 2] =
                            unsafe { transmute((&_r_debug) as *const RTLDDebug as usize) };
                        let start = if is_pie_enabled(elf) {
                            dyn_start + i * size_of::<Dyn>() + size_of::<Dyn>() / 2
                        } else {
                            dyn_start + i * size_of::<Dyn>() + size_of::<Dyn>() / 2
                                - mmap.as_mut_ptr() as usize
                        };
                        mmap[start..start + size_of::<Dyn>() / 2].clone_from_slice(&bytes);
                    }
                }
                _ => (),
            }
        }
        return Ok((mmap, tcb_master));
    }

    fn collect_syms(
        elf: &Elf,
        mmap: &[u8],
    ) -> Result<(BTreeMap<String, Symbol>, BTreeMap<String, Symbol>)> {
        let mut globals = BTreeMap::new();
        let mut weak_syms = BTreeMap::new();
        for sym in elf.dynsyms.iter() {
            let bind = sym.st_bind();
            if sym.st_shndx == SHN_UNDEF as usize
                || ![sym::STB_GLOBAL, sym::STB_WEAK].contains(&bind)
            {
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
                        sym_type: sym::st_type(sym.st_info),
                    }
                } else {
                    Symbol {
                        base: 0,
                        value: sym.st_value as usize,
                        size: sym.st_size as usize,
                        sym_type: sym::st_type(sym.st_info),
                    }
                };
            } else {
                continue;
            }
            match sym.st_bind() {
                sym::STB_GLOBAL => {
                    trace!("  global {}: {:x?} = {:p}", &name, sym, value.as_ptr());
                    globals.insert(name, value);
                }
                sym::STB_WEAK => {
                    trace!("  weak {}: {:x?} = {:p}", &name, sym, value.as_ptr());
                    weak_syms.insert(name, value);
                }
                _ => unreachable!(),
            }
        }
        return Ok((globals, weak_syms));
    }

    fn init_fini_arrays(elf: &Elf, mmap_addr: usize) -> ((usize, usize), (usize, usize)) {
        let mut init_array: (usize, usize) = (0, 0);
        let mut fini_array: (usize, usize) = (0, 0);
        for section in elf
            .section_headers
            .iter()
            .filter(|s| s.sh_type == SHT_INIT_ARRAY || s.sh_type == SHT_FINI_ARRAY)
        {
            let addr = if is_pie_enabled(&elf) {
                mmap_addr + section.vm_range().start
            } else {
                section.vm_range().start
            };
            if section.sh_type == SHT_INIT_ARRAY {
                init_array = (addr, section.sh_size as usize);
            } else {
                fini_array = (addr, section.sh_size as usize);
            }
        }
        return (init_array, fini_array);
    }
}

impl Drop for DSO {
    fn drop(&mut self) {
        self.run_fini();
        unsafe { sys_mman::munmap(self.mmap.as_mut_ptr() as *mut c_void, self.mmap.len()) };
    }
}

pub fn is_pie_enabled(elf: &Elf) -> bool {
    return elf.header.e_type == ET_DYN;
}

fn basename(path: &str) -> String {
    return path.split("/").last().unwrap_or(path).to_string();
}

fn dirname(path: &str) -> String {
    let mut parts: Vec<&str> = path.split("/").collect();
    parts.truncate(parts.len() - 1);
    return parts.join("/");
}
