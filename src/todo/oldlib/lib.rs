#![no_std]
#![feature(
    alloc,
    allocator_api,
    alloc_system,
    compiler_builtins_lib,
    const_fn,
    const_ptr_null,
    core_intrinsics,
    global_allocator,
    lang_items,
    linkage,
    const_size_of,
    const_cell_new,
)]

#[macro_use]
extern crate alloc;
extern crate alloc_system;
extern crate byteorder;
extern crate compiler_builtins;
extern crate libc;
extern crate syscall;
extern crate redox_termios;

use alloc::Vec;
use alloc::String;
use alloc::fmt::Write;
use alloc::boxed::Box;
use core::{ptr, mem, intrinsics, slice, str};
use libc::{c_int, c_void, c_char, size_t};

#[macro_use]
mod macros;
mod types;
mod dns;
mod mallocnull;
mod rawfile;
pub mod event;
pub mod process;
pub mod file;
pub mod folder;
pub mod time;
pub mod unimpl;
pub mod user;
pub mod redox;
pub mod socket;
pub mod hostname;
pub mod termios;
pub mod netdb;

pub use mallocnull::MallocNull;
pub use rawfile::RawFile;

#[global_allocator]
static ALLOCATOR: alloc_system::System = alloc_system::System;

extern "C" {
    // Newlib uses this function instead of just a global to support reentrancy
    pub fn __errno() -> *mut c_int;
    pub fn malloc(size: size_t) -> *mut c_void;
    pub fn free(ptr: *mut c_void);
    pub fn strlen(s: *const c_char) -> size_t;
    pub fn __libc_fini_array();
    pub static mut environ: *mut *mut c_char;
}

pub unsafe fn cstr_to_slice<'a>(buf: *const c_char) -> &'a [u8] {
    slice::from_raw_parts(buf as *const u8, ::strlen(buf) as usize)
}
pub unsafe fn cstr_to_slice_mut<'a>(buf: *const c_char) -> &'a mut [u8] {
    slice::from_raw_parts_mut(buf as *mut u8, ::strlen(buf) as usize)
}

pub fn file_read_all<T: AsRef<[u8]>>(path: T) -> syscall::Result<Vec<u8>> {
    let fd = RawFile::open(path, syscall::O_RDONLY)?;

    let mut st = syscall::Stat::default();
    syscall::fstat(*fd, &mut st)?;
    let size = st.st_size as usize;

    let mut buf = Vec::with_capacity(size);
    unsafe { buf.set_len(size) };
    syscall::read(*fd, buf.as_mut_slice())?;

    Ok(buf)
}

/// Implements an `Iterator` which returns on either newline or EOF.
#[derive(Clone, Copy)]
pub struct RawLineBuffer {
    fd: usize,
    cur: usize,
    read: usize,
    buf: [u8; 8 * 1024],
}

impl Default for RawLineBuffer {
    fn default() -> RawLineBuffer {
        RawLineBuffer {
            fd: 0,
            cur: 0,
            read: 0,
            buf: unsafe { mem::uninitialized() },
        }
    }
}

impl RawLineBuffer {
    fn new(fd: usize) -> RawLineBuffer {
        RawLineBuffer {
            fd: fd,
            cur: 0,
            read: 0,
            buf: unsafe { mem::uninitialized() },
        }
    }
}

impl Iterator for RawLineBuffer {
    type Item = syscall::Result<Box<str>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur != 0 && self.read != 0 {
            if let Some(mut pos) = self.buf[self.cur..self.read].iter().position(
                |&x| x == b'\n',
            )
            {
                pos += self.cur + 1;
                let line = unsafe { str::from_utf8_unchecked(&self.buf[self.cur..pos]) };
                let boxed_array: Box<[u8]> = Box::from(line.as_bytes());
                let boxed_line: Box<str> = unsafe { mem::transmute(boxed_array) };
                self.cur = pos;
                return Some(Ok(boxed_line));
            }

            let mut temp: [u8; 8 * 1024] = unsafe { mem::uninitialized() };
            unsafe {
                ptr::copy(self.buf[self.cur..].as_ptr(), temp.as_mut_ptr(), self.read);
                ptr::swap(self.buf.as_mut_ptr(), temp.as_mut_ptr());
            };

            self.read -= self.cur;
            self.cur = 0;
        }

        let end = match syscall::read(self.fd, &mut self.buf[self.cur..]).unwrap() {
            0 => return None,
            read => {
                self.read += read;
                self.buf[..self.read]
                    .iter()
                    .position(|&x| x == b'\n')
                    .unwrap_or(0)
            }
        };

        self.cur = end;
        let line = unsafe { str::from_utf8_unchecked(&self.buf[..end]) };
        let boxed_array: Box<[u8]> = Box::from(line.as_bytes());
        let boxed_line: Box<str> = unsafe { mem::transmute(boxed_array) };

        Some(Ok(boxed_line))
    }
}

#[no_mangle]
pub unsafe extern "C" fn __errno_location() -> *mut c_int {
    __errno()
}

#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}

#[lang = "panic_fmt"]
#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn rust_begin_unwind(_msg: core::fmt::Arguments,
                               _file: &'static str,
                               _line: u32,
                               _col: u32) -> ! {
   let mut s = String::new();
    let _ = s.write_fmt(_msg);
    let _ = syscall::write(2, _file.as_bytes());
    let _ = syscall::write(2, "\n".as_bytes());
    let _ = syscall::write(2, s.as_bytes());
    let _ = syscall::write(2, "\n".as_bytes());
    unsafe { intrinsics::abort() }
}

libc_fn!(unsafe initialize_standard_library() {
    let mut buf = file_read_all("env:").unwrap();
    let size = buf.len();
    buf.push(0);

    let mut vars = Vec::new();

    vars.push(&mut buf[0] as *mut u8 as *mut c_char);
    for i in 0..size {
        if buf[i] == b'\n' {
            if i != size - 1 {
                vars.push(&mut buf[i + 1] as *mut u8 as *mut c_char);
            }
            buf[i] = b'\0';
        }
    }
    vars.push(ptr::null_mut());

    environ = vars.as_mut_ptr();
    mem::forget(vars); // Do not free memory
    mem::forget(buf);
});
