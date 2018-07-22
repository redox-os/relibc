#![no_std]
#![allow(non_camel_case_types)]
#![feature(alloc, allocator_api, const_vec_new)]
#![cfg_attr(target_os = "redox", feature(thread_local))]

#[cfg_attr(target_os = "redox", macro_use)]
extern crate alloc;

#[cfg(all(not(feature = "no_std"), target_os = "linux"))]
#[macro_use]
extern crate sc;

#[cfg(all(not(feature = "no_std"), target_os = "redox"))]
extern crate syscall;

pub use allocator::*;

#[cfg(not(feature = "ralloc"))]
#[path = "allocator/dlmalloc.rs"]
mod allocator;

#[cfg(feature = "ralloc")]
#[path = "allocator/ralloc.rs"]
mod allocator;

pub use sys::*;

#[cfg(all(not(feature = "no_std"), target_os = "linux"))]
#[path = "linux/mod.rs"]
mod sys;

#[cfg(all(not(feature = "no_std"), target_os = "redox"))]
#[path = "redox/mod.rs"]
mod sys;

pub mod types;

use alloc::Vec;
use core::{mem, ptr};

use types::*;

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

pub const AF_INET: c_int = 2;
pub const SOCK_STREAM: c_int = 1;
pub const SOCK_DGRAM: c_int = 2;
pub const SOCK_NONBLOCK: c_int = 0o4000;
pub const SOCK_CLOEXEC: c_int = 0o2000000;

pub const SIG_BLOCK: c_int = 0;
pub const SIG_UNBLOCK: c_int = 1;
pub const SIG_SETMASK: c_int = 2;

pub type in_addr_t = [u8; 4];
pub type in_port_t = u16;
pub type sa_family_t = u16;
pub type socklen_t = u32;

#[repr(C)]
pub struct sockaddr {
    pub sa_family: sa_family_t,
    pub data: [c_char; 14],
}

#[repr(C)]
pub struct sigaction {
    pub sa_handler: extern "C" fn(c_int),
    pub sa_flags: c_ulong,
    pub sa_restorer: unsafe extern "C" fn(),
    pub sa_mask: sigset_t,
}

const NSIG: usize = 64;

pub type sigset_t = [c_ulong; NSIG / (8 * mem::size_of::<c_ulong>())];

//TODO #[thread_local]
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut errno: c_int = 0;

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut environ: *mut *mut c_char = ptr::null_mut();
#[allow(non_upper_case_globals)]
pub static mut inner_environ: Vec<*mut c_char> = Vec::new();
