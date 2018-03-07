//! fcntl implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/fcntl.h.html

#![no_std]
#![allow(non_camel_case_types)]

#[cfg(all(not(feature = "no_std"), target_os = "linux"))]
#[macro_use]
extern crate sc;

#[cfg(all(not(feature = "no_std"), target_os = "redox"))]
#[macro_use]
extern crate syscall;

pub use sys::*;

#[cfg(all(not(feature = "no_std"), target_os = "linux"))]
#[path = "linux/mod.rs"]
mod sys;

#[cfg(all(not(feature = "no_std"), target_os = "redox"))]
#[path = "redox/mod.rs"]
mod sys;

pub mod types;

use core::fmt;

use types::*;

pub unsafe fn c_str(s: *const c_char) -> &'static [u8] {
    use core::slice;

    let mut size = 0;

    loop {
        if *s.offset(size) == 0 {
            break;
        }
        size += 1;
    }

    slice::from_raw_parts(s as *const u8, size as usize)
}

pub struct FileWriter(pub c_int);

impl FileWriter {
    pub fn write(&mut self, buf: &[u8]) {
        write(self.0, buf);
    }
}

impl fmt::Write for FileWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}
