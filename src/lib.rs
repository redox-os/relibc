//! POSIX C library, implemented in Rust.
//!
//! This crate exists to provide a standard libc as its public API. This is
//! largely provided by automatically generated bindings to the functions and
//! data structures in the [`header`] module.
//!
//! Currently, Linux and Redox syscall backends are supported.

#![no_std]
#![feature(alloc_error_handler)]
#![feature(c_variadic)]
#![feature(core_intrinsics)]
#![feature(macro_derive)]
#![feature(lang_items)]
#![feature(linkage)]
#![feature(ptr_as_uninit)]
#![feature(slice_ptr_get)]
#![feature(stmt_expr_attributes)]
#![feature(sync_unsafe_cell)]
#![feature(thread_local)]
#![feature(negative_impls)]

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
pub mod casting;
pub mod cxa;
pub mod db;
pub mod error;
pub mod fs;
pub mod header;
pub mod io;
pub mod iter;
pub mod ld_so;
pub mod out;
pub mod panic;
pub mod platform;
pub mod pthread;
pub mod raw_cell;
pub mod start;
pub mod sync;

use crate::platform::{Allocator, NEWALLOCATOR};

#[global_allocator]
static ALLOCATOR: Allocator = NEWALLOCATOR;

#[cfg(not(test))]
#[panic_handler]
#[linkage = "weak"]
pub fn rust_begin_unwind(pi: &::core::panic::PanicInfo) -> ! {
    crate::panic::relibc_panic(pi)
}

#[cfg(not(test))]
#[lang = "eh_personality"]
#[linkage = "weak"]
pub extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[alloc_error_handler]
#[linkage = "weak"]
#[expect(improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub extern "C" fn rust_oom(layout: ::core::alloc::Layout) -> ! {
    panic!(
        "RELIBC OOM: {} bytes aligned to {} bytes",
        layout.size(),
        layout.align()
    );
}

#[cfg(not(test))]
#[allow(non_snake_case)]
#[linkage = "weak"]
#[unsafe(no_mangle)]
pub extern "C" fn _Unwind_Resume() -> ! {
    panic!("_Unwind_Resume")
}
