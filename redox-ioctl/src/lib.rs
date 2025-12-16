#![feature(macro_metavar_expr_concat)]
#![no_std]

extern crate alloc;

#[macro_use]
mod ioctl_data;
pub use ioctl_data::IoctlData;

pub mod drm;
