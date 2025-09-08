//! Implementation specific, non-standard path aliases

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
mod sys;

pub use sys::*;
