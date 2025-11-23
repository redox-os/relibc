// relibc/src/abi_types.rs
#![no_std]

//! ABI Types and Constants for Redox OS
//!
//! This module defines the strict memory layout for ELF structures and system call definitions
//! required to stabilize the interface between the kernel, the dynamic linker (ld.so),
//! and user-space applications across x86-64, AArch64, and RISC-V.

pub mod syscall_defs {
    /// System call number for writing to a file descriptor.
    /// Uses x86-64 Linux convention as the primary reference.
    #[cfg(target_arch = "x86_64")]
    pub const SYS_WRITE: u64 = 1;
    
    /// System call number for exiting the current process.
    #[cfg(target_arch = "x86_64")]
    pub const SYS_EXIT: u64 = 60;

    // AArch64 Linux/Redox syscall constants
    #[cfg(target_arch = "aarch64")]
    pub const SYS_WRITE: u64 = 64;
    #[cfg(target_arch = "aarch64")]
    pub const SYS_EXIT: u64 = 93;

    // RISC-V Linux/Redox syscall constants
    #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))]
    pub const SYS_WRITE: u64 = 64;
    #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))]
    pub const SYS_EXIT: u64 = 93;
}

// ELF 64-bit Type Definitions
pub type Elf64_Addr = u64;
pub type Elf64_Off = u64;
pub type Elf64_Xword = u64;
pub type Elf64_Sxword = i64;
pub type Elf64_Word = u32;

/// ELF Dynamic Section Entry
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Elf64_Dyn {
    pub d_tag: Elf64_Sxword, // Dynamic entry type tag
    pub d_un: Elf64_Xword,   // Integer value or address
}

/// ELF Relocation Entry with Addend
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Elf64_Rela {
    pub r_offset: Elf64_Addr,  // Location at which to apply the action
    pub r_info: Elf64_Xword,   // Symbol table index and type of relocation
    pub r_addend: Elf64_Sxword,// Constant addend used to compute value
}

// Critical Relocation Constants for Base Patching
// These are the specific constants required for R_*_RELATIVE relocations
// on the supported architectures.

/// AMD x86-64: B + A
pub const R_X86_64_RELATIVE: u32 = 8;

/// ARM AArch64: B + A
pub const R_AARCH64_RELATIVE: u32 = 1027;

/// RISC-V: B + A
pub const R_RISCV_RELATIVE: u32 = 3;