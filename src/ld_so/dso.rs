//! See:
//! * <https://refspecs.linuxfoundation.org/elf/elf.pdf>
//! * <https://www.akkadia.org/drepper/dsohowto.pdf>

use object::{
    elf::{self, Dyn64, FileHeader64},
    read::elf::{
        Dyn as _, ElfFile64, FileHeader, GnuHashTable, HashTable as SysVHashTable, ProgramHeader,
        Sym, SymbolTable, Version, VersionTable,
    },
    Endianness, NativeEndian, Object, ObjectSection, ReadRef, SectionKind, StringTable,
    SymbolIndex,
};

use super::{
    debug::_r_debug,
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
    mem::transmute,
    ptr::{self, NonNull},
    slice,
};
#[cfg(target_pointer_width = "32")]
use goblin::elf32::{
    header::ET_DYN,
    program_header,
    section_header::{SHT_FINI_ARRAY, SHT_INIT_ARRAY},
};
#[cfg(target_pointer_width = "64")]
use goblin::elf64::{
    header::ET_DYN,
    program_header,
    section_header::{SHT_FINI_ARRAY, SHT_INIT_ARRAY},
};
use goblin::error::{Error, Result};

type Dyn = Dyn64<NativeEndian>;

enum HashTable<'a, Elf: FileHeader> {
    Gnu(GnuHashTable<'a, Elf>),
    Sysv(SysVHashTable<'a, Elf>),
}

impl<'a, Elf> HashTable<'a, Elf>
where
    Elf: FileHeader,
{
    /// Use the hash table to find the symbol table entry with the given name, hash, and version.
    #[inline]
    pub fn find<R: ReadRef<'a>>(
        &self,
        endian: Elf::Endian,
        name: &str,
        version: Option<&Version<'_>>,
        symbols: &SymbolTable<'a, Elf, R>,
        versions: &VersionTable<'a, Elf>,
    ) -> Option<(SymbolIndex, &'a Elf::Sym)> {
        let name = name.as_bytes();

        match self {
            Self::Gnu(hash_table) => {
                let hash = elf::gnu_hash(name);
                hash_table.find(endian, name, hash, version, symbols, versions)
            }

            Self::Sysv(hash_table) => {
                let hash = elf::hash(name);
                hash_table.find(endian, name, hash, version, symbols, versions)
            }
        }
    }
}

pub(super) struct Dynamic<'a> {
    runpath: Option<String>,
    got: Option<NonNull<usize>>,
    needed: Vec<&'a str>,
    pub(super) jmprel: usize,
    hash_table: HashTable<'a, FileHeader64<Endianness>>,
    pub(super) dynstrtab: StringTable<'a>,
    soname: Option<&'a str>,
}

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum SymbolBinding {
    /// Global symbols are visible to all object files being combined. One
    /// file's definition of a global symbol will satisfy another file's
    /// undefined reference to the same global symbol.
    Global = elf::STB_GLOBAL,
    /// Weak symbols resemble global symbols, but their definitions have lower
    /// precedence.
    Weak = elf::STB_WEAK,
}

impl SymbolBinding {
    #[inline]
    pub fn is_global(&self) -> bool {
        matches!(self, Self::Global)
    }
}

/// Use to represent a library as well as all the symbols that is loaded withen it.
pub struct DSO {
    pub name: String,
    pub id: usize,
    pub dlopened: bool,
    pub entry_point: usize,
    /// Loaded library in-memory data
    pub mmap: &'static [u8],
    pub global_syms: BTreeMap<String, Symbol>,
    pub weak_syms: BTreeMap<String, Symbol>,
    /// .init_array addr and len
    pub init_array: (usize, usize),
    /// .fini_array addr and len
    pub fini_array: (usize, usize),
    pub tls_module_id: usize,
    pub tls_offset: usize,

    pub(super) dynamic: Dynamic<'static>,
    pub(super) symbols: SymbolTable<'static, FileHeader64<Endianness>>,

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
        let elf = ElfFile64::<object::Endianness>::parse(data).unwrap();
        let (mmap, tcb_master, dynamic) =
            DSO::mmap_and_copy(path, &elf, data, base_addr, tls_offset).unwrap();

        // dynamic.hash_table = HashTable::Gnu(
        //     elf.elf_section_table()
        //         .gnu_hash(Endianness::default(), data)
        //         .unwrap()
        //         .unwrap()
        //         .0,
        // );

        let (init_array, fini_array) = DSO::init_fini_arrays(&elf, mmap.as_ptr() as usize);

        let name = match dynamic.soname {
            Some(soname) => soname.to_string(),
            _ => basename(path),
        };
        let tls_offset = match tcb_master {
            Some(ref master) => master.offset,
            _ => 0,
        };
        let entry_point = if is_pie_enabled(&elf) {
            mmap.as_ptr() as usize + elf.entry() as usize
        } else {
            elf.entry() as usize
        };
        let endian = Endianness::default();
        let sections = elf.elf_header().sections(endian, data).unwrap();
        let symbols = sections.symbols(endian, data, elf::SHT_DYNSYM).unwrap();
        let dso = DSO {
            name,
            id,
            dlopened,
            entry_point,
            mmap,
            global_syms: BTreeMap::new(),
            weak_syms: BTreeMap::new(),
            init_array,
            fini_array,
            tls_module_id: if tcb_master.is_some() {
                tls_module_id
            } else {
                0
            },
            tls_offset,

            pie: is_pie_enabled(&elf),
            symbols,
            dynamic,
            scope: Scope::local(),
        };

        Ok((dso, tcb_master))
    }

    /// Global Offset Table
    #[inline]
    pub fn got(&self) -> Option<NonNull<usize>> {
        self.dynamic.got
    }

    #[inline]
    pub fn runpath(&self) -> Option<&String> {
        self.dynamic.runpath.as_ref()
    }

    #[inline]
    pub fn dependencies(&self) -> &[&str] {
        &self.dynamic.needed
    }

    pub fn get_sym(&self, name: &str) -> Option<(Symbol, SymbolBinding)> {
        let endian = Endianness::default();
        let (_index, sym) = self.dynamic.hash_table.find(
            endian,
            name,
            None,
            &self.symbols,
            &VersionTable::default(),
        )?;

        Some((
            Symbol {
                base: if self.pie {
                    self.mmap.as_ptr() as usize
                } else {
                    0
                },
                value: sym.st_value(endian) as usize,
                size: sym.st_size(endian) as usize,
                sym_type: sym.st_type(),
            },
            // TODO(andypython): move this into [`Symbol`]
            match sym.st_bind() {
                elf::STB_GLOBAL => SymbolBinding::Global,
                elf::STB_WEAK => SymbolBinding::Weak,
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

    fn mmap_and_copy<'a>(
        path: &str,
        elf: &ElfFile64<'a>,
        data: &'a [u8],
        base_addr: Option<usize>,
        tls_offset: usize,
    ) -> object::Result<(&'static [u8], Option<Master>, Dynamic<'a>)> {
        let endian = elf.endian();
        trace!("# {}", path);
        // data for struct LinkMap
        let mut l_ld = 0;
        // Calculate virtual memory bounds
        let bounds = {
            let mut bounds_opt: Option<(usize, usize)> = None;
            for ph in elf.elf_program_headers() {
                let voff = ph.p_vaddr(endian) % ph.p_align(endian);
                let vaddr = (ph.p_vaddr(endian) - voff) as usize;
                let vsize = ((ph.p_memsz(endian) + voff) as usize)
                    .next_multiple_of(ph.p_align(endian) as usize);

                match ph.p_type(endian) {
                    program_header::PT_DYNAMIC => {
                        l_ld = ph.p_vaddr(endian);
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
            bounds_opt
                .ok_or(Error::Malformed(
                    "Unable to find PT_LOAD section".to_string(),
                ))
                .unwrap()
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
                .map_err(|e| Error::Malformed(format!("failed to map {}. errno: {}", path, e.0)))
                .unwrap();

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

        let base = if is_pie_enabled(elf) {
            mmap.as_ptr() as usize
        } else {
            0
        };

        // Copy data
        let mut dynamic = None;
        for ph in elf.elf_program_headers() {
            let voff = ph.p_vaddr(endian) % ph.p_align(endian);
            // let vaddr = (ph.p_vaddr - voff) as usize;
            let vsize = ((ph.p_memsz(endian) + voff) as usize)
                .next_multiple_of(ph.p_align(endian) as usize);

            match ph.p_type(endian) {
                program_header::PT_LOAD => {
                    if skip_load_segment_copy {
                        continue;
                    }
                    let obj_data = {
                        let (offset, size) = ph.file_range(endian);
                        let offset = offset as usize;
                        let range = offset..(offset + size as usize);
                        match data.get(range.clone()) {
                            Some(some) => some,
                            None => {
                                return Err(Error::Malformed(format!(
                                    "failed to read {:x?}",
                                    range
                                )))
                                .unwrap()
                            }
                        }
                    };

                    let mmap_data = {
                        let range = if is_pie_enabled(elf) {
                            let addr = ph.p_vaddr(endian) as usize;
                            addr..addr + obj_data.len()
                        } else {
                            let addr = ph.p_vaddr(endian) as usize - mmap.as_ptr() as usize;
                            addr..addr + obj_data.len()
                        };
                        match mmap.get_mut(range.clone()) {
                            Some(some) => some,
                            None => {
                                return Err(Error::Malformed(format!(
                                    "failed to write {:x?}",
                                    range
                                )))
                                .unwrap();
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
                            mmap.as_ptr().add(ph.p_vaddr(endian) as usize)
                        } else {
                            ph.p_vaddr(endian) as *const u8
                        }
                    };
                    tcb_master = Some(Master {
                        ptr,
                        len: ph.p_filesz(endian) as usize,
                        offset: tls_offset + vsize,
                    });
                    trace!("  tcb master {:x?}", tcb_master);
                }

                program_header::PT_DYNAMIC => {
                    dynamic = Some(ph.dynamic(endian, data).unwrap().unwrap())
                }
                _ => (),
            }
        }

        Ok((
            mmap,
            tcb_master,
            Self::parse_dynamic(path, base, endian, mmap, dynamic.unwrap())?,
        ))
    }

    fn parse_dynamic<'a>(
        path: &str,
        base: usize,
        endian: Endianness,
        mmap: &'static [u8],
        entries: &'a [Dyn64<Endianness>],
    ) -> object::Result<Dynamic<'a>> {
        let mut runpath = None;
        let mut got = None;
        let mut needed = vec![];
        let mut jmprel = None;
        let mut soname = None;
        let mut hash_table = None;
        let (mut strtab_offset, mut strtab_size) = (None, None);

        for (i, entry) in entries.iter().enumerate() {
            let val = entry.d_val(endian);
            let tag = entry.d_tag(endian) as u32;

            match tag {
                elf::DT_DEBUG => {
                    // let vaddr = ph.p_vaddr(endian) as usize;
                    // let bytes: [u8; size_of::<Dyn>() / 2] =
                    //     unsafe { transmute((&_r_debug) as *const RTLDDebug as usize) };
                    // let start = if is_pie_enabled(elf) {
                    //     vaddr + i * size_of::<Dyn>() + size_of::<Dyn>() / 2
                    // } else {
                    //     vaddr + i * size_of::<Dyn>() + size_of::<Dyn>() / 2
                    //         - mmap.as_mut_ptr() as usize
                    // };
                    // mmap[start..start + size_of::<Dyn>() / 2].clone_from_slice(&bytes);
                }

                elf::DT_GNU_HASH => {
                    let value = GnuHashTable::parse(endian, &mmap[val as usize..])?;
                    hash_table = Some(HashTable::Gnu(value));
                }

                elf::DT_HASH if hash_table.is_none() => {
                    hash_table = Some(HashTable::Sysv(SysVHashTable::parse(
                        endian,
                        &mmap[val as usize..],
                    )?));
                }

                elf::DT_PLTGOT => {
                    let ptr = NonNull::new(val as *mut usize).expect("DT_PLTGOT is NULL");
                    got = Some(unsafe { ptr.byte_add(base) });
                }

                elf::DT_NEEDED => needed.push(entry),
                elf::DT_JMPREL => jmprel = Some(val as usize),
                elf::DT_RUNPATH => runpath = Some(entry),
                elf::DT_STRTAB => strtab_offset = Some(val),
                elf::DT_STRSZ => strtab_size = Some(val),
                elf::DT_SONAME => soname = Some(entry),

                _ => {}
            }
        }

        let strtab_offset = strtab_offset.expect("mandatory DT_STRTAB not present");
        let strtab_size = strtab_size.expect("mandatory DT_STRSZ not present");

        let dynstrtab = StringTable::new(mmap, strtab_offset, strtab_offset + strtab_size);

        let get_str = |entry: &Dyn64<Endianness>| {
            entry
                .string(endian, dynstrtab)
                .map(|bytes| core::str::from_utf8(bytes).expect("non utf-8 elf symbol name"))
        };

        let needed = needed
            .into_iter()
            .map(get_str)
            .collect::<object::Result<Vec<_>>>()?;

        let base = dirname(path);

        let runpath = runpath
            .map(get_str)
            .transpose()?
            .map(|value| value.replace("$ORIGIN", &base));

        let soname = soname.map(get_str).transpose()?;

        let jmprel = jmprel.unwrap_or_default();
        let hash_table = hash_table.expect("either DT_GNU_HASH and/or DT_HASH mut be present");

        Ok(Dynamic {
            runpath,
            got,
            needed,
            jmprel,
            soname,
            hash_table,
            dynstrtab,
        })
    }

    fn init_fini_arrays(elf: &ElfFile64, mmap_addr: usize) -> ((usize, usize), (usize, usize)) {
        let mut init_array: (usize, usize) = (0, 0);
        let mut fini_array: (usize, usize) = (0, 0);
        for section in elf.sections().filter(|s| {
            matches!(
                s.kind(),
                SectionKind::Elf(SHT_INIT_ARRAY) | SectionKind::Elf(SHT_FINI_ARRAY)
            )
        }) {
            let addr = if is_pie_enabled(elf) {
                mmap_addr + section.address() as usize
            } else {
                section.address() as usize
            };
            if let SectionKind::Elf(SHT_INIT_ARRAY) = section.kind() {
                init_array = (addr, section.size() as usize);
            } else {
                fini_array = (addr, section.size() as usize);
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

pub fn is_pie_enabled(elf: &ElfFile64) -> bool {
    elf.elf_header().e_type.get(elf.endian()) == ET_DYN
}

fn basename(path: &str) -> String {
    path.split("/").last().unwrap_or(path).to_string()
}

fn dirname(path: &str) -> String {
    let mut parts: Vec<&str> = path.split("/").collect();
    parts.truncate(parts.len() - 1);
    parts.join("/")
}
