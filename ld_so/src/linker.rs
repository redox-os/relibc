// ld_so/src/linker.rs
#![no_std]

use core::ptr;
use core::slice;
use relibc::abi_types::*;

/// Standard ELF Hash Function (PJW Hash)
/// Used to map symbol names to hash buckets for O(1) lookup.
fn elf_hash(name: &[u8]) -> u32 {
    let mut h: u32 = 0;
    let mut g: u32;
    for &b in name {
        h = (h << 4) + (b as u32);
        g = h & 0xf0000000;
        if g != 0 {
            h ^= g >> 24;
        }
        h &= !g;
    }
    h
}

/// Extracts the relocation type from r_info.
#[inline(always)]
fn elf64_r_type(info: Elf64_Xword) -> u32 {
    (info & 0xffffffff) as u32
}

/// Extracts the symbol table index from r_info.
#[inline(always)]
fn elf64_r_sym(info: Elf64_Xword) -> u32 {
    (info >> 32) as u32
}

/// The Linker Context
/// Holds pointers to the dynamic structures of a loaded object.
pub struct Linker {
    base_addr: usize,
    symtab: *const Elf64_Sym,
    strtab: *const u8,
    hashtab: *const u32,
}

impl Linker {
    /// Initialize the linker by scanning the Dynamic Section.
    /// This mimics `ld.so` initialization logic.
    pub unsafe fn new(base_addr: usize, dyn_ptr: *const Elf64_Dyn) -> Option<Self> {
        let mut symtab = ptr::null();
        let mut strtab = ptr::null();
        let mut hashtab = ptr::null();

        let mut curr = dyn_ptr;
        while (*curr).d_tag != DT_NULL {
            match (*curr).d_tag {
                DT_SYMTAB => symtab = (base_addr + (*curr).d_un as usize) as *const Elf64_Sym,
                DT_STRTAB => strtab = (base_addr + (*curr).d_un as usize) as *const u8,
                DT_HASH => hashtab = (base_addr + (*curr).d_un as usize) as *const u32,
                _ => {}
            }
            curr = curr.add(1);
        }

        if symtab.is_null() || strtab.is_null() {
            return None;
        }

        Some(Linker { base_addr, symtab, strtab, hashtab })
    }

    /// Resolve a symbol by name.
    /// If a hash table is present, uses it. Otherwise, simplistic linear scan is omitted for brevity.
    pub unsafe fn lookup_symbol(&self, name: &[u8]) -> Option<Elf64_Addr> {
        if self.hashtab.is_null() {
            return None;
        }

        // DT_HASH Layout:
        // nbucket (u32)
        // nchain (u32)
        // buckets[nbucket]
        // chains[nchain]
        let nbucket = *self.hashtab;
        let buckets = self.hashtab.add(2);
        let chains = buckets.add(nbucket as usize);

        let hash = elf_hash(name);
        let mut idx = *buckets.add((hash % nbucket) as usize);

        while idx != 0 {
            let sym = &*self.symtab.add(idx as usize);

            // Check name match
            let sym_name_ptr = self.strtab.add(sym.st_name as usize);
            if self.streq(sym_name_ptr, name) {
                // Found symbol! Return Absolute Address (Base + Value)
                if sym.st_value != 0 {
                    return Some(self.base_addr as u64 + sym.st_value);
                }
            }

            idx = *chains.add(idx as usize);
        }
        None
    }

    unsafe fn streq(&self, s1: *const u8, s2: &[u8]) -> bool {
        let mut i = 0;
        loop {
            let c1 = *s1.add(i);
            if i >= s2.len() { return c1 == 0; }
            let c2 = s2[i];
            if c1 != c2 { return false; }
            if c1 == 0 { return true; }
            i += 1;
        }
    }

    /// Perform Relocations (Phase 2)
    /// Handles Relative, Global Data (GOT), and Jump Slots (PLT).
    pub unsafe fn relocate(&self, rela_ptr: *const Elf64_Rela, rela_size: usize) {
        let count = rela_size / core::mem::size_of::<Elf64_Rela>();
        let rela_slice = slice::from_raw_parts(rela_ptr, count);

        for rela in rela_slice {
            let r_type = elf64_r_type(rela.r_info);
            let r_sym = elf64_r_sym(rela.r_info);
            let target = (self.base_addr as u64 + rela.r_offset) as *mut u64;

            match r_type {
                // Phase 1: Relative (Base + Addend)
                R_X86_64_RELATIVE | R_AARCH64_RELATIVE | R_RISCV_RELATIVE => {
                    let val = (self.base_addr as u64).wrapping_add(rela.r_addend as u64);
                    ptr::write_volatile(target, val);
                },
                // Phase 2: Global Data / Jump Slots (Symbol Resolution)
                R_X86_64_GLOB_DAT | R_X86_64_JUMP_SLOT |
                R_AARCH64_GLOB_DAT | R_AARCH64_JUMP_SLOT |
                R_RISCV_JUMP_SLOT => {
                    if r_sym != 0 {
                        let sym = &*self.symtab.add(r_sym as usize);

                        // Self-lookup optimization: if symbol is defined in this DSO
                        if sym.st_shndx != 0 { // SHN_UNDEF = 0
                            let val = (self.base_addr as u64).wrapping_add(sym.st_value);
                            ptr::write_volatile(target, val);
                        } else {
                            // TODO: External Lookup via DT_NEEDED dependencies
                            // let name_ptr = self.strtab.add(sym.st_name as usize);
                            // let val = global_scope.lookup(name_ptr);
                        }
                    }
                },
                _ => {} // Ignore others
            }
        }
    }
}

// Shim for the old API to maintain compatibility with Verification App
// This function allows the Phase 1 test harness to continue working without
// needing to construct a full `Linker` object with a valid `Elf64_Dyn` pointer.
pub unsafe fn relocate_shared_object(base_addr: Elf64_Addr, rela_ptr: *const Elf64_Rela, rela_size: usize) {
    // In the boot-strap/test phase, we don't have the dynamic section pointer passed in.
    // We strictly perform relative relocations here manually.
    let count = rela_size / core::mem::size_of::<Elf64_Rela>();
    let slice = slice::from_raw_parts(rela_ptr, count);

    for rela in slice {
        let r_type = elf64_r_type(rela.r_info);
        // Check Architecture & Type
        let is_rel = match r_type {
            R_X86_64_RELATIVE if cfg!(target_arch = "x86_64") => true,
            R_AARCH64_RELATIVE if cfg!(target_arch = "aarch64") => true,
            R_RISCV_RELATIVE if cfg!(target_arch = "riscv64") => true,
            _ => false
        };

        if is_rel {
            let ptr = (base_addr + rela.r_offset) as *mut u64;
            let val = base_addr.wrapping_add(rela.r_addend as u64);
            ptr::write_volatile(ptr, val);
        }
    }
}