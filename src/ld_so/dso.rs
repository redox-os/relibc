//! See:
//! * <https://refspecs.linuxfoundation.org/elf/elf.pdf>
//! * <https://www.akkadia.org/drepper/dsohowto.pdf>

use object::{
    elf,
    read::elf::{
        Dyn as _, GnuHashTable, HashTable as SysVHashTable, ProgramHeader as _, Rel as _,
        Rela as _, Sym as _, Version, VersionTable,
    },
    NativeEndian, Object, StringTable, SymbolIndex,
};

use super::{
    debug::{RTLDDebug, _r_debug},
    linker::{Resolve, Scope, Symbol, __plt_resolve_trampoline, GLOBAL_SCOPE},
    tcb::Master,
};
use crate::{
    header::sys_mman,
    platform::{types::c_void, Pal, Sys},
};
use alloc::{
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use core::{
    ffi::c_char,
    mem::size_of,
    ptr::{self, NonNull},
    slice,
};

pub const CHAR_BITS: usize = size_of::<c_char>() * 8;
pub type Relr = usize;

#[cfg(target_pointer_width = "32")]
mod shim {
    use object::{elf::*, read::elf::ElfFile32, NativeEndian};
    pub type Dyn = Dyn32<NativeEndian>;
    pub type Rel = Rel32<NativeEndian>;
    pub type Rela = Rela32<NativeEndian>;
    pub type Sym = Sym32<NativeEndian>;
    pub type FileHeader = FileHeader32<NativeEndian>;
    pub type ProgramHeader = ProgramHeader32<NativeEndian>;
    pub type ElfFile<'a> = ElfFile32<'a, NativeEndian>;
}

#[cfg(target_pointer_width = "64")]
mod shim {
    use object::{elf::*, read::elf::ElfFile64, NativeEndian};
    pub type Dyn = Dyn64<NativeEndian>;
    pub type Rel = Rel64<NativeEndian>;
    pub type Rela = Rela64<NativeEndian>;
    pub type Sym = Sym64<NativeEndian>;
    pub type FileHeader = FileHeader64<NativeEndian>;
    pub type ProgramHeader = ProgramHeader64<NativeEndian>;
    pub type ElfFile<'a> = ElfFile64<'a, NativeEndian>;
}

pub use shim::*;

enum HashTable<'a> {
    Gnu(GnuHashTable<'a, FileHeader>),
    Sysv(SysVHashTable<'a, FileHeader>),
}

impl<'a> HashTable<'a> {
    /// Use the hash table to find the symbol table entry with the given name, hash, and version.
    #[inline]
    pub fn find(
        &self,
        name: &str,
        version: Option<&Version<'_>>,
        symbols: &'a [Sym],
        strings: StringTable<'a>,
        versions: &VersionTable<'a, FileHeader>,
    ) -> Option<(SymbolIndex, &'a Sym)> {
        let name = name.as_bytes();

        match self {
            Self::Gnu(hash_table) => {
                let hash = elf::gnu_hash(name);
                hash_table.find(
                    NativeEndian,
                    name,
                    hash,
                    version,
                    symbols,
                    strings,
                    versions,
                )
            }

            Self::Sysv(hash_table) => {
                let hash = elf::hash(name);
                hash_table.find(
                    NativeEndian,
                    name,
                    hash,
                    version,
                    symbols,
                    strings,
                    versions,
                )
            }
        }
    }

    fn symbol_table_length(&self) -> usize {
        match self {
            Self::Gnu(hash_table) => hash_table
                .symbol_table_length(NativeEndian)
                .expect("empty GNU symbol hash table")
                as usize,
            Self::Sysv(hash_table) => hash_table.symbol_table_length() as usize,
        }
    }
}

type InitFn = unsafe extern "C" fn();

pub(super) struct Dynamic<'data> {
    runpath: Option<String>,
    got: Option<NonNull<usize>>,
    needed: Vec<&'data str>,
    pub(super) jmprel: usize,
    hash_table: HashTable<'data>,
    pub(super) dynstrtab: StringTable<'data>,
    soname: Option<&'data str>,
    init_array: &'data [unsafe extern "C" fn()],
    fini_array: &'data [unsafe extern "C" fn()],
    rela: &'data [Rela],
    relr: &'data [Relr],
    rel: &'data [Rel],
    symbols: &'data [Sym],
    explicit_addend: bool,
    pltrelsz: usize,
}

impl<'data> Dynamic<'data> {
    pub fn symbol(&self, index: SymbolIndex) -> Option<&'data Sym> {
        // Symbol table entry for index 0 is reserved.
        assert!(index != SymbolIndex(0));
        self.symbols.get(index.0)
    }

    fn symbol_name(&self, index: SymbolIndex) -> Option<&'data str> {
        let sym = self.symbol(index)?;
        let name = sym.name(NativeEndian, self.dynstrtab).ok()?;
        Some(core::str::from_utf8(name).expect("non UTF-8 ELF symbol name"))
    }

    fn static_relocations(&self) -> impl Iterator<Item = Relocation> + '_ {
        self.rela
            .iter()
            .map(Relocation::from)
            .chain(self.rel.iter().map(Relocation::from))
    }
}

unsafe impl Send for Dynamic<'_> {}
unsafe impl Sync for Dynamic<'_> {}

#[derive(Debug)]
struct Relocation {
    offset: usize,
    addend: Option<usize>,
    sym: SymbolIndex,
    kind: RelocationKind,
}

#[cfg(target_pointer_width = "32")]
impl From<&Rela> for Relocation {
    fn from(reloc: &Rela) -> Self {
        Self {
            offset: reloc.r_offset(NativeEndian) as usize,
            addend: Some(reloc.r_addend(NativeEndian) as usize),
            sym: SymbolIndex(reloc.r_sym(NativeEndian) as usize),
            kind: RelocationKind::new(reloc.r_type(NativeEndian)),
        }
    }
}

#[cfg(target_pointer_width = "64")]
impl From<&Rela> for Relocation {
    fn from(reloc: &Rela) -> Self {
        let is_mips64el = cfg!(all(target_arch = "mips64", target_endian = "little"));
        Self {
            offset: reloc.r_offset(NativeEndian) as usize,
            addend: Some(reloc.r_addend(NativeEndian) as usize),
            sym: SymbolIndex(reloc.r_sym(NativeEndian, is_mips64el) as usize),
            kind: RelocationKind::new(reloc.r_type(NativeEndian, is_mips64el)),
        }
    }
}

impl From<&Rel> for Relocation {
    fn from(reloc: &Rel) -> Self {
        Self {
            offset: reloc.r_offset(NativeEndian) as usize,
            addend: None,
            sym: SymbolIndex(reloc.r_sym(NativeEndian) as usize),
            kind: RelocationKind::new(reloc.r_type(NativeEndian)),
        }
    }
}

// This is matched up to REL_* constants used by musl for ease of comparison
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum RelocationKind {
    COPY,
    DTPMOD,
    DTPOFF,
    GOT,
    IRELATIVE,
    OFFSET,
    PLT,
    RELATIVE,
    SYMBOLIC,
    TLSDESC,
    TPOFF,
    UNKNOWN(u32),
}

impl RelocationKind {
    #[cfg(target_arch = "aarch64")]
    pub fn new(kind: u32) -> Self {
        //WARNING: Only use R_AARCH64_* constants here!
        match kind {
            elf::R_AARCH64_COPY => Self::COPY,
            elf::R_AARCH64_TLS_DTPMOD => Self::DTPMOD,
            elf::R_AARCH64_TLS_DTPREL => Self::DTPOFF,
            elf::R_AARCH64_GLOB_DAT => Self::GOT,
            elf::R_AARCH64_IRELATIVE => Self::IRELATIVE,
            elf::R_AARCH64_JUMP_SLOT => Self::PLT,
            elf::R_AARCH64_RELATIVE => Self::RELATIVE,
            elf::R_AARCH64_ABS64 => Self::SYMBOLIC,
            elf::R_AARCH64_TLSDESC => Self::TLSDESC,
            elf::R_AARCH64_TLS_TPREL => Self::TPOFF,
            _ => Self::UNKNOWN(kind),
        }
    }

    #[cfg(target_arch = "riscv64")]
    pub fn new(kind: u32) -> Self {
        //WARNING: Only use R_RISCV_* constants here!
        match kind {
            elf::R_RISCV_COPY => Self::COPY,
            elf::R_RISCV_TLS_DTPMOD64 => Self::DTPMOD,
            elf::R_RISCV_TLS_DTPREL64 => Self::DTPOFF,
            elf::R_RISCV_IRELATIVE => Self::IRELATIVE,
            elf::R_RISCV_JUMP_SLOT => Self::PLT,
            elf::R_RISCV_RELATIVE => Self::RELATIVE,
            elf::R_RISCV_64 => Self::SYMBOLIC,
            //TODO: not defined, should be 12: elf::R_RISCV_TLSDESC => Self::TLSDESC,
            elf::R_RISCV_TLS_TPREL64 => Self::TPOFF,
            _ => Self::UNKNOWN(kind),
        }
    }

    #[cfg(target_arch = "x86")]
    pub fn new(kind: u32) -> Self {
        //WARNING: Only use R_386_* constants here!
        match kind {
            elf::R_386_COPY => Self::COPY,
            elf::R_386_TLS_DTPMOD32 => Self::DTPMOD,
            elf::R_386_TLS_DTPOFF32 => Self::DTPOFF,
            elf::R_386_GLOB_DAT => Self::GOT,
            elf::R_386_IRELATIVE => Self::IRELATIVE,
            elf::R_386_JMP_SLOT => Self::PLT,
            elf::R_386_PC32 => Self::OFFSET,
            elf::R_386_RELATIVE => Self::RELATIVE,
            elf::R_386_32 => Self::SYMBOLIC,
            elf::R_386_TLS_DESC => Self::TLSDESC,
            elf::R_386_TLS_TPOFF => Self::TPOFF,
            _ => Self::UNKNOWN(kind),
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn new(kind: u32) -> Self {
        //WARNING: Only use R_X86_64_* constants here!
        match kind {
            elf::R_X86_64_COPY => Self::COPY,
            elf::R_X86_64_DTPMOD64 => Self::DTPMOD,
            elf::R_X86_64_DTPOFF64 => Self::DTPOFF,
            elf::R_X86_64_GLOB_DAT => Self::GOT,
            elf::R_X86_64_IRELATIVE => Self::IRELATIVE,
            elf::R_X86_64_JUMP_SLOT => Self::PLT,
            elf::R_X86_64_RELATIVE => Self::RELATIVE,
            elf::R_X86_64_64 => Self::SYMBOLIC,
            elf::R_X86_64_TLSDESC => Self::TLSDESC,
            elf::R_X86_64_TPOFF64 => Self::TPOFF,
            _ => Self::UNKNOWN(kind),
        }
    }
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
    pub tls_module_id: usize,
    pub tls_offset: usize,

    pub(super) dynamic: Dynamic<'static>,

    pub scope: spin::Once<Scope>,
    /// Position Independent Executable.
    pub pie: bool,
}

impl DSO {
    pub fn new<'a>(
        path: &str,
        data: &'a [u8],
        base_addr: Option<usize>,
        dlopened: bool,
        id: usize,
        tls_module_id: usize,
        tls_offset: usize,
    ) -> object::Result<(DSO, Option<Master>, Vec<ProgramHeader>)> {
        let elf = ElfFile::parse(data).unwrap();
        let (mmap, tcb_master, dynamic) =
            DSO::mmap_and_copy(path, &elf, data, base_addr, tls_offset).unwrap();

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

        let dso = DSO {
            name,
            id,
            dlopened,
            entry_point,
            mmap,
            tls_module_id: if tcb_master.is_some() {
                tls_module_id
            } else {
                0
            },
            tls_offset,

            pie: is_pie_enabled(&elf),
            dynamic,
            scope: spin::Once::new(),
        };

        Ok((dso, tcb_master, elf.elf_program_headers().to_vec()))
    }

    #[inline]
    pub fn scope(&self) -> &Scope {
        self.scope.get().expect("scope not initialized")
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

    pub fn get_sym<'a>(&self, name: &'a str) -> Option<(Symbol<'a>, SymbolBinding)> {
        let (_, sym) = self.dynamic.hash_table.find(
            name,
            None,
            &self.dynamic.symbols,
            self.dynamic.dynstrtab,
            &VersionTable::default(),
        )?;

        if sym.st_shndx(NativeEndian) == elf::SHN_UNDEF {
            return None;
        }

        Some((
            Symbol {
                name,
                base: if self.pie {
                    self.mmap.as_ptr() as usize
                } else {
                    0
                },
                value: sym.st_value(NativeEndian) as usize,
                size: sym.st_size(NativeEndian) as usize,
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
        for f in self.dynamic.init_array {
            unsafe { f() }
        }
    }

    pub fn run_fini(&self) {
        for f in self.dynamic.fini_array.iter().rev() {
            unsafe { f() }
        }
    }

    fn mmap_and_copy<'a>(
        path: &str,
        elf: &ElfFile<'a>,
        data: &'a [u8],
        base_addr: Option<usize>,
        tls_offset: usize,
    ) -> object::Result<(&'static [u8], Option<Master>, Dynamic<'static>)> {
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
                    elf::PT_DYNAMIC => {
                        l_ld = ph.p_vaddr(endian);
                    }
                    elf::PT_LOAD => {
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
                .ok_or("Unable to find PT_LOAD section".to_string())
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
                .map_err(|e| format!("failed to map {}. errno: {}", path, e.0))
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

        // Copy data
        let mut dynamic = None;
        for ph in elf.elf_program_headers() {
            let voff = ph.p_vaddr(endian) % ph.p_align(endian);
            let vsize = ((ph.p_memsz(endian) + voff) as usize)
                .next_multiple_of(ph.p_align(endian) as usize);

            match ph.p_type(endian) {
                elf::PT_LOAD => {
                    if skip_load_segment_copy {
                        continue;
                    }
                    let obj_data = {
                        let (offset, size) = ph.file_range(endian);
                        let offset = offset as usize;
                        let range = offset..(offset + size as usize);
                        match data.get(range.clone()) {
                            Some(some) => some,
                            None => return Err(format!("failed to read {:x?}", range)).unwrap(),
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
                                return Err(format!("failed to write {:x?}", range)).unwrap();
                            }
                        }
                    };
                    trace!(
                        "  copy {:#x}, {:#x}: {:#x}, {:#x}",
                        ph.p_vaddr(endian) - voff,
                        vsize,
                        voff,
                        obj_data.len()
                    );
                    mmap_data.copy_from_slice(obj_data);
                }
                elf::PT_TLS => {
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

                elf::PT_DYNAMIC => dynamic = Some((ph, ph.dynamic(endian, data).unwrap().unwrap())),
                _ => (),
            }
        }

        let (parsed_dynamic, debug) =
            Self::parse_dynamic(path, mmap, is_pie_enabled(elf), dynamic.unwrap())?;

        if let Some(i) = debug {
            // FIXME: cleanup
            let (ph, _) = dynamic.unwrap();
            let vaddr = ph.p_vaddr(endian) as usize;
            let bytes: [u8; size_of::<Dyn>() / 2] =
                unsafe { core::mem::transmute((&_r_debug) as *const RTLDDebug as usize) };
            let start = if is_pie_enabled(elf) {
                vaddr + i * size_of::<Dyn>() + size_of::<Dyn>() / 2
            } else {
                vaddr + i * size_of::<Dyn>() + size_of::<Dyn>() / 2
                    - mmap.as_ptr().cast_mut() as usize
            };
            unsafe {
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    mmap.as_ptr().cast_mut().add(start),
                    bytes.len(),
                );
            }
        }

        Ok((mmap, tcb_master, parsed_dynamic))
    }

    fn parse_dynamic<'a>(
        path: &str,
        mmap: &'a [u8],
        is_pie: bool,
        (_, entries): (&ProgramHeader, &[Dyn]),
    ) -> object::Result<(Dynamic<'a>, Option<usize>)> {
        const DT_RELRSZ: u32 = 35;
        const DT_RELR: u32 = 36;
        const DT_RELRENT: u32 = 37;

        let mut runpath = None;
        let mut got = None;
        let mut needed = vec![];
        let mut jmprel = None;
        let mut soname = None;
        let mut hash_table = None;
        let mut explicit_addend = None;
        let mut pltrelsz = None;
        let mut debug = None;
        let mut symtab_ptr = None;
        let (mut rel_ptr, mut rel_len) = (None, None);
        let (mut relr_ptr, mut relr_len) = (None, None);
        let (mut strtab_offset, mut strtab_size) = (None, None);
        let (mut init_array_ptr, mut init_array_len) = (None, None);
        let (mut fini_array_ptr, mut fini_array_len) = (None, None);
        let (mut rela_offset, mut rela_len) = (None, None);

        for (i, entry) in entries.iter().enumerate() {
            let val = entry.d_val(NativeEndian);
            let relative_idx = val as usize - if is_pie { 0 } else { mmap.as_ptr() as usize };
            let ptr = (val as usize + if is_pie { mmap.as_ptr() as usize } else { 0 }) as *const u8;
            let tag = entry.d_tag(NativeEndian) as u32;

            match tag {
                elf::DT_DEBUG => debug = Some(i),

                // {Gnu,SysV}HashTable::parse()
                //
                // > The header does not contain a length field, and so all of
                // > `data` will be used as the hash table values. It does not
                // > matter if this is longer than needed...
                elf::DT_GNU_HASH => {
                    let value = GnuHashTable::parse(NativeEndian, &mmap[relative_idx..])?;
                    hash_table = Some(HashTable::Gnu(value));
                }

                // XXX: Both GNU_HASH and HASH may be present, we give priority
                // to GNU_HASH as it is significantly faster.
                elf::DT_HASH if hash_table.is_none() => {
                    let value = SysVHashTable::parse(NativeEndian, &mmap[relative_idx..])?;
                    hash_table = Some(HashTable::Sysv(value));
                }

                elf::DT_PLTGOT => {
                    got = Some(NonNull::new(ptr as *mut usize).expect("DT_PLTGOT is NULL"));
                }

                elf::DT_NEEDED => needed.push(entry),
                elf::DT_JMPREL => jmprel = Some(ptr as usize),
                elf::DT_RUNPATH => runpath = Some(entry), // FIXME(andypython): rpath
                elf::DT_STRTAB => strtab_offset = Some(relative_idx),
                elf::DT_STRSZ => strtab_size = Some(val),
                elf::DT_SONAME => soname = Some(entry),

                elf::DT_RELA => rela_offset = Some(ptr.cast::<Rela>()),
                elf::DT_RELASZ => rela_len = Some(val as usize / size_of::<Rela>()),
                elf::DT_RELAENT => {
                    assert_eq!(val, size_of::<Rela>() as _)
                }

                elf::DT_REL => rel_ptr = Some(ptr.cast::<Rel>()),
                elf::DT_RELSZ => rel_len = Some(val as usize / size_of::<Rel>()),
                elf::DT_RELENT => {
                    assert_eq!(val, size_of::<Rel>() as _)
                }

                DT_RELR => relr_ptr = Some(ptr.cast::<Relr>()),
                DT_RELRSZ => relr_len = Some(val as usize / size_of::<Relr>()),
                DT_RELRENT => {
                    assert_eq!(val, size_of::<Relr>() as _)
                }

                elf::DT_PLTREL => {
                    let val = val as u32;
                    if val == elf::DT_RELA {
                        explicit_addend = Some(true);
                    } else {
                        assert_eq!(val, elf::DT_REL);
                        explicit_addend = Some(false);
                    }
                }
                elf::DT_PLTRELSZ => pltrelsz = Some(val as usize),

                elf::DT_INIT_ARRAY if val != 0 => init_array_ptr = Some(ptr.cast::<InitFn>()),
                elf::DT_INIT_ARRAYSZ => init_array_len = Some(val as usize / size_of::<InitFn>()),

                elf::DT_FINI_ARRAY if val != 0 => fini_array_ptr = Some(ptr.cast::<InitFn>()),
                elf::DT_FINI_ARRAYSZ => fini_array_len = Some(val as usize / size_of::<InitFn>()),

                elf::DT_SYMTAB => symtab_ptr = Some(ptr as *const Sym),
                elf::DT_SYMENT => {
                    assert_eq!(val as usize, size_of::<Sym>());
                }

                _ => {}
            }
        }

        let strtab_offset = strtab_offset.expect("mandatory DT_STRTAB not present");
        let strtab_size = strtab_size.expect("mandatory DT_STRSZ not present");

        let dynstrtab = StringTable::new(
            &*mmap,
            strtab_offset as u64,
            strtab_offset as u64 + strtab_size as u64,
        );

        let get_str = |entry: &Dyn| {
            entry
                .string(NativeEndian, dynstrtab)
                .map(|bytes| core::str::from_utf8(bytes).expect("non utf-8 elf symbol name"))
        };

        unsafe fn get_array<'a, T>(ptr: Option<*const T>, len: Option<usize>) -> &'a [T] {
            if let Some(ptr) = ptr {
                let len = len.expect("dynamic entry was present without it's corresponding size");
                core::slice::from_raw_parts(ptr, len)
            } else {
                assert!(len.is_none());
                &[]
            }
        }

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

        let init_array = unsafe { get_array(init_array_ptr, init_array_len) };
        let fini_array = unsafe { get_array(fini_array_ptr, fini_array_len) };
        let rela = unsafe { get_array(rela_offset, rela_len) };
        let relr = unsafe { get_array(relr_ptr, relr_len) };
        let rel = unsafe { get_array(rel_ptr, rel_len) };

        Ok((
            Dynamic {
                symbols: unsafe { get_array(symtab_ptr, Some(hash_table.symbol_table_length())) },
                runpath,
                got,
                needed,
                jmprel,
                soname,
                hash_table,
                dynstrtab,
                init_array,
                fini_array,
                rela,
                rel,
                relr,
                explicit_addend: explicit_addend.unwrap_or_default(),
                pltrelsz: pltrelsz.unwrap_or_default(),
            },
            debug,
        ))
    }

    fn static_relocate(&self, global_scope: &Scope, reloc: Relocation) -> object::Result<()> {
        let b = self.mmap.as_ptr() as usize;

        let (sym, my_sym) = if reloc.sym.0 > 0 {
            let name = self.dynamic.symbol_name(reloc.sym).unwrap();

            let lookup_scopes = [global_scope, self.scope()];
            let sym = if matches!(reloc.kind, RelocationKind::COPY) {
                lookup_scopes
                    .iter()
                    .find_map(|scope| scope._get_sym(name, 1))
            } else {
                resolve_sym(name, &lookup_scopes)
            }
            .map(|(sym, _, obj)| (sym, obj));

            (sym, self.dynamic.symbol(reloc.sym))
        } else {
            (None, None)
        };

        let (s, t, tls_id) = sym
            .as_ref()
            .map(|(sym, obj)| (sym.as_ptr() as usize, obj.tls_offset, obj.tls_module_id))
            //TODO: is self.tls_module_id the right fallback?
            .unwrap_or((0, 0, self.tls_module_id));

        let ptr = if self.pie {
            (b + reloc.offset) as *mut u8
        } else {
            reloc.offset as *mut u8
        };
        let p = ptr as usize;
        let a = match reloc.addend {
            Some(some) => some,
            None => match reloc.kind {
                RelocationKind::COPY | RelocationKind::GOT | RelocationKind::PLT => 0,
                _ => unsafe { *(ptr as *mut usize) },
            },
        };

        //TODO: support different sizes?
        let set_usize = |value| unsafe {
            *(ptr as *mut usize) = value;
        };

        match reloc.kind {
            RelocationKind::DTPMOD => set_usize(tls_id),
            //TODO: Subtract DTP_OFFSET, which is 0x800 on riscv64, 0 on x86?
            RelocationKind::DTPOFF => {
                if reloc.sym.0 > 0 {
                    let (sym, _) = sym
                        .as_ref()
                        .expect("RelocationKind::DTPOFF called without valid symbol");
                    set_usize(sym.value + a);
                } else {
                    set_usize(a);
                }
            }
            RelocationKind::GOT => set_usize(s),
            RelocationKind::OFFSET => set_usize((s + a).wrapping_sub(p)),
            RelocationKind::RELATIVE => set_usize(b + a),
            RelocationKind::SYMBOLIC => set_usize(s + a),
            RelocationKind::TPOFF => {
                if reloc.sym.0 > 0 {
                    let (sym, _) = sym
                        .as_ref()
                        .expect("RelocationKind::TPOFF called without valid symbol");
                    set_usize((sym.value + a).wrapping_sub(t));
                } else {
                    set_usize(a.wrapping_sub(t));
                }
            }
            RelocationKind::IRELATIVE => unsafe {
                let f: unsafe extern "C" fn() -> usize = core::mem::transmute(b + a);
                set_usize(f());
            },
            RelocationKind::COPY => unsafe {
                let (sym, obj) = sym
                    .as_ref()
                    .expect("RelocationKind::COPY called without valid symbol");
                let my_sym = my_sym.expect("RelocationKind::COPY called without valid symbol");
                assert!(
                    sym.size == my_sym.st_size(NativeEndian) as usize,
                    "RelocationKind::COPY failed: I was trying to use the symbol {} from {} for {} but they had different sizes. Please consider relinking.",
                    sym.name,
                    obj.name,
                    self.name
                );
                // SAFETY: Both the source and destination have the same size.
                ptr::copy_nonoverlapping(sym.as_ptr() as *const u8, ptr, sym.size);
            },
            _ => unimplemented!("relocation type {:?}", reloc.kind),
        }

        Ok(())
    }

    fn lazy_relocate(&self, global_scope: &Scope, resolve: Resolve) -> object::Result<()> {
        let Some(got) = self.got() else {
            assert_eq!(self.dynamic.jmprel, 0);
            return Ok(());
        };

        let object_base_addr = self.mmap.as_ptr() as usize;
        let jmprel = self.dynamic.jmprel;
        let pltrelsz = self.dynamic.pltrelsz;

        unsafe {
            got.add(1).write(core::ptr::addr_of!(*self) as usize);
            got.add(2).write(__plt_resolve_trampoline as usize);
        }

        let relsz = if self.dynamic.explicit_addend {
            size_of::<Rela>()
        } else {
            size_of::<Rel>()
        };

        for addr in (jmprel..(jmprel + pltrelsz)).step_by(relsz) {
            let reloc: Relocation = if self.dynamic.explicit_addend {
                unsafe { &*(addr as *const Rela) }.into()
            } else {
                unsafe { &*(addr as *const Rel) }.into()
            };

            let ptr = if self.pie {
                (object_base_addr + reloc.offset) as *mut usize
            } else {
                reloc.offset as *mut usize
            };

            match (reloc.kind, resolve) {
                (RelocationKind::PLT, Resolve::Lazy) if self.pie => unsafe {
                    *ptr += object_base_addr;
                },

                (RelocationKind::PLT, Resolve::Lazy) => {
                    // NOP.
                }

                (RelocationKind::PLT, Resolve::Now) => {
                    let name = self.dynamic.symbol_name(reloc.sym).unwrap();

                    let resolved = resolve_sym(name, &[global_scope, self.scope()])
                        .map(|(sym, _, _)| sym.as_ptr() as usize)
                        .unwrap_or_else(|| {
                            panic!(
                                "unresolved symbol: {name} for soname {:?}",
                                self.dynamic.soname
                            )
                        });

                    unsafe {
                        *ptr = resolved + reloc.addend.unwrap_or(0);
                    }
                }

                _ => {
                    unimplemented!(
                        "relocation type {:?} with resolve {:?}",
                        reloc.kind,
                        resolve
                    )
                }
            }
        }

        Ok(())
    }

    pub fn relocate(&self, ph: &[ProgramHeader], resolve: Resolve) -> object::Result<()> {
        let global_scope = GLOBAL_SCOPE.read();
        let base = self.mmap.as_ptr();

        // Apply DT_RELR relative relocations.
        let mut addr = ptr::null_mut();
        for &entry in self.dynamic.relr {
            if entry & 1 == 0 {
                // An even entry sets up `addr` for subsequent odd entries.
                unsafe {
                    addr = base.add(entry) as *mut usize;
                    *addr += base as usize;
                    addr = addr.add(1);
                }
            } else {
                // An odd entry indicates a bitmap describing at maximum 63
                // (for 64-bit) or 31 (for 32-bit) locations following `addr`.
                // Odd entries can be chained.
                let mut entry = entry >> 1;
                let mut i = 0;
                while entry != 0 {
                    if entry & 1 != 0 {
                        unsafe {
                            *addr.add(i) += base as usize;
                        }
                    }
                    entry >>= 1;
                    i += 1;
                }

                addr = unsafe { addr.add(CHAR_BITS * size_of::<Relr>() - 1) };
            }
        }

        self.dynamic
            .static_relocations()
            .try_for_each(|reloc| self.static_relocate(&global_scope, reloc))?;

        self.lazy_relocate(&global_scope, resolve)?;

        // Protect pages
        for ph in ph
            .iter()
            .filter(|ph| ph.p_type(NativeEndian) == elf::PT_LOAD)
        {
            let voff = ph.p_vaddr(NativeEndian) % ph.p_align(NativeEndian);
            let vaddr = (ph.p_vaddr(NativeEndian) - voff) as usize;
            let vsize = ((ph.p_memsz(NativeEndian) + voff) as usize)
                .next_multiple_of(ph.p_align(NativeEndian) as usize);

            let mut prot = 0;
            if ph.p_flags(NativeEndian) & elf::PF_R == elf::PF_R {
                prot |= sys_mman::PROT_READ;
            }

            // W ^ X. If it is executable, do not allow it to be writable, even if requested
            if ph.p_flags(NativeEndian) & elf::PF_X == elf::PF_X {
                prot |= sys_mman::PROT_EXEC;
            } else if ph.p_flags(NativeEndian) & elf::PF_W == elf::PF_W {
                prot |= sys_mman::PROT_WRITE;
            }

            unsafe {
                let ptr = if self.pie {
                    self.mmap.as_ptr().add(vaddr)
                } else {
                    vaddr as *const u8
                };
                trace!("  prot {:#x}, {:#x}: {:p}, {:#x}", vaddr, vsize, ptr, prot);
                Sys::mprotect(ptr as *mut c_void, vsize, prot).expect("[ld.so]: mprotect failed");
            }
        }

        Ok(())
    }
}

impl Drop for DSO {
    fn drop(&mut self) {
        self.run_fini();
        unsafe { Sys::munmap(self.mmap.as_ptr() as *mut c_void, self.mmap.len()).unwrap() };
    }
}

fn is_pie_enabled(elf: &ElfFile) -> bool {
    elf.elf_header().e_type.get(elf.endian()) == elf::ET_DYN
}

fn basename(path: &str) -> String {
    path.split("/").last().unwrap_or(path).to_string()
}

fn dirname(path: &str) -> String {
    let mut parts: Vec<&str> = path.split("/").collect();
    parts.truncate(parts.len() - 1);
    parts.join("/")
}

pub fn resolve_sym<'a>(
    name: &'a str,
    scopes: &[&'a Scope],
) -> Option<(Symbol<'a>, SymbolBinding, Arc<DSO>)> {
    scopes.iter().find_map(|scope| scope.get_sym(name))
}
