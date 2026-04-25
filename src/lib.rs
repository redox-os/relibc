//! POSIX C library, implemented in Rust.
//!
//! This crate exists to provide a standard libc as its public API. This is
//! largely provided by automatically generated bindings to the functions and
//! data structures in the [`header`] module.
//!
//! Currently, Linux and Redox syscall backends are supported.
//!
//! # Scope
//! As a general rule, relibc aims to implement the most recent version of
//! POSIX, as well as common extensions and legacy interfaces as deemed
//! appropriate. Thus, relibc's documentation will generally defer to the
//! POSIX standard or to Linux man pages, except to document quirks, safety
//! requirements or internal interfaces.
//!
//! * See [The Open Group Base Specifications Issue 8](https://pubs.opengroup.org/onlinepubs/9799919799/) (POSIX.1-2024) for the current POSIX spec.
//! * Linux man pages are available online at [man7.org](https://www.man7.org/linux/man-pages/index.html) and [linux.die.net](https://linux.die.net/man/).
//!
//! For an overview of the relevant standards, see the
//! [`standards` man page](https://www.man7.org/linux/man-pages/man7/standards.7.html).
//! Historical versions of the POSIX specification are available here:
//! * [X/Open CAE Specification: System Interfaces and Headers Issue 4, Version 2](https://pubs.opengroup.org/onlinepubs/9695969499/toc.pdf) (1994)
//! * [The Single UNIX® Specification, Version 2](https://pubs.opengroup.org/onlinepubs/7908799/index.html) (1997)
//! * [The Open Group Base Specifications Issue 6 (2004 edition)](https://pubs.opengroup.org/onlinepubs/009695399/) (POSIX.1-2001 including Technical Corrigenda 1 and 2)
//! * [The Open Group Base Specifications Issue 7](http://pubs.opengroup.org/onlinepubs/9699919799.2008edition/) (POSIX.1-2008)
//!     * [Issue 7 (2013 edition)](https://pubs.opengroup.org/onlinepubs/9699919799.2013edition/) (POSIX.1-2008 including Technical Corrigendum 1)
//!     * [Issue 7 (2016 edition)](http://pubs.opengroup.org/onlinepubs/9699919799.2016edition/) (POSIX.1-2008 including Technical Corrigenda 1 and 2)
//!     * [Issue 7 (2018 edition)](https://pubs.opengroup.org/onlinepubs/9699919799/) (POSIX.1-2017)

#![no_std]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(c_variadic)]
#![feature(core_intrinsics)]
#![feature(macro_derive)]
#![feature(maybe_uninit_slice)]
#![feature(lang_items)]
#![feature(linkage)]
#![feature(pointer_is_aligned_to)]
#![feature(ptr_as_uninit)]
#![feature(slice_ptr_get)]
#![feature(stmt_expr_attributes)]
#![feature(sync_unsafe_cell)]
#![feature(thread_local)]
#![feature(vec_into_raw_parts)]
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

use crate::platform::{Allocator, NEWALLOCATOR};

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
#[allow(improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub extern "C" fn rust_oom(layout: ::core::alloc::Layout) -> ! {
    // Layout not FFI-safe?
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
