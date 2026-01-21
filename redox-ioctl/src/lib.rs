#![feature(macro_metavar_expr_concat)]
#![deny(unsafe_op_in_unsafe_fn)]
#![no_std]

extern crate alloc;

#[macro_use]
mod ioctl_data;
pub use ioctl_data::IoctlData;

pub mod drm;
