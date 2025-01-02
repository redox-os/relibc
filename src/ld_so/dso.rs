//! See:
//! * <https://refspecs.linuxfoundation.org/elf/elf.pdf>
//! * <https://www.akkadia.org/drepper/dsohowto.pdf>

use super::{
    debug::{RTLDDebug, _r_debug},
    linker::{Scope, Symbol},
    tcb::Master,
};
use crate::{
    header::sys_mman,
    platform::{types::c_void, Pal, Sys},
};
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};
use core::{
    mem::{size_of, transmute},
    ptr::{self, NonNull},
    slice,
};
#[cfg(target_pointer_width = "32")]
use goblin::elf32::{
    dynamic::{Dyn, DT_DEBUG, DT_RUNPATH},
    header::ET_DYN,
    program_header,
    section_header::{SHN_UNDEF, SHT_FINI_ARRAY, SHT_INIT_ARRAY},
    sym,
};
#[cfg(target_pointer_width = "64")]
use goblin::elf64::{
    dynamic::{Dyn, DT_DEBUG, DT_RUNPATH},
    header::ET_DYN,
    program_header,
    section_header::{SHT_FINI_ARRAY, SHT_INIT_ARRAY},
    sym,
};
use goblin::{
    elf::{
        dynamic::{DT_GNU_HASH, DT_PLTGOT},
        sym::{STB_GLOBAL, STB_WEAK},
        Dynamic, Elf, Sym,
    },
    error::{Error, Result},
    strtab::Strtab,
};

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum SymbolBinding {
    /// Global symbols are visible to all object files being combined. One
    /// file's definition of a global symbol will satisfy another file's
    /// undefined reference to the same global symbol.
    Global = STB_GLOBAL,
    /// Weak symbols resemble global symbols, but their definitions have lower
    /// precedence.
    Weak = STB_WEAK,
}

impl SymbolBinding {
    #[inline]
    pub fn is_global(&self) -> bool {
        matches!(self, Self::Global)
    }
}

enum SymbolHashTable {
    Gnu {
        bloom_filter: NonNull<[u64]>,
        buckets: NonNull<[u32]>,
        chains: NonNull<u32>,
        bloom_shift: u32,
        symbol_off: u32,
    },

    Sysv,
}

impl SymbolHashTable {
    fn find<'a>(&self, name: &str, dynsyms: &'a [Sym], dynstrtab: &'a Strtab) -> Option<&'a Sym> {
        match self {
            Self::Gnu {
                bloom_filter,
                buckets,
                chains,
                bloom_shift,
                symbol_off,
            } => {
                #[cfg(target_pointer_width = "64")]
                const BLOOM_WIDTH: u32 = 64;

                let h1 = self.hash(name);
                let h2 = h1 >> bloom_shift;

                let bloom_filter = unsafe { bloom_filter.as_ref() };
                let mask: u64 = (1 << (h1 & (BLOOM_WIDTH - 1))) | (1 << (h2 & (BLOOM_WIDTH - 1)));
                let bloom_idx = (h1 / BLOOM_WIDTH) & (bloom_filter.len() as u32 - 1);
                if bloom_filter.get(bloom_idx as usize)? & mask != mask {
                    return None;
                }

                let buckets = unsafe { buckets.as_ref() };
                let mut idx = *buckets.get(h1 as usize % buckets.len())?;
                if idx == 0 {
                    return None;
                }

                let chains = chains.as_ptr();
                loop {
                    let h2 = unsafe { chains.add((idx - symbol_off) as usize).read() };
                    if let Some(sym) = dynsyms.get(idx as usize) {
                        let n = dynstrtab.get_at(sym.st_name).unwrap();
                        if n == name && [STB_GLOBAL, STB_WEAK].contains(&sym.st_bind()) {
                            return Some(sym);
                        }
                    }
                    if h2 & 1 != 0 {
                        return None;
                    }
                    idx += 1;
                }
            }

            Self::Sysv => unimplemented!(),
        }
    }

    fn hash(&self, name: &str) -> u32 {
        match self {
            Self::Sysv => {
                let mut h = 0u32;
                let mut g;
                for c in name.chars() {
                    h = (h << 4) + c as u32;
                    g = h & 0xf0000000;
                    if g != 0 {
                        h ^= g >> 24;
                    }
                    h &= !g;
                }
                h
            }

            Self::Gnu { .. } => {
                let mut h = 5381;
                for c in name.chars() {
                    h = (h << 5) + h + c as u32;
                }
                h
            }
        }
    }
}

/// Use to represent a library as well as all the symbols that is loaded withen it.
pub struct DSO {
    pub name: String,
    pub id: usize,
    pub dlopened: bool,
    pub entry_point: usize,
    pub runpath: Option<String>,
    /// Loaded library in-memory data
    pub mmap: &'static [u8],
    pub global_syms: BTreeMap<String, Symbol>,
    pub weak_syms: BTreeMap<String, Symbol>,
    pub dependencies: Vec<String>,
    /// .init_array addr and len
    pub init_array: (usize, usize),
    /// .fini_array addr and len
    pub fini_array: (usize, usize),
    pub tls_module_id: usize,
    pub tls_offset: usize,

    pub dynamic: Option<Dynamic>,
    pub dynsyms: Vec<goblin::elf::sym::Sym>,
    pub dynstrtab: Strtab<'static>,

    hash_table: SymbolHashTable,

    pub scope: Scope,
    /// Position Independent Executable.
    pub pie: bool,
}

unsafe impl Send for DSO {}
unsafe impl Sync for DSO {}

impl DSO {
    pub fn new(
        path: &str,
        data: &'static [u8],
        base_addr: Option<usize>,
        dlopened: bool,
        id: usize,
        tls_module_id: usize,
        tls_offset: usize,
    ) -> Result<(DSO, Option<Master>)> {
        let elf = Elf::parse(data)?;
        let (mmap, tcb_master) = DSO::mmap_and_copy(path, &elf, data, base_addr, tls_offset)?;

        let dynamic = elf.dynamic.as_ref().unwrap();
        let base = if is_pie_enabled(&elf) {
            mmap.as_ptr() as usize
        } else {
            0
        };

        let mut hash_table = None;

        for dynamic in dynamic.dyns.iter() {
            if dynamic.d_tag == DT_GNU_HASH {
                #[derive(Debug)]
                #[repr(C)]
                struct Header {
                    n_buckets: u32,
                    symbol_offset: u32,
                    bloom_size: u32,
                    bloom_shift: u32,
                }

                let (header, bloom_filter, buckets, chains);

                unsafe {
                    let ptr = NonNull::new(dynamic.d_val as *mut u8).unwrap().add(base);
                    header = ptr.cast::<Header>().as_ref();
                    assert!(header.bloom_size.is_power_of_two());
                    bloom_filter = ptr.add(size_of::<Header>()).cast::<u64>();
                    buckets = bloom_filter.add(header.bloom_size as usize).cast::<u32>();
                    chains = buckets.add(header.n_buckets as usize);
                }

                hash_table = Some(SymbolHashTable::Gnu {
                    bloom_filter: NonNull::slice_from_raw_parts(
                        bloom_filter,
                        header.bloom_size as usize,
                    ),
                    buckets: NonNull::slice_from_raw_parts(buckets, header.n_buckets as usize),
                    chains,
                    bloom_shift: header.bloom_shift,
                    symbol_off: header.symbol_offset,
                });
            }
        }

        let (init_array, fini_array) = DSO::init_fini_arrays(&elf, mmap.as_ptr() as usize);

        let name = match elf.soname {
            Some(soname) => soname.to_string(),
            _ => basename(path),
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
            name,
            id,
            dlopened,
            entry_point,
            runpath: DSO::get_runpath(path, &elf)?,
            mmap,
            global_syms: BTreeMap::new(),
            weak_syms: BTreeMap::new(),
            dependencies: elf.libraries.iter().map(|s| s.to_string()).collect(),
            init_array,
            fini_array,
            tls_module_id: if tcb_master.is_some() {
                tls_module_id
            } else {
                0
            },
            tls_offset,

            pie: is_pie_enabled(&elf),
            dynamic: elf.dynamic,
            dynsyms: elf.dynsyms.to_vec(),
            dynstrtab: elf.dynstrtab,

            scope: Scope::local(),
            hash_table: hash_table.expect("required hash table not present"),
        };

        Ok((dso, tcb_master))
    }

    /// Global Offset Table
    pub(super) fn got(&self) -> Option<NonNull<usize>> {
        let dynamic = self.dynamic.as_ref()?;
        let object_base_addr = self.mmap.as_ptr() as u64;

        let got = if let Some(ptr) = {
            dynamic
                .dyns
                .iter()
                .find(|r#dyn| r#dyn.d_tag == DT_PLTGOT)
                .map(|r#dyn| r#dyn.d_val)
        } {
            if self.pie {
                (object_base_addr + ptr) as *mut usize
            } else {
                ptr as *mut usize
            }
        } else {
            assert_eq!(dynamic.info.jmprel, 0);
            return None;
        };

        Some(NonNull::new(got).expect("global offset table"))
    }

    pub fn get_sym(&self, name: &str) -> Option<(Symbol, SymbolBinding)> {
        let sym = self.hash_table.find(name, &self.dynsyms, &self.dynstrtab)?;

        Some((
            Symbol {
                base: if self.pie {
                    self.mmap.as_ptr() as usize
                } else {
                    0
                },
                value: sym.st_value as usize,
                size: sym.st_size as usize,
                sym_type: sym::st_type(sym.st_info),
            },
            // TODO(andypython): move this into [`Symbol`]
            match sym.st_bind() {
                STB_GLOBAL => SymbolBinding::Global,
                STB_WEAK => SymbolBinding::Weak,
                bind => unreachable!("get_sym bind {bind}"),
            },
        ))
    }

    pub fn run_init(&self) {
        unsafe {
            let (addr, size) = self.init_array;
            for i in (0..size).step_by(8) {
                let func = transmute::<usize, *const Option<extern "C" fn()>>(addr + i);
                if let Some(f) = *func {
                    f();
                }
            }
        }
    }

    pub fn run_fini(&self) {
        unsafe {
            let (addr, size) = self.fini_array;
            for i in (0..size).step_by(8).rev() {
                let func = transmute::<usize, *const Option<extern "C" fn()>>(addr + i);
                if let Some(f) = *func {
                    f();
                }
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
                        .get_at(entry.d_val as usize)
                        .ok_or(Error::Malformed("Missing RUNPATH in dynstrtab".to_string()))?;
                    let base = dirname(path);
                    return Ok(Some(runpath.replace("$ORIGIN", &base)));
                }
                _ => return Ok(None),
            }
        }

        Ok(None)
    }

    fn mmap_and_copy(
        path: &str,
        elf: &Elf,
        data: &[u8],
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
                let vsize = ((ph.p_memsz + voff) as usize).next_multiple_of(ph.p_align as usize);

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
                let size = if is_pie_enabled(elf) {
                    bounds.1
                } else {
                    bounds.1 - bounds.0
                };
                _r_debug.insert_first(addr, path, addr + l_ld as usize);
                slice::from_raw_parts_mut(addr as *mut u8, size)
            } else {
                let (start, end) = bounds;
                let size = end - start;
                let mut flags = sys_mman::MAP_ANONYMOUS | sys_mman::MAP_PRIVATE;
                if start != 0 {
                    flags |= sys_mman::MAP_FIXED_NOREPLACE;
                }
                trace!("  mmap({:#x}, {:x}, {:x})", start, size, flags);
                let ptr = Sys::mmap(
                    start as *mut c_void,
                    size,
                    //TODO: Make it possible to not specify PROT_EXEC on Redox
                    sys_mman::PROT_READ | sys_mman::PROT_WRITE,
                    flags,
                    -1,
                    0,
                )
                .map_err(|e| Error::Malformed(format!("failed to map {}. errno: {}", path, e.0)))?;

                if !(start as *mut c_void).is_null() {
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
            // let vaddr = (ph.p_vaddr - voff) as usize;
            let vsize = ((ph.p_memsz + voff) as usize).next_multiple_of(ph.p_align as usize);

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
                        ptr,
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
                        for (i, entry) in dynamic.dyns.iter().enumerate() {
                            if entry.d_tag == DT_DEBUG {
                                debug_start = Some(i);
                                break;
                            }
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

        Ok((mmap, tcb_master))
    }

    fn init_fini_arrays(elf: &Elf, mmap_addr: usize) -> ((usize, usize), (usize, usize)) {
        let mut init_array: (usize, usize) = (0, 0);
        let mut fini_array: (usize, usize) = (0, 0);
        for section in elf
            .section_headers
            .iter()
            .filter(|s| s.sh_type == SHT_INIT_ARRAY || s.sh_type == SHT_FINI_ARRAY)
        {
            let addr = if is_pie_enabled(elf) {
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

        (init_array, fini_array)
    }
}

impl Drop for DSO {
    fn drop(&mut self) {
        self.run_fini();
        unsafe { Sys::munmap(self.mmap.as_ptr() as *mut c_void, self.mmap.len()).unwrap() };
    }
}

pub fn is_pie_enabled(elf: &Elf) -> bool {
    elf.header.e_type == ET_DYN
}

fn basename(path: &str) -> String {
    path.split("/").last().unwrap_or(path).to_string()
}

fn dirname(path: &str) -> String {
    let mut parts: Vec<&str> = path.split("/").collect();
    parts.truncate(parts.len() - 1);
    parts.join("/")
}
