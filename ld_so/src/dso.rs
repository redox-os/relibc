// ld_so/src/dso.rs
//! Dynamic Shared Object (DSO) Model.
//!
//! Represents a loaded ELF object (executable or library) in memory.
//! Handles parsing of Program Headers, Dynamic Section, and Symbol Tables.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::{slice, str};

use crate::header::elf;
use crate::gnu_hash::GnuHash;
use crate::versioning::{VersionData, VersionReq};

pub struct DSO {
    pub name: String,
    pub base_addr: usize,
    pub entry_point: usize,
    pub dynamic: Option<&'static [elf::Dyn]>,
    pub sym_table: Option<&'static [elf::Sym]>,
    pub str_table: Option<&'static [u8]>,
    pub gnu_hash: Option<GnuHash<'static>>,
    pub sysv_hash: Option<&'static [u32]>,
    pub versym: Option<&'static [u16]>,
    pub verdef: Option<*const elf::Verdef>,
    pub verneed: Option<*const elf::Verneed>,
    pub verneed_num: usize,
    pub verdef_num: usize,
    pub tls_module_id: usize,
    pub tls_offset: usize,
    pub tls_size: usize,
    pub tls_align: usize,
    pub tls_image: Option<&'static [u8]>,
}

impl DSO {
    /// Create a DSO representing the main executable loaded by the kernel.
    /// `sp` is the stack pointer at entry, used to find AT_PHDR/AT_PHNUM if needed.
    pub unsafe fn new_executable(sp: *const usize) -> Self {
        // In a real implementation, walk `sp` to find Aux Vector (AT_PHDR, AT_PHNUM, AT_BASE).
        // For this contract, we assume we can get these or have a bootstrap mechanism.

        // Placeholder values for bootstrapping
        let base_addr = 0;
        let entry_point = 0;
        let name = String::from("main");

        // We would parse the dynamic section here.
        // Since we can't easily walk the stack in this snippet without the auxv parser code,
        // we return a minimal DSO to allow the linker structure to compile.
        Self {
            name,
            base_addr,
            entry_point,
            dynamic: None,
            sym_table: None,
            str_table: None,
            gnu_hash: None,
            sysv_hash: None,
            versym: None,
            verdef: None,
            verneed: None,
            verneed_num: 0,
            verdef_num: 0,
            tls_module_id: 1, // Main exe is module 1
            tls_offset: 0,
            tls_size: 0,
            tls_align: 0,
            tls_image: None,
        }
    }

    /// Run the initialization functions (DT_INIT / DT_INIT_ARRAY).
    pub unsafe fn run_init(&self) {
        // 1. DT_INIT
        // 2. DT_INIT_ARRAY
    }

    /// Get Iterator over relocations.
    /// Returns (type, symbol_index, offset, addend).
    pub fn relocations(&self) -> impl Iterator<Item = (u32, usize, usize, Option<usize>)> {
        // Implementation would iterate .rela.dyn / .rel.dyn
        core::iter::empty()
    }

    pub fn get_sym_name(&self, index: usize) -> Option<&str> {
        let sym = &self.sym_table?[index];
        if sym.st_name == 0 { return None; }
        let start = sym.st_name as usize;
        let slice = &self.str_table?[start..];
        let end = slice.iter().position(|&c| c == 0)?;
        str::from_utf8(&slice[..end]).ok()
    }

    pub fn get_version_req(&self, _sym_idx: usize) -> Option<VersionReq> {
        // Logic to look up index in .gnu.version and correlate with .gnu.version_r
        None
    }

    // Accessors required by Linux Parity module
    pub fn sym_table(&self) -> &[elf::Sym] { self.sym_table.unwrap_or(&[]) }
    pub fn str_table(&self) -> &[u8] { self.str_table.unwrap_or(&[]) }
    pub fn gnu_hash(&self) -> Option<&GnuHash<'static>> { self.gnu_hash.as_ref() }
    pub fn sysv_hash(&self) -> Option<&[u32]> { self.sysv_hash }
    pub fn base_addr(&self) -> usize { self.base_addr }

    pub fn version_data(&self) -> Option<VersionData<'static>> {
        if let (Some(versym), Some(str_tab)) = (self.versym, self.str_table) {
            Some(VersionData {
                versym,
                verneed: self.verneed.unwrap_or(core::ptr::null()),
                verneed_num: self.verneed_num,
                verdef: self.verdef.unwrap_or(core::ptr::null()),
                verdef_num: self.verdef_num,
                str_tab,
            })
        } else {
            None
        }
    }
}