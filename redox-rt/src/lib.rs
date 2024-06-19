#![no_std]
#![feature(array_chunks, int_roundings, let_chains, slice_ptr_get)]
#![forbid(unreachable_patterns)]

extern crate alloc;

pub mod arch;
pub mod proc;

// TODO: Replace auxvs with a non-stack-based interface, but keep getauxval for compatibility
#[path = "../../src/platform/auxv_defs.rs"]
pub mod auxv_defs;
