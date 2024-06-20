#![no_std]
#![feature(asm_const, array_chunks, int_roundings, let_chains, slice_ptr_get, sync_unsafe_cell, thread_local)]
#![forbid(unreachable_patterns)]

extern crate alloc;

#[macro_export]
macro_rules! asmfunction(
    ($name:ident $(-> $ret:ty)? : [$($asmstmt:expr),*$(,)?] <= [$($decl:ident = $(sym $symname:ident)?$(const $constval:expr)?),*$(,)?]$(,)? ) => {
        ::core::arch::global_asm!(concat!("
            .p2align 4
            .section .text.", stringify!($name), ", \"ax\", @progbits
            .globl ", stringify!($name), "
            .type ", stringify!($name), ", @function
        ", stringify!($name), ":
        ", $($asmstmt, "\n",)* "
            .size ", stringify!($name), ", . - ", stringify!($name), "
        "), $($decl = $(sym $symname)?$(const $constval)?),*);

        extern "C" {
            pub fn $name() $(-> $ret)?;
        }
    }
);

pub mod arch;
pub mod proc;

// TODO: Replace auxvs with a non-stack-based interface, but keep getauxval for compatibility
#[path = "../../src/platform/auxv_defs.rs"]
pub mod auxv_defs;

pub mod signal;
pub mod sync;
pub mod thread;
