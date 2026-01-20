//! POSIX C library, implemented in Rust.
//!
//! This crate exists to provide a standard libc as its public API. This is
//! largely provided by automatically generated bindings to the functions and
//! data structures in the [`header`] module.
//!
//! Currently, Linux and Redox syscall backends are supported.

#![no_std]
#![allow(warnings)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(unused_variables)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(asm_const)]
#![feature(c_variadic)]
#![feature(core_intrinsics)]
#![feature(macro_derive)]
#![feature(maybe_uninit_slice)]
#![feature(lang_items)]
#![feature(linkage)]
#![feature(pointer_is_aligned_to)]
#![feature(ptr_as_uninit)]
#![feature(slice_as_chunks)]
#![feature(slice_ptr_get)]
#![feature(stmt_expr_attributes)]
#![feature(strict_provenance)]
#![feature(sync_unsafe_cell)]
#![feature(thread_local)]
#![feature(vec_into_raw_parts)]
#![feature(negative_impls)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(clippy::derive_hash_xor_eq)]
#![allow(clippy::eval_order_dependence)]
#![allow(clippy::mut_from_ref)]
#![deny(unsafe_op_in_unsafe_fn)]
// TODO: fix these
#![warn(unaligned_references)]

#[macro_use]
extern crate alloc;
extern crate cbitset;
extern crate memchr;
extern crate posix_regex;
extern crate rand;

#[cfg(target_os = "linux")]
#[macro_use]
extern crate sc;

#[cfg(target_os = "redox")]
extern crate syscall;

#[macro_use]
mod macros;
pub mod c_str;
pub mod c_vec;
pub mod cxa;
pub mod db;
pub mod error;
pub mod fs;
pub mod header;
pub mod io;
pub mod iter;
pub mod ld_so;
pub mod out;
pub mod platform;
pub mod pthread;
pub mod raw_cell;
pub mod start;
pub mod sync;

use crate::platform::{Allocator, NEWALLOCATOR, Pal, Sys};

#[global_allocator]
static ALLOCATOR: Allocator = NEWALLOCATOR;

#[unsafe(no_mangle)]
pub extern "C" fn relibc_panic(pi: &::core::panic::PanicInfo) -> ! {
    use core::fmt::Write;

    let mut w = platform::FileWriter::new(2);
    let _ = w.write_fmt(format_args!("RELIBC PANIC: {}\n", pi));

    core::intrinsics::abort();
}

#[cfg(not(test))]
#[panic_handler]
#[linkage = "weak"]
pub fn rust_begin_unwind(pi: &::core::panic::PanicInfo) -> ! {
    relibc_panic(pi)
}

#[cfg(not(test))]
#[lang = "eh_personality"]
#[linkage = "weak"]
pub extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[alloc_error_handler]
#[linkage = "weak"]
#[unsafe(no_mangle)]
pub extern "C" fn rust_oom(layout: ::core::alloc::Layout) -> ! {
    use core::fmt::Write;

    let mut w = platform::FileWriter::new(2);
    let _ = w.write_fmt(format_args!(
        "RELIBC OOM: {} bytes aligned to {} bytes\n",
        layout.size(),
        layout.align()
    ));

    core::intrinsics::abort();
}

#[cfg(not(test))]
#[allow(non_snake_case)]
#[linkage = "weak"]
#[unsafe(no_mangle)]
pub extern "C" fn _Unwind_Resume() -> ! {
    use core::fmt::Write;

    let mut w = platform::FileWriter::new(2);
    let _ = w.write_str("_Unwind_Resume\n");

    core::intrinsics::abort();
}
