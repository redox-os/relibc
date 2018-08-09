#![no_std]
#![allow(non_camel_case_types)]
#![feature(alloc, allocator_api, const_vec_new)]
#![cfg_attr(target_os = "redox", feature(thread_local))]

#[cfg_attr(target_os = "redox", macro_use)]
extern crate alloc;

#[cfg(target_os = "linux")]
#[macro_use]
extern crate sc;

#[cfg(target_os = "redox")]
extern crate syscall;

#[cfg(target_os = "redox")]
extern crate spin;

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

pub mod rawfile;
pub mod types;

pub use rawfile::RawFile;

use alloc::vec::Vec;
use core::{fmt, ptr};

use types::*;

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

//TODO #[thread_local]
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut errno: c_int = 0;

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut environ: *mut *mut c_char = ptr::null_mut();
#[allow(non_upper_case_globals)]
pub static mut inner_environ: Vec<*mut c_char> = Vec::new();

pub unsafe fn c_str_mut<'a>(s: *mut c_char) -> &'a mut [u8] {
    use core::usize;

    c_str_n_mut(s, usize::MAX)
}

pub unsafe fn c_str_n_mut<'a>(s: *mut c_char, n: usize) -> &'a mut [u8] {
    assert!(s != ptr::null_mut());
    use core::slice;

    let mut size = 0;

    for _ in 0..n {
        if *s.offset(size) == 0 {
            break;
        }
        size += 1;
    }

    slice::from_raw_parts_mut(s as *mut u8, size as usize)
}
pub unsafe fn c_str<'a>(s: *const c_char) -> &'a [u8] {
    use core::usize;

    c_str_n(s, usize::MAX)
}

pub unsafe fn c_str_n<'a>(s: *const c_char, n: usize) -> &'a [u8] {
    assert!(s != ptr::null());
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
    bytes.as_ptr() as *const c_char
}

// NOTE: defined here rather than in string because memcpy() is useful in multiple crates
pub unsafe fn memcpy(s1: *mut c_void, s2: *const c_void, n: usize) -> *mut c_void {
    let mut i = 0;
    while i + 7 < n {
        *(s1.offset(i as isize) as *mut u64) = *(s2.offset(i as isize) as *const u64);
        i += 8;
    }
    while i < n {
        *(s1 as *mut u8).offset(i as isize) = *(s2 as *const u8).offset(i as isize);
        i += 1;
    }
    s1
}

pub trait Write: fmt::Write {
    fn write_u8(&mut self, byte: u8) -> fmt::Result;
}

impl<'a, W: Write> Write for &'a mut W {
    fn write_u8(&mut self, byte: u8) -> fmt::Result {
        (**self).write_u8(byte)
    }
}

pub trait Read {
    fn read_u8(&mut self) -> Result<Option<u8>, ()>;
}

impl<'a, R: Read> Read for &'a mut R {
    fn read_u8(&mut self) -> Result<Option<u8>, ()> {
        (**self).read_u8()
    }
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

impl Write for FileWriter {
    fn write_u8(&mut self, byte: u8) -> fmt::Result {
        self.write(&[byte]);
        Ok(())
    }
}

pub struct FileReader(pub c_int);

impl FileReader {
    pub fn read(&mut self, buf: &mut [u8]) -> isize {
        read(self.0, buf)
    }
}

impl Read for FileReader {
    fn read_u8(&mut self) -> Result<Option<u8>, ()> {
        let mut buf = [0];
        match self.read(&mut buf) {
            0 => Ok(None),
            n if n < 0 => Err(()),
            _ => Ok(Some(buf[0]))
        }
    }
}

pub struct StringWriter(pub *mut u8, pub usize);

impl StringWriter {
    pub unsafe fn write(&mut self, buf: &[u8]) {
        if self.1 > 1 {
            let copy_size = buf.len().min(self.1 - 1);
            memcpy(
                self.0 as *mut c_void,
                buf.as_ptr() as *const c_void,
                copy_size,
            );
            self.1 -= copy_size;

            self.0 = self.0.offset(copy_size as isize);
            *self.0 = 0;
        }
    }
}

impl fmt::Write for StringWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe { self.write(s.as_bytes()) };
        Ok(())
    }
}

impl Write for StringWriter {
    fn write_u8(&mut self, byte: u8) -> fmt::Result {
        unsafe { self.write(&[byte]) };
        Ok(())
    }
}

pub struct UnsafeStringWriter(pub *mut u8);

impl UnsafeStringWriter {
    pub unsafe fn write(&mut self, buf: &[u8]) {
        memcpy(
            self.0 as *mut c_void,
            buf.as_ptr() as *const c_void,
            buf.len(),
        );
        *self.0.offset(buf.len() as isize) = b'\0';
        self.0 = self.0.offset(buf.len() as isize);
    }
}

impl fmt::Write for UnsafeStringWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe { self.write(s.as_bytes()) };
        Ok(())
    }
}

impl Write for UnsafeStringWriter {
    fn write_u8(&mut self, byte: u8) -> fmt::Result {
        unsafe { self.write(&[byte]) };
        Ok(())
    }
}

pub struct StringReader<'a>(pub &'a [u8]);

impl<'a> Read for StringReader<'a> {
    fn read_u8(&mut self) -> Result<Option<u8>, ()> {
        if self.0.is_empty() {
            Ok(None)
        } else {
            let byte = self.0[0];
            self.0 = &self.0[1..];
            Ok(Some(byte))
        }
    }
}

pub struct UnsafeStringReader(pub *const u8);

impl Read for UnsafeStringReader {
    fn read_u8(&mut self) -> Result<Option<u8>, ()> {
        unsafe {
            if *self.0 == 0 {
                Ok(None)
            } else {
                let byte = *self.0;
                self.0 = self.0.offset(1);
                Ok(Some(byte))
            }
        }
    }
}

pub struct CountingWriter<T> {
    pub inner: T,
    pub written: usize
}
impl<T> CountingWriter<T> {
    pub /* const */ fn new(writer: T) -> Self {
        Self {
            inner: writer,
            written: 0
        }
    }
}
impl<T: fmt::Write> fmt::Write for CountingWriter<T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.written += s.len();
        self.inner.write_str(s)
    }
}
impl<T: Write> Write for CountingWriter<T> {
    fn write_u8(&mut self, byte: u8) -> fmt::Result {
        self.written += 1;
        self.inner.write_u8(byte)
    }
}
