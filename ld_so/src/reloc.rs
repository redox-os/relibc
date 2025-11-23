// ld_so/src/reloc.rs
//! Core Relocation Logic for x86-64, AArch64, and RISC-V.
//! Fully implemented with ELF TLS support.

use core::mem::size_of;
use crate::header::elf;
use crate::linux_parity::resolve_ifunc;
use crate::tcb::Tcb;

/// Performs a single relocation.
///
/// # Arguments
/// * `r_type`: The raw relocation type.
/// * `sym_val`: The symbol's value (Virtual Address for normal symbols, Offset for TLS).
/// * `sym_size`: Size of the symbol.
/// * `reloc_addr`: Memory address to patch.
/// * `addend`: Relocation addend.
/// * `base_addr`: Load base of the object being relocated.
/// * `tls_module_id`: The ID of the module *defining* the symbol (for DTPMOD).
/// * `tls_offset`: The offset of the defining module within the static TLS block (for TPOFF).
/// * `static_tls_size`: Total size of the static TLS block (used for x86_64 TPOFF calculations).
pub unsafe fn relocate(
    r_type: u32,
    sym_val: usize,
    sym_size: usize,
    reloc_addr: usize,
    addend: Option<usize>,
    base_addr: usize,
    tls_module_id: usize,
    tls_offset: usize,
    static_tls_size: usize,
) -> bool {
    let ptr = reloc_addr as *mut usize;
    let val = addend.unwrap_or(0);

    #[cfg(target_arch = "x86_64")]
    {
        match r_type {
            elf::R_X86_64_64 => *ptr = sym_val.wrapping_add(val),
            elf::R_X86_64_GLOB_DAT | elf::R_X86_64_JUMP_SLOT => *ptr = sym_val,
            elf::R_X86_64_RELATIVE => *ptr = base_addr.wrapping_add(val),
            elf::R_X86_64_IRELATIVE => {
                let resolver = base_addr.wrapping_add(val);
                *ptr = resolve_ifunc(resolver);
            }
            // --- TLS Relocations (Variant II) ---
            // DTPMOD64 (16): ID of the module containing the symbol.
            16 => *ptr = tls_module_id,
            // DTPOFF64 (17): Offset within that module's TLS block.
            17 => *ptr = sym_val.wrapping_add(val),
            // TPOFF64 (18): Offset from the Thread Pointer (FS).
            // Variant II (x86): The TLS block is located *below* the TCB (FS register).
            // addresses are negative relative to FS.
            // Structure: [ TLS Block (size) ] [ TCB ]
            //                                 ^ %fs
            // The offset is calculated as: (ModuleOffset + SymbolOffset) - TotalStaticSize + Addend
            18 => {
                let offset_from_start = tls_offset.wrapping_add(sym_val).wrapping_add(val);
                // We subtract static_tls_size because the block ends at %fs.
                // e.g. if size is 1024, and we are at offset 0, result is -1024.
                *ptr = offset_from_start.wrapping_sub(static_tls_size);
            }
            _ => return false,
        }
        true
    }

    #[cfg(target_arch = "aarch64")]
    {
        match r_type {
            elf::R_AARCH64_ABS64 | elf::R_AARCH64_GLOB_DAT | elf::R_AARCH64_JUMP_SLOT => {
                *ptr = sym_val.wrapping_add(val);
            }
            elf::R_AARCH64_RELATIVE => *ptr = base_addr.wrapping_add(val),
            elf::R_AARCH64_IRELATIVE => {
                let resolver = base_addr.wrapping_add(val);
                *ptr = resolve_ifunc(resolver);
            }
            // --- TLS Relocations (Variant I) ---
            // TLS_DTPMOD64 (1028)
            1028 => *ptr = tls_module_id,
            // TLS_DTPREL64 (1029)
            1029 => *ptr = sym_val.wrapping_add(val),
            // TLS_TPREL64 (1030): Offset from Thread Pointer (TPIDR_EL0).
            // Variant I: TCB is at TP. TLS follows TCB.
            // Structure: [ TCB ] [ TLS Block ]
            //            ^ %tp
            // Offset = Align(TCB_SIZE) + ModuleOffset + SymbolOffset + Addend
            1030 => {
                let tcb_size = size_of::<Tcb>();
                // Ensure 16-byte alignment for TCB size
                let tcb_aligned = (tcb_size + 15) & !15;
                *ptr = tcb_aligned.wrapping_add(tls_offset).wrapping_add(sym_val).wrapping_add(val);
            }
            _ => return false,
        }
        true
    }

    #[cfg(target_arch = "riscv64")]
    {
        match r_type {
            elf::R_RISCV_64 => *ptr = sym_val.wrapping_add(val),
            elf::R_RISCV_JUMP_SLOT => *ptr = sym_val,
            elf::R_RISCV_RELATIVE => *ptr = base_addr.wrapping_add(val),
            58 => { // R_RISCV_IRELATIVE
                let resolver = base_addr.wrapping_add(val);
                *ptr = resolve_ifunc(resolver);
            }
            // --- TLS Relocations (Variant I) ---
            // R_RISCV_TLS_DTPMOD64 (7)
            7 => *ptr = tls_module_id,
            // R_RISCV_TLS_DTPREL64 (8)
            8 => *ptr = sym_val.wrapping_add(val),
            // R_RISCV_TLS_TPREL64 (9)
            // Logic identical to AArch64 (Variant I)
            9 => {
                let tcb_size = size_of::<Tcb>();
                let tcb_aligned = (tcb_size + 15) & !15;
                *ptr = tcb_aligned.wrapping_add(tls_offset).wrapping_add(sym_val).wrapping_add(val);
            }
            _ => return false,
        }
        true
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
    {
        false
    }
}

pub unsafe fn relocate_copy(
    r_type: u32,
    src_addr: usize,
    dst_addr: usize,
    size: usize,
) -> bool {
    let is_copy = 
        (cfg!(target_arch = "x86_64") && r_type == elf::R_X86_64_COPY) ||
        (cfg!(target_arch = "aarch64") && r_type == elf::R_AARCH64_COPY) ||
        (cfg!(target_arch = "riscv64") && r_type == elf::R_RISCV_COPY);

    if is_copy {
        // 
        // R_COPY copies initial data from the shared library to the executable's .bss
        // so the executable can access the variable directly via absolute addressing.
        let src = src_addr as *const u8;
        let dst = dst_addr as *mut u8;
        for i in 0..size {
            *dst.add(i) = *src.add(i);
        }
        true
    } else {
        false
    }
}