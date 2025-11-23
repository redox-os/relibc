// relibc/src/abi_types.rs
#![no_std]

//! This module defines the structures and constants required for the dynamic linker
//! to interact with the kernel and parse ELF files. It includes definitions for
//! ELF headers, dynamic sections, symbol tables, and relocation entries.

pub mod syscall_defs {
    // System call numbers for x86_64 architecture
    #[cfg(target_arch = "x86_64")]
    pub const SYS_WRITE: u64 = 1;
    #[cfg(target_arch = "x86_64")]
    pub const SYS_OPEN: u64 = 2;
    #[cfg(target_arch = "x86_64")]
    pub const SYS_MMAP: u64 = 9;
    #[cfg(target_arch = "x86_64")]
    pub const SYS_EXIT: u64 = 60;
    #[cfg(target_arch = "x86_64")]
    pub const SYS_ARCH_PRCTL: u64 = 158;

    // System call numbers for AArch64 architecture
    #[cfg(target_arch = "aarch64")]
    pub const SYS_WRITE: u64 = 64;
    #[cfg(target_arch = "aarch64")]
    pub const SYS_OPEN: u64 = 56;
    #[cfg(target_arch = "aarch64")]
    pub const SYS_MMAP: u64 = 222;
    #[cfg(target_arch = "aarch64")]
    pub const SYS_EXIT: u64 = 93;

    // System call numbers for RISC-V architecture
    #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))]
    pub const SYS_WRITE: u64 = 64;
    #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))]
    pub const SYS_OPEN: u64 = 56;
    #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))]
    pub const SYS_MMAP: u64 = 222;
    #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))]
    pub const SYS_EXIT: u64 = 93;
}

// Type definitions for 64-bit ELF structures
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
    pub st_name: Elf64_Word,
    pub st_info: u8,
    pub st_other: u8,
    pub st_shndx: Elf64_Half,
    pub st_value: Elf64_Addr,
    pub st_size: Elf64_Xword,
}

/// ELF Relocation Entry with Addend
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Elf64_Rela {
    pub r_offset: Elf64_Addr,
    pub r_info: Elf64_Xword,
    pub r_addend: Elf64_Sxword,
}

/// ELF Program Header
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Elf64_Phdr {
    pub p_type: Elf64_Word,
    pub p_flags: Elf64_Word,
    pub p_offset: Elf64_Off,
    pub p_vaddr: Elf64_Addr,
    pub p_paddr: Elf64_Addr,
    pub p_filesz: Elf64_Xword,
    pub p_memsz: Elf64_Xword,
    pub p_align: Elf64_Xword,
}

// Program Header Types
pub const PT_LOAD: u32 = 1;
pub const PT_DYNAMIC: u32 = 2;
pub const PT_TLS: u32 = 7;

// Dynamic Tags
pub const DT_NULL: Elf64_Sxword = 0;
pub const DT_NEEDED: Elf64_Sxword = 1;
pub const DT_PLTRELSZ: Elf64_Sxword = 2;
pub const DT_PLTGOT: Elf64_Sxword = 3;
pub const DT_HASH: Elf64_Sxword = 4;
pub const DT_STRTAB: Elf64_Sxword = 5;
pub const DT_SYMTAB: Elf64_Sxword = 6;
pub const DT_RELA: Elf64_Sxword = 7;
pub const DT_RELASZ: Elf64_Sxword = 8;
pub const DT_RELAENT: Elf64_Sxword = 9;
pub const DT_INIT: Elf64_Sxword = 12;
pub const DT_FINI: Elf64_Sxword = 13;
pub const DT_RPATH: Elf64_Sxword = 15;
pub const DT_INIT_ARRAY: Elf64_Sxword = 25;
pub const DT_FINI_ARRAY: Elf64_Sxword = 26;
pub const DT_INIT_ARRAYSZ: Elf64_Sxword = 27;
pub const DT_FINI_ARRAYSZ: Elf64_Sxword = 28;
pub const DT_RUNPATH: Elf64_Sxword = 29;

// Relocation Constants

// x86-64
pub const R_X86_64_64: u32 = 1;
pub const R_X86_64_GLOB_DAT: u32 = 6;
pub const R_X86_64_JUMP_SLOT: u32 = 7;
pub const R_X86_64_RELATIVE: u32 = 8;
pub const R_X86_64_DTPMOD64: u32 = 16;
pub const R_X86_64_DTPOFF64: u32 = 17;
pub const R_X86_64_TPOFF64: u32 = 18;

// AArch64
pub const R_AARCH64_ABS64: u32 = 257;
pub const R_AARCH64_GLOB_DAT: u32 = 1025;
pub const R_AARCH64_JUMP_SLOT: u32 = 1026;
pub const R_AARCH64_RELATIVE: u32 = 1027;
pub const R_AARCH64_TLS_DTPMOD64: u32 = 1028;
pub const R_AARCH64_TLS_DTPREL64: u32 = 1029;
pub const R_AARCH64_TLS_TPREL64: u32 = 1030;

// RISC-V
pub const R_RISCV_64: u32 = 2;
pub const R_RISCV_RELATIVE: u32 = 3;
pub const R_RISCV_JUMP_SLOT: u32 = 5;
pub const R_RISCV_TLS_DTPMOD64: u32 = 7;
pub const R_RISCV_TLS_DTPREL64: u32 = 9;
pub const R_RISCV_TLS_TPREL64: u32 = 11;

// Helper for syscalls (x86-64 specific)
pub const ARCH_SET_FS: u64 = 0x1002;
pub const ARCH_SET_GS: u64 = 0x1001;