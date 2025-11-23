// ld_so/src/lib.rs
//! The Relibc User-Space Dynamic Linker (ld.so)
//!
//! This crate implements the ELF dynamic linking process for Redox and Linux.
//! It is a freestanding Pure Rust library that bootstraps itself.

#![no_std]
#![feature(naked_functions)]
#![feature(link_args)]
#![allow(clippy::missing_safety_doc)]

extern crate alloc;

// --- Core Modules ---
pub mod callbacks;
pub mod debug;
pub mod dso;
pub mod header; // Assumes symlink or path to main relibc header module
pub mod linker;
pub mod reloc;
pub mod start;
pub mod tcb;
pub mod verify;

// --- Linux Parity Modules ---
pub mod gnu_hash;
pub mod linux_parity;
pub mod versioning;

use core::arch::global_asm;
use crate::linker::Linker;
use crate::dso::DSO;

// Define the entry point _start.
// This symbol is jumped to by the kernel after mapping the interpreter.
// We must capture the stack pointer (sp) which points to [argc, argv..., envp..., auxv...].

#[cfg(target_arch = "x86_64")]
global_asm!(
    ".globl _start",
    "_start:",
    "mov rdi, rsp", // Arg 1: Stack Pointer
    "call linker_entry",
    "ud2" // Should not return
);

#[cfg(target_arch = "aarch64")]
global_asm!(
    ".globl _start",
    "_start:",
    "mov x0, sp", // Arg 1: Stack Pointer
    "bl linker_entry",
    "brk #1"
);

#[cfg(target_arch = "riscv64")]
global_asm!(
    ".globl _start",
    "_start:",
    "mv a0, sp", // Arg 1: Stack Pointer
    "call linker_entry",
    "unimp"
);

/// The Rust Entry Point.
///
/// # Safety
/// This function is unsafe because it operates on raw stack pointers and relies on
/// correct kernel initialization.
#[no_mangle]
pub unsafe extern "C" fn linker_entry(sp: *const usize) -> ! {
    // 1. Calculate Load Base
    // We need to know where ld.so is loaded in memory to handle self-relocation
    // and find our own dynamic section.
    // Ideally, we find AT_BASE from the aux vector on the stack, but we might need
    // to calculate it manually if we haven't parsed auxv yet.
    // For now, we assume we can get it from a helper or calculation.
    let load_base = get_load_base();

    // 2. Self-Verification
    // Ensure our own relocations (processed by asm or previous stage) are valid.
    if let Err(e) = verify::verify_self_integrity(load_base) {
        // If we can't print (libc not ready), we might just have to crash or use a raw syscall write.
        // debug::panic_raw(e); 
        core::intrinsics::abort();
    }

    // 3. Initialize the Linker
    let mut linker = Linker::new();

    // 4. Bootstrap Main Executable
    // In a real implementation, we parse argv/envp/auxv here to find the executable path
    // or file descriptor passed by the kernel.
    // For this contract freeze, we construct a placeholder DSO for the main app.
    
    // This would typically be parsed from AT_PHDR / AT_ENTRY in the aux vector
    let main_dso = DSO::new_executable(sp);
    
    // 5. Perform Linking
    linker.link(main_dso);

    // 6. Enter Application
    // The linker.link() function handles initizers.
    // Now we jump to the entry point of the main executable.
    // We must restore the stack pointer to the state expected by the app's _start.
    enter_entry_point(sp, get_entry_point(sp));
}

/// Calculates the load base of the dynamic linker itself.
/// Uses PC-relative addressing trickery or AUX vectors.
#[inline(always)]
unsafe fn get_load_base() -> usize {
    // Simplified: On modern Linux/Redox, the kernel passes the interpreter base in AT_BASE (auxv[7]).
    // However, accessing it requires walking `sp`.
    // A robust impl would walk `sp` past argc, argv, envp to find auxv.
    // Here we stub it for the contract.
    0 // Placeholder: Implement auxv walker or PC-relative calculation
}

/// Helper to extract the entry point from the stack (AT_ENTRY).
#[inline(always)]
unsafe fn get_entry_point(sp: *const usize) -> usize {
    // Placeholder: Walk sp to find AT_ENTRY
    0 
}

/// Jump to the application entry point.
#[cfg(target_arch = "x86_64")]
unsafe fn enter_entry_point(sp: *const usize, entry: usize) -> ! {
    core::arch::asm!(
        "mov rsp, {0}",
        "jmp {1}",
        in(reg) sp,
        in(reg) entry,
        options(noreturn)
    )
}

#[cfg(target_arch = "aarch64")]
unsafe fn enter_entry_point(sp: *const usize, entry: usize) -> ! {
    core::arch::asm!(
        "mov sp, {0}",
        "br {1}",
        in(reg) sp,
        in(reg) entry,
        options(noreturn)
    )
}

#[cfg(target_arch = "riscv64")]
unsafe fn enter_entry_point(sp: *const usize, entry: usize) -> ! {
    core::arch::asm!(
        "mv sp, {0}",
        "jr {1}",
        in(reg) sp,
        in(reg) entry,
        options(noreturn)
    )
}

// Panic handler for no_std
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort()
}

#[no_mangle]
pub extern "C" fn __dso_handle() {}

#[no_mangle]
pub extern "C" fn __rust_alloc(size: usize, align: usize) -> *mut u8 {
    // Linker needs a simple bump allocator or similar since malloc isn't up yet.
    // For now, return null to indicate strict no-alloc requirement in bootstrap.
    core::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn __rust_dealloc(_ptr: *mut u8, _size: usize, _align: usize) {
    // No-op
}

#[no_mangle]
pub extern "C" fn __rust_realloc(_ptr: *mut u8, _old_size: usize, _align: usize, _new_size: usize) -> *mut u8 {
    core::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn __rust_alloc_zeroed(_size: usize, _align: usize) -> *mut u8 {
    core::ptr::null_mut()
}