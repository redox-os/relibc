//! fcntl implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/fcntl.h.html

#![no_std]
#![allow(non_camel_case_types)]
#![feature(alloc)]
#![feature(global_allocator)]
//TODO #![feature(thread_local)]

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

extern crate alloc;
extern crate ralloc;

pub mod types;

use core::fmt;

use types::*;

#[global_allocator]
static ALLOCATOR: ralloc::Allocator = ralloc::Allocator;
//TODO #[thread_local]
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut errno: c_int = 0;

pub unsafe fn c_str(s: *const c_char) -> &'static [u8] {
    use core::usize;

    c_str_n(s, usize::MAX)
}

pub unsafe fn c_str_n(s: *const c_char, n: usize) -> &'static [u8] {
    use core::slice;

    let mut size = 0;

    for _ in 0..n {
        if *s.offset(size) == 0 {
            break;
        }
        size += 1;
    }

    slice::from_raw_parts(s as *const u8, size as usize)
}

pub unsafe fn cstr_from_bytes_with_nul_unchecked(bytes: &[u8]) -> *const c_char {
    &*(bytes as *const [u8] as *const c_char)
}

pub struct FileWriter(pub c_int);

impl FileWriter {
    pub fn write(&mut self, buf: &[u8]) -> isize {
        write(self.0, buf)
    }
}

impl fmt::Write for FileWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

pub struct FileReader(pub c_int);

impl FileReader {
    pub fn read(&mut self, buf: &mut [u8]) -> isize {
        read(self.0, buf)
    }
}

pub struct StringWriter(pub *mut u8, pub usize);

impl StringWriter {
    pub unsafe fn write(&mut self, buf: &[u8]) {
        for &b in buf.iter() {
            if self.1 == 0 {
                break;
            } else if self.1 == 1 {
                *self.0 = b'\0';
            } else {
                *self.0 = b;
            }

            self.0 = self.0.offset(1);
            self.1 -= 1;

            if self.1 > 0 {
                *self.0 = b'\0';
            }
        }
    }
}

impl fmt::Write for StringWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe { self.write(s.as_bytes()) };
        Ok(())
    }
}

pub struct UnsafeStringWriter(pub *mut u8);

impl UnsafeStringWriter {
    pub unsafe fn write(&mut self, buf: &[u8]) {
        for &b in buf.iter() {
            *self.0 = b;
            self.0 = self.0.offset(1);
            *self.0 = b'\0';
        }
    }
}

impl fmt::Write for UnsafeStringWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe { self.write(s.as_bytes()) };
        Ok(())
    }
}
