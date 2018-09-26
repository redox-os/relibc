use alloc::vec::Vec;
use core::{fmt, ptr};
use io::{self, Read};

pub use self::allocator::*;

#[cfg(not(feature = "ralloc"))]
#[path = "allocator/dlmalloc.rs"]
mod allocator;

#[cfg(feature = "ralloc")]
#[path = "allocator/ralloc.rs"]
mod allocator;

pub use self::pal::{Pal, PalSignal, PalSocket};

mod pal;

pub use self::sys::Sys;

#[cfg(all(not(feature = "no_std"), target_os = "linux"))]
#[path = "linux/mod.rs"]
mod sys;

#[cfg(all(not(feature = "no_std"), target_os = "redox"))]
#[path = "redox/mod.rs"]
mod sys;

pub use self::rlb::{Line, RawLineBuffer};
pub mod rlb;

use self::types::*;
pub mod types;

//TODO #[thread_local]
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut errno: c_int = 0;

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut environ: *mut *mut c_char = ptr::null_mut();
#[allow(non_upper_case_globals)]
pub static mut inner_environ: Vec<*mut c_char> = Vec::new();

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

pub trait WriteByte: fmt::Write {
    fn write_u8(&mut self, byte: u8) -> fmt::Result;
}

impl<'a, W: WriteByte> WriteByte for &'a mut W {
    fn write_u8(&mut self, byte: u8) -> fmt::Result {
        (**self).write_u8(byte)
    }
}

pub struct FileWriter(pub c_int);

impl FileWriter {
    pub fn write(&mut self, buf: &[u8]) -> isize {
        Sys::write(self.0, buf)
    }
}

impl fmt::Write for FileWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl WriteByte for FileWriter {
    fn write_u8(&mut self, byte: u8) -> fmt::Result {
        self.write(&[byte]);
        Ok(())
    }
}

pub struct FileReader(pub c_int);

impl FileReader {
    pub fn read(&mut self, buf: &mut [u8]) -> isize {
        Sys::read(self.0, buf)
    }
}

impl Read for FileReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let i = Sys::read(self.0, buf);
        if i >= 0 {
            Ok(i as usize)
        } else {
            Err(io::Error::from_raw_os_error(-i as i32))
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

impl WriteByte for StringWriter {
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

impl WriteByte for UnsafeStringWriter {
    fn write_u8(&mut self, byte: u8) -> fmt::Result {
        unsafe { self.write(&[byte]) };
        Ok(())
    }
}

pub struct UnsafeStringReader(pub *const u8);

impl Read for UnsafeStringReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe {
            for i in 0..buf.len() {
                if *self.0 == 0 {
                    return Ok(i);
                }

                buf[i] = *self.0;
                self.0 = self.0.offset(1);
            }
            Ok(buf.len())
        }
    }
}

pub struct CountingWriter<T> {
    pub inner: T,
    pub written: usize,
}
impl<T> CountingWriter<T> {
    pub fn new(writer: T) -> Self {
        Self {
            inner: writer,
            written: 0,
        }
    }
}
impl<T: fmt::Write> fmt::Write for CountingWriter<T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.written += s.len();
        self.inner.write_str(s)
    }
}
impl<T: WriteByte> WriteByte for CountingWriter<T> {
    fn write_u8(&mut self, byte: u8) -> fmt::Result {
        self.written += 1;
        self.inner.write_u8(byte)
    }
}
