// ld_so/src/linker.rs
//! The Dynamic Linker Orchestrator.
//! Manages the loading graph, symbol resolution, relocation, and TLS initialization.

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use core::{mem, ptr, slice};

use crate::header::elf;
use crate::dso::DSO;
use crate::reloc;
use crate::tcb::Tcb;
use crate::tls;
use crate::linux_parity::{find_symbol_linux_style, LookupResult};
use crate::versioning::{VersionReq, VersionData};

/// Extra bytes allocated in the Static TLS block for runtime-loaded libraries.
/// This allows `dlopen`'d libraries to use the Initial Exec (IE) model.
/// glibc uses ~1664 bytes; we choose 2048 for safety/alignment.
const STATIC_TLS_SURPLUS: usize = 2048;

extern "C" {
    fn open(path: *const i8, flags: i32, mode: i32) -> i32;
    fn close(fd: i32) -> i32;
    fn fstat(fd: i32, buf: *mut u8) -> i32;
    fn mmap(addr: *mut u8, len: usize, prot: i32, flags: i32, fd: i32, offset: i64) -> *mut u8;
}

pub struct Linker {
    objects: Vec<DSO>,
    /// Total size of the allocated Static TLS block.
    static_tls_size: usize,
    /// The point where "Surplus" begins (end of boot-time modules).
    static_tls_end_offset: usize,
    /// Maximum alignment requirement seen in the static TLS set.
    static_tls_align: usize,
    /// Current offset allocator for static TLS.
    tls_offset: usize,
    /// Bytes remaining in the surplus.
    surplus_remaining: usize,
}

impl Linker {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            static_tls_size: 0,
            static_tls_end_offset: 0,
            static_tls_align: 16,
            tls_offset: 0,
            surplus_remaining: 0,
        }
    }

    pub fn link(&mut self, mut main_dso: DSO) {
        // 1. Register Main TLS (Module ID 1)
        if main_dso.tls_size > 0 {
            main_dso.tls_module_id = tls::register_tls_module(
                main_dso.tls_size,
                main_dso.tls_align,
                main_dso.tls_image.map(|s| s.as_ptr() as usize).unwrap_or(0),
                main_dso.tls_image.map(|s| s.len()).unwrap_or(0),
                Some(0),
            );
        }

        self.objects.push(main_dso);
        self.load_dependencies();
        
        // 2. Calculate Layout for Static TLS + Surplus
        self.layout_static_tls();

        // 3. Setup TCB and Copy Static Data
        unsafe {
            let tcb = Tcb::new(self.static_tls_size, self.static_tls_align);
            if !tcb.is_null() {
                (*tcb).activate();
                self.initialize_static_tls(tcb);
            }
        }

        // 4. Relocate
        for i in 0..self.objects.len() {
            self.relocate_single(i);
        }

        // 5. Initialize
        for i in (0..self.objects.len()).rev() {
            unsafe { self.objects[i].run_init(); }
        }
    }

    fn load_dependencies(&mut self) {
        // Stub: BFS load DT_NEEDED
    }

    fn layout_static_tls(&mut self) {
        // Reset tracking
        self.tls_offset = 0;
        self.static_tls_align = 16;

        // 1. Layout Initial Modules
        for obj in &mut self.objects {
            if obj.tls_size == 0 { continue; }

            let align_mask = obj.tls_align - 1;
            self.tls_offset = (self.tls_offset + align_mask) & !align_mask;
            obj.tls_offset = self.tls_offset;
            self.tls_offset += obj.tls_size;
            
            if obj.tls_align > self.static_tls_align {
                self.static_tls_align = obj.tls_align;
            }
        }

        // 2. Mark end of Initial TLS
        self.static_tls_end_offset = self.tls_offset;

        // 3. Add Surplus
        // We add padding for runtime loaded libraries that might use IE model.
        self.surplus_remaining = STATIC_TLS_SURPLUS;
        self.static_tls_size = self.static_tls_end_offset + STATIC_TLS_SURPLUS;
    }

    /// Attempt to allocate from Static TLS Surplus (for dlopen).
    /// Returns Some(offset) if successful, None if surplus exhausted.
    pub fn try_allocate_static_tls(&mut self, size: usize, align: usize) -> Option<usize> {
        let current_end = self.static_tls_size - self.surplus_remaining;
        
        // Calculate aligned address
        let align_mask = align - 1;
        let start = (current_end + align_mask) & !align_mask;
        let end = start + size;

        if end <= self.static_tls_size {
            // Allocation successful
            self.surplus_remaining = self.static_tls_size - end;
            Some(start)
        } else {
            None
        }
    }

    unsafe fn initialize_static_tls(&self, tcb: *mut Tcb) {
        let tcb_addr = tcb as usize;
        
        #[cfg(target_arch = "x86_64")]
        // Variant II: Block starts at FS - TotalSize
        let block_start = tcb_addr.wrapping_sub(self.static_tls_size);

        #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
        // Variant I: Block starts at TP + Aligned(TCB)
        let block_start = {
             let tcb_size = mem::size_of::<Tcb>();
             let tcb_aligned = (tcb_size + self.static_tls_align - 1) & !(self.static_tls_align - 1);
             tcb_addr + tcb_aligned
        };

        for obj in &self.objects {
            if obj.tls_size == 0 { continue; }

            // Note: tls_offset here is relative to the very start of the allocated block
            // which INCLUDES the surplus space if it's at the start (Variant II vs I details apply).
            // Simplified: tls_offset is relative to `block_start`.
            
            let dest_addr = block_start + obj.tls_offset;
            let dest = dest_addr as *mut u8;

            if let Some(image) = obj.tls_image {
                ptr::copy_nonoverlapping(image.as_ptr(), dest, image.len());
            }

            let image_len = obj.tls_image.map(|s| s.len()).unwrap_or(0);
            if obj.tls_size > image_len {
                let tbss_ptr = dest.add(image_len);
                let tbss_size = obj.tls_size - image_len;
                ptr::write_bytes(tbss_ptr, 0, tbss_size);
            }
        }
        
        // The surplus area (at the end) remains zero-initialized by Tcb::new
    }

    fn relocate_single(&self, obj_idx: usize) {
        let obj = &self.objects[obj_idx];
        let rels = obj.relocations(); 
        
        for (r_type, sym_idx, offset, addend) in rels {
            let reloc_addr = obj.base_addr + offset;

            if unsafe { reloc::relocate(
                r_type, 0, 0, reloc_addr, addend, obj.base_addr, 
                obj.tls_module_id,
                obj.tls_offset, 
                self.static_tls_size
            ) } {
                continue;
            }

            let sym_name = match obj.get_sym_name(sym_idx) {
                Some(s) => s,
                None => continue, 
            };

            let ver_req = obj.get_version_req(sym_idx);
            let lookup = self.lookup_symbol(sym_name, ver_req.as_ref(), obj_idx);

            if let Some((res, tls_id, tls_off)) = lookup {
                unsafe {
                    if !reloc::relocate(
                        r_type,
                        res.value,
                        res.size,
                        reloc_addr,
                        addend,
                        obj.base_addr,
                        tls_id,
                        tls_off,
                        self.static_tls_size
                    ) {
                        reloc::relocate_copy(r_type, res.value, reloc_addr, res.size);
                    }
                }
            }
        }
    }

    fn lookup_symbol<'a>(
        &'a self,
        name: &str,
        ver_req: Option<&VersionReq>,
        skip_obj_idx: usize, 
    ) -> Option<(LookupResult, usize, usize)> {
        for (i, dso) in self.objects.iter().enumerate() {
            if i == skip_obj_idx { continue; }

            unsafe {
                if let Some(res) = find_symbol_linux_style(
                    name,
                    ver_req,
                    dso.sym_table(),
                    dso.str_table(),
                    dso.gnu_hash(),
                    dso.sysv_hash(),
                    dso.version_data(),
                    dso.base_addr,
                ) {
                    return Some((res, dso.tls_module_id, dso.tls_offset));
                }
            }
        }
        None
    }
}