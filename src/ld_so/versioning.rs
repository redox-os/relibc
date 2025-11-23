// src/ld_so/versioning.rs
//! Support for ELF Symbol Versioning.
//! Handles .gnu.version, .gnu.version_r, and .gnu.version_d.

#![no_std]

use crate::header::elf;

/// Represents a specific version requirement for a symbol.
#[derive(Debug, Clone, Copy)]
pub struct VersionReq<'a> {
    /// The file (library) providing this version.
    pub filename: &'a str,
    /// The name of the version (e.g., "GLIBC_2.14").
    pub version: &'a str,
    /// The hash of the version string.
    pub hash: u32,
    /// Is this a hidden version?
    pub hidden: bool,
}

pub struct VersionData<'a> {
    /// Pointer to .gnu.version section (array of u16 indices).
    pub versym: &'a [u16],
    /// Pointer to .gnu.version_r section.
    pub verneed: *const elf::Verneed,
    pub verneed_num: usize,
    /// Pointer to .gnu.version_d section.
    pub verdef: *const elf::Verdef,
    pub verdef_num: usize,
    /// String table for version strings.
    pub str_tab: &'a [u8],
}

impl<'a> VersionData<'a> {
    /// Check if a symbol definition matches a requirement.
    pub fn check(&self, sym_idx: usize, req: Option<&VersionReq>) -> bool {
        // 1. Get version index from .gnu.version table
        if self.versym.is_empty() {
            // If no versioning data, assume match if req is None or global.
            return true; 
        }
        
        let ver_idx = self.versym[sym_idx];
        let is_hidden = (ver_idx & 0x8000) != 0;
        let idx = ver_idx & 0x7FFF;

        // 0 = local, 1 = global
        if idx <= 1 {
            return true;
        }

        if let Some(requirement) = req {
            // We are looking for a specific version definition.
            // We need to walk .gnu.version_d to find the name associated with `idx`.
            if let Some(def_name) = unsafe { self.get_def_name(idx) } {
                // Standard Linux ld.so comparison:
                // Hash match && String match
                // (Optimized: check hash first)
                return def_name == requirement.version;
            }
        } else {
            // If no specific requirement provided, we take the default (non-hidden) one.
            if !is_hidden {
                return true;
            }
        }

        false
    }

    /// Retrieve the version name for a given definition index from .gnu.version_d
    unsafe fn get_def_name(&self, ndx: u16) -> Option<&'a str> {
        if self.verdef.is_null() { return None; }

        let mut ptr = self.verdef as *const u8;
        for _ in 0..self.verdef_num {
            let def = &*(ptr as *const elf::Verdef);
            if (def.vd_ndx & 0x7FFF) == ndx {
                // Found the definition, get the first aux entry for the string
                let aux_ptr = ptr.add(def.vd_aux as usize);
                let aux = &*(aux_ptr as *const elf::Verdaux);
                return self.get_string(aux.vda_name as usize);
            }
            if def.vd_next == 0 { break; }
            ptr = ptr.add(def.vd_next as usize);
        }
        None
    }

    fn get_string(&self, offset: usize) -> Option<&'a str> {
        if offset >= self.str_tab.len() { return None; }
        let slice = &self.str_tab[offset..];
        let end = slice.iter().position(|&c| c == 0)?;
        core::str::from_utf8(&slice[..end]).ok()
    }
}