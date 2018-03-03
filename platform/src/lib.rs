//! fcntl implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/fcntl.h.html

#![no_std]
#![allow(non_camel_case_types)]

#[macro_use]
extern crate syscall;

pub use sys::*;

#[cfg(all(not(feature="no_std"), target_os = "linux"))]
#[path="linux/mod.rs"]
mod sys;

#[cfg(all(not(feature="no_std"), target_os = "redox"))]
#[path="redox/mod.rs"]
mod sys;

pub mod types;
