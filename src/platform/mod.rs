use crate::io::{self, Read, Write};
use alloc::{boxed::Box, vec::Vec};
use core::{fmt, ptr};

pub use self::allocator::*;

#[cfg(not(feature = "ralloc"))]
#[path = "allocator/dlmalloc.rs"]
mod allocator;

#[cfg(feature = "ralloc")]
#[path = "allocator/ralloc.rs"]
mod allocator;

pub use self::pal::{Pal, PalEpoll, PalPtrace, PalSignal, PalSocket};

mod pal;

pub use self::sys::{e, Sys};

#[cfg(all(not(feature = "no_std"), target_os = "linux"))]
#[path = "linux/mod.rs"]
pub(crate) mod sys;

#[cfg(all(not(feature = "no_std"), target_os = "redox"))]
#[path = "redox/mod.rs"]
pub(crate) mod sys;

#[cfg(test)]
mod test;

mod pte;

pub use self::rlb::{Line, RawLineBuffer};
pub mod rlb;

#[cfg(target_os = "linux")]
pub mod auxv_defs;

#[cfg(target_os = "redox")]
pub use redox_exec::auxv_defs;

use self::types::*;
pub mod types;

#[thread_local]
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut errno: c_int = 0;

#[allow(non_upper_case_globals)]
pub static mut argv: *mut *mut c_char = ptr::null_mut();
#[allow(non_upper_case_globals)]
pub static mut inner_argv: Vec<*mut c_char> = Vec::new();
#[allow(non_upper_case_globals)]
pub static mut program_invocation_name: *mut c_char = ptr::null_mut();
#[allow(non_upper_case_globals)]
pub static mut program_invocation_short_name: *mut c_char = ptr::null_mut();

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut environ: *mut *mut c_char = ptr::null_mut();

pub static mut OUR_ENVIRON: Vec<*mut c_char> = Vec::new();

pub fn environ_iter() -> impl Iterator<Item = *mut c_char> + 'static {
    unsafe {
        let mut ptrs = environ;

        core::iter::from_fn(move || {
            let ptr = ptrs.read();
            if ptr.is_null() {
                None
            } else {
                ptrs = ptrs.add(1);
                Some(ptr)
            }
        })
    }
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
impl Write for StringWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.1 > 1 {
            let copy_size = buf.len().min(self.1 - 1);
            unsafe {
                ptr::copy_nonoverlapping(buf.as_ptr(), self.0, copy_size);
                self.1 -= copy_size;

                self.0 = self.0.add(copy_size);
                *self.0 = 0;
            }
        }

        // Pretend the entire slice was written. This is because many functions
        // (like snprintf) expects a return value that reflects how many bytes
        // *would have* been written. So keeping track of this information is
        // good, and then if we want the *actual* written size we can just go
        // `cmp::min(written, maxlen)`.
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl fmt::Write for StringWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // can't fail
        self.write(s.as_bytes()).unwrap();
        Ok(())
    }
}
impl WriteByte for StringWriter {
    fn write_u8(&mut self, byte: u8) -> fmt::Result {
        // can't fail
        self.write(&[byte]).unwrap();
        Ok(())
    }
}

pub struct UnsafeStringWriter(pub *mut u8);
impl Write for UnsafeStringWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unsafe {
            ptr::copy_nonoverlapping(buf.as_ptr(), self.0, buf.len());
            self.0 = self.0.add(buf.len());
            *self.0 = b'\0';
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl fmt::Write for UnsafeStringWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // can't fail
        self.write(s.as_bytes()).unwrap();
        Ok(())
    }
}
impl WriteByte for UnsafeStringWriter {
    fn write_u8(&mut self, byte: u8) -> fmt::Result {
        // can't fail
        self.write(&[byte]).unwrap();
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
impl<T: Write> Write for CountingWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let res = self.inner.write(buf);
        if let Ok(written) = res {
            self.written += written;
        }
        res
    }
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match self.inner.write_all(&buf) {
            Ok(()) => (),
            Err(ref err) if err.kind() == io::ErrorKind::WriteZero => (),
            Err(err) => return Err(err),
        }
        self.written += buf.len();
        Ok(())
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

// TODO: Set a global variable once get_auxvs is called, and then implement getauxval based on
// get_auxv.

#[cold]
pub unsafe fn get_auxvs(mut ptr: *const usize) -> Box<[[usize; 2]]> {
    //traverse the stack and collect argument environment variables
    let mut auxvs = Vec::new();

    while *ptr != self::auxv_defs::AT_NULL {
        let kind = ptr.read();
        ptr = ptr.add(1);
        let value = ptr.read();
        ptr = ptr.add(1);
        auxvs.push([kind, value]);
    }

    auxvs.sort_unstable_by_key(|[kind, _]| *kind);
    auxvs.into_boxed_slice()
}
pub fn get_auxv(auxvs: &[[usize; 2]], key: usize) -> Option<usize> {
    auxvs
        .binary_search_by_key(&key, |[entry_key, _]| *entry_key)
        .ok()
        .map(|idx| auxvs[idx][1])
}

#[cold]
#[cfg(target_os = "redox")]
pub fn init(auxvs: Box<[[usize; 2]]>) {
    use self::auxv_defs::*;

    if let (Some(cwd_ptr), Some(cwd_len)) = (
        get_auxv(&auxvs, AT_REDOX_INITIALCWD_PTR),
        get_auxv(&auxvs, AT_REDOX_INITIALCWD_LEN),
    ) {
        let cwd_bytes: &'static [u8] =
            unsafe { core::slice::from_raw_parts(cwd_ptr as *const u8, cwd_len) };
        if let Ok(cwd) = core::str::from_utf8(cwd_bytes) {
            self::sys::path::setcwd_manual(cwd.into());
        }
    }
}
#[cfg(not(target_os = "redox"))]
pub fn init(auxvs: Box<[[usize; 2]]>) {}
