// ld_so/src/linker.rs
#![no_std]

use core::ptr;
// Import the stabilized ABI types defined in relibc
use relibc::abi_types::{Elf64_Addr, Elf64_Rela, Elf64_Xword};
use relibc::abi_types::{R_X86_64_RELATIVE, R_AARCH64_RELATIVE, R_RISCV_RELATIVE};

/// Extracts the relocation type from the r_info field.
/// The type is stored in the lower 32 bits.
#[inline(always)]
fn elf64_r_type(info: Elf64_Xword) -> u32 {
    (info & 0xffffffff) as u32
}

/// Core relocation logic for position-independent shared objects.
///
/// This function iterates over the ELF relocation table and applies base address patches
/// (R_*_RELATIVE) required for the shared object to run at the loaded virtual address.
///
/// # Arguments
///
/// * `base_address` - The base virtual address (B) where the object is loaded.
/// * `rela_table_ptr` - Raw pointer to the start of the .rela.dyn section.
/// * `rela_size` - The total size in bytes of the relocation table.
///
/// # Safety
///
/// This function is strictly unsafe. It performs arbitrary writes to memory addresses
/// derived from the relocation table. The caller must ensure that `base_address` is valid
/// and that the memory segments pointed to by `r_offset` are writable.
pub unsafe fn relocate_shared_object(
    base_address: Elf64_Addr,
    rela_table_ptr: *const Elf64_Rela,
    rela_size: usize,
) {
    // Calculate the number of entries in the relocation table
    let entry_size = core::mem::size_of::<Elf64_Rela>();
    let entry_count = rela_size / entry_size;
    
    // Create a safe slice wrapper over the raw pointer for iteration
    let rela_slice = core::slice::from_raw_parts(rela_table_ptr, entry_count);

    for rela in rela_slice {
        let r_type = elf64_r_type(rela.r_info);

        // Check if this is a relative relocation for the current architecture.
        // This calculation (Base + Addend) is mathematically identical across these ISAs.
        let is_relative = if cfg!(target_arch = "x86_64") {
            r_type == R_X86_64_RELATIVE
        } else if cfg!(target_arch = "aarch64") {
            r_type == R_AARCH64_RELATIVE
        } else if cfg!(target_arch = "riscv64") {
            r_type == R_RISCV_RELATIVE
        } else {
            false
        };

        if is_relative {
            // Calculation: New Value = Base Address (B) + Addend (A)
            
            // 1. Calculate the pointer to the memory location to be patched (P)
            //    P = base_address + r_offset
            let patch_ptr = (base_address.wrapping_add(rela.r_offset)) as *mut u64;

            // 2. Calculate the value to write
            //    Val = base_address + r_addend
            //    Note: r_addend is signed (i64), so we cast to u64 for wrapping arithmetic.
            let val = base_address.wrapping_add(rela.r_addend as u64);

            // 3. Perform a volatile write to ensure the patch actually happens.
            //    Volatile is crucial here to prevent the compiler from assuming 
            //    that 'text/data' segments are immutable or optimizing out the write.
            ptr::write_volatile(patch_ptr, val);
        } else {
            // Non-relative relocations (e.g., JUMP_SLOT, GLOB_DAT) require symbol resolution.
            // Per the Phase 1 requirements, we explicitly defer these.
            continue;
        }
    }
}