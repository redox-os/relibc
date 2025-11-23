// relibc/src/abi_types.rs
#![no_std]

//! ABI Types and Constants for Redox OS (Phase 2: Symbol Resolution)
//!
//! This module defines the strict memory layout for ELF structures, symbol tables,
//! and relocation constants required for a functional dynamic linker.

pub mod syscall_defs {
    #[cfg(target_arch = "x86_64")]
    pub const SYS_WRITE: u64 = 1;
    #[cfg(target_arch = "x86_64")]
    pub const SYS_EXIT: u64 = 60;

    #[cfg(target_arch = "aarch64")]
    pub const SYS_WRITE: u64 = 64;
    #[cfg(target_arch = "aarch64")]
    pub const SYS_EXIT: u64 = 93;

    #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))]
    pub const SYS_WRITE: u64 = 64;
    #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))]
    pub const SYS_EXIT: u64 = 93;
}

pub type Elf64_Addr = u64;
pub type Elf64_Off = u64;
pub type Elf64_Xword = u64;
pub type Elf64_Sxword = i64;
pub type Elf64_Word = u32;
pub type Elf64_Half = u16;

/// ELF Dynamic Section Entry
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Elf64_Dyn {
    pub d_tag: Elf64_Sxword,
    pub d_un: Elf64_Xword,
}

/// ELF Symbol Table Entry
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Elf64_Sym {
    pub st_name: Elf64_Word,  // Index into string table
    pub st_info: u8,          // Type and Binding attributes
    pub st_other: u8,         // Visibility
    pub st_shndx: Elf64_Half, // Section index
    pub st_value: Elf64_Addr, // Value (address)
    pub st_size: Elf64_Xword, // Size of object
}

/// ELF Relocation Entry with Addend
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Elf64_Rela {
    pub r_offset: Elf64_Addr,
    pub r_info: Elf64_Xword,
    pub r_addend: Elf64_Sxword,
}

// Dynamic Section Tags
pub const DT_NULL: Elf64_Sxword = 0;
pub const DT_HASH: Elf64_Sxword = 4;
pub const DT_STRTAB: Elf64_Sxword = 5;
pub const DT_SYMTAB: Elf64_Sxword = 6;
pub const DT_RELA: Elf64_Sxword = 7;
pub const DT_RELASZ: Elf64_Sxword = 8;
pub const DT_RELAENT: Elf64_Sxword = 9;

// Symbol Binding (st_info >> 4)
pub const STB_LOCAL: u8 = 0;
pub const STB_GLOBAL: u8 = 1;
pub const STB_WEAK: u8 = 2;

// Symbol Visibility (st_other & 0x3)
pub const STV_DEFAULT: u8 = 0;
pub const STV_INTERNAL: u8 = 1;
pub const STV_HIDDEN: u8 = 2;
pub const STV_PROTECTED: u8 = 3;

// Relocation Constants
// x86-64
pub const R_X86_64_64: u32 = 1;
pub const R_X86_64_GLOB_DAT: u32 = 6; // GOT entry
pub const R_X86_64_JUMP_SLOT: u32 = 7; // PLT entry
pub const R_X86_64_RELATIVE: u32 = 8;

// AArch64
pub const R_AARCH64_ABS64: u32 = 257;
pub const R_AARCH64_GLOB_DAT: u32 = 1025;
pub const R_AARCH64_JUMP_SLOT: u32 = 1026;
pub const R_AARCH64_RELATIVE: u32 = 1027;

// RISC-V
pub const R_RISCV_64: u32 = 2;
pub const R_RISCV_RELATIVE: u32 = 3;
pub const R_RISCV_JUMP_SLOT: u32 = 5;