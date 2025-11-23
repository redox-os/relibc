// ld_so/src/linker.rs
#![no_std]
extern crate alloc;

use core::ptr;
use core::slice;
use alloc::vec::Vec;
use alloc::string::String;
use relibc::abi_types::*;

fn elf_hash(name: &[u8]) -> u32 {
    let mut h: u32 = 0;
    let mut g: u32;
    for &b in name {
        h = (h << 4) + (b as u32);
        g = h & 0xf0000000;
        if g != 0 { h ^= g >> 24; }
        h &= !g;
    }
    h
}

#[inline(always)] fn elf64_r_type(info: Elf64_Xword) -> u32 { (info & 0xffffffff) as u32 }
#[inline(always)] fn elf64_r_sym(info: Elf64_Xword) -> u32 { (info >> 32) as u32 }

type InitFunc = extern "C" fn();

/// Represents a loaded shared object (DSO)
pub struct Dso {
    pub base_addr: usize,
    pub name: String,
    pub symtab: *const Elf64_Sym,
    pub strtab: *const u8,
    pub hashtab: *const u32,
    pub tls_module_id: usize,
    pub tls_offset: usize,
    pub dependencies: Vec<String>,
    // Constructors
    pub init: Option<InitFunc>,
    pub init_array: *const InitFunc,
    pub init_array_sz: usize,
}

pub struct Linker {
    objects: Vec<Dso>,
    next_tls_module_id: usize,
}

impl Linker {
    pub fn new() -> Self {
        Linker {
            objects: Vec::new(),
            next_tls_module_id: 1,
        }
    }

    /// Phase 3: Loading (including Constructor parsing)
    pub unsafe fn load_object(&mut self, name: &str, base_addr: usize, dyn_ptr: *const Elf64_Dyn) -> Result<(), &'static str> {
        let mut symtab = ptr::null();
        let mut strtab = ptr::null();
        let mut hashtab = ptr::null();
        let mut dependencies = Vec::new();
        let mut init = None;
        let mut init_array = ptr::null();
        let mut init_array_sz = 0;

        // 1. First Pass: Parsing
        let mut curr = dyn_ptr;
        while (*curr).d_tag != DT_NULL {
            match (*curr).d_tag {
                DT_SYMTAB => symtab = (base_addr + (*curr).d_un as usize) as *const Elf64_Sym,
                DT_STRTAB => strtab = (base_addr + (*curr).d_un as usize) as *const u8,
                DT_HASH => hashtab = (base_addr + (*curr).d_un as usize) as *const u32,
                DT_INIT => {
                    let ptr = (base_addr + (*curr).d_un as usize) as *const ();
                    init = Some(core::mem::transmute::<*const (), InitFunc>(ptr));
                },
                DT_INIT_ARRAY => init_array = (base_addr + (*curr).d_un as usize) as *const InitFunc,
                DT_INIT_ARRAYSZ => init_array_sz = (*curr).d_un as usize,
                _ => {}
            }
            curr = curr.add(1);
        }

        if symtab.is_null() || strtab.is_null() { return Err("Missing SYMTAB/STRTAB"); }

        // 2. Second Pass: Dependencies
        curr = dyn_ptr;
        while (*curr).d_tag != DT_NULL {
            if (*curr).d_tag == DT_NEEDED {
                let name_offset = (*curr).d_un as usize;
                let dep_name = self.get_string(strtab, name_offset);
                dependencies.push(String::from(dep_name));
            }
            curr = curr.add(1);
        }

        let dso = Dso {
            base_addr,
            name: String::from(name),
            symtab,
            strtab,
            hashtab,
            tls_module_id: self.next_tls_module_id,
            tls_offset: 0,
            dependencies,
            init,
            init_array,
            init_array_sz,
        };

        self.next_tls_module_id += 1;
        self.objects.push(dso);
        Ok(())
    }

    unsafe fn get_string(&self, strtab: *const u8, offset: usize) -> &str {
        let start = strtab.add(offset);
        let mut len = 0;
        while *start.add(len) != 0 { len += 1; }
        let slice = slice::from_raw_parts(start, len);
        core::str::from_utf8_unchecked(slice)
    }

    pub unsafe fn lookup_global(&self, name: &[u8]) -> Option<(usize, usize)> {
        for dso in &self.objects {
            if let Some(addr) = self.lookup_in_dso(dso, name) {
                return Some((addr as usize, dso.tls_module_id));
            }
        }
        None
    }

    unsafe fn lookup_in_dso(&self, dso: &Dso, name: &[u8]) -> Option<Elf64_Addr> {
        if dso.hashtab.is_null() { return None; }
        let nbucket = *dso.hashtab;
        let buckets = dso.hashtab.add(2);
        let chains = buckets.add(nbucket as usize);
        let hash = elf_hash(name);
        let mut idx = *buckets.add((hash % nbucket) as usize);

        while idx != 0 {
            let sym = &*dso.symtab.add(idx as usize);
            let sym_name_ptr = dso.strtab.add(sym.st_name as usize);
            if self.streq(sym_name_ptr, name) {
                if sym.st_value != 0 {
                    return Some(dso.base_addr as u64 + sym.st_value);
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

    pub unsafe fn relocate(&self, dso_idx: usize, rela_ptr: *const Elf64_Rela, rela_size: usize) {
        let dso = &self.objects[dso_idx];
        let count = rela_size / core::mem::size_of::<Elf64_Rela>();
        let rela_slice = slice::from_raw_parts(rela_ptr, count);

        for rela in rela_slice {
            let r_type = elf64_r_type(rela.r_info);
            let r_sym = elf64_r_sym(rela.r_info);
            let target = (dso.base_addr as u64 + rela.r_offset) as *mut u64;

            match r_type {
                R_X86_64_RELATIVE => {
                    let val = (dso.base_addr as u64).wrapping_add(rela.r_addend as u64);
                    ptr::write_volatile(target, val);
                },
                R_X86_64_GLOB_DAT | R_X86_64_JUMP_SLOT => {
                    if r_sym != 0 {
                        let sym = &*dso.symtab.add(r_sym as usize);
                        let name_ptr = dso.strtab.add(sym.st_name as usize);
                        let mut name_len = 0;
                        while *name_ptr.add(name_len) != 0 { name_len += 1; }
                        let name = slice::from_raw_parts(name_ptr, name_len);

                        if let Some((addr, _)) = self.lookup_global(name) {
                            ptr::write_volatile(target, addr as u64);
                        }
                    }
                },
                R_X86_64_DTPMOD64 => {
                    if r_sym != 0 {
                        ptr::write_volatile(target, dso.tls_module_id as u64);
                    }
                },
                _ => {}
            }
        }
    }

    pub unsafe fn init_tls(&self) {
        #[cfg(target_arch = "x86_64")]
        {
            let layout = core::alloc::Layout::from_size_align(4096, 16).unwrap();
            let tcb_ptr = alloc::alloc::alloc(layout) as *mut u64;
            if !tcb_ptr.is_null() {
                *tcb_ptr = tcb_ptr as u64;
                let _ = self.syscall_arch_prctl(ARCH_SET_FS, tcb_ptr as u64);
            }
        }
    }

    unsafe fn syscall_arch_prctl(&self, code: u64, addr: u64) -> u64 {
        #[cfg(target_arch = "x86_64")]
        {
            let ret: u64;
            core::arch::asm!(
            "syscall",
            in("rax") 158,
            in("rdi") code,
            in("rsi") addr,
            lateout("rax") ret,
            out("rcx") _,
            out("r11") _,
            options(nostack, preserves_flags)
            );
            ret
        }
        #[cfg(not(target_arch = "x86_64"))]
        0
    }

    /// Phase 3: Execute Constructors
    /// Iterates over loaded objects and runs .init and .init_array functions.
    pub unsafe fn run_constructors(&self) {
        // In a real linker, this would be reverse topological order of dependencies.
        // Here we just iterate linearly (reverse of load order usually).
        for dso in self.objects.iter().rev() {
            // 1. DT_INIT
            if let Some(init_func) = dso.init {
                init_func();
            }

            // 2. DT_INIT_ARRAY
            if !dso.init_array.is_null() && dso.init_array_sz > 0 {
                let count = dso.init_array_sz / core::mem::size_of::<InitFunc>();
                let slice = slice::from_raw_parts(dso.init_array, count);
                for func in slice {
                    func();
                }
            }
        }
    }
}

// Shim
pub unsafe fn relocate_shared_object(base_addr: Elf64_Addr, rela_ptr: *const Elf64_Rela, rela_size: usize) {
    let count = rela_size / core::mem::size_of::<Elf64_Rela>();
    let slice = slice::from_raw_parts(rela_ptr, count);
    for rela in slice {
        if elf64_r_type(rela.r_info) == R_X86_64_RELATIVE {
            let ptr = (base_addr + rela.r_offset) as *mut u64;
            let val = base_addr.wrapping_add(rela.r_addend as u64);
            ptr::write_volatile(ptr, val);
        }
    }
}