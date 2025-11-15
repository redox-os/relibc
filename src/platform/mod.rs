//! Platform abstractions and environment.

use crate::{
    error::{Errno, ResultExt},
    io::{self, Read, Write},
    raw_cell::RawCell,
};
use alloc::{boxed::Box, vec::Vec};
use core::{cell::Cell, fmt, ptr};

pub use self::allocator::*;

mod allocator;

pub use self::pal::{Pal, PalEpoll, PalPtrace, PalSignal, PalSocket, PalTimer};

mod pal;

pub use self::sys::Sys;

#[cfg(all(not(feature = "no_std"), target_os = "linux"))]
#[path = "linux/mod.rs"]
pub(crate) mod sys;

#[cfg(all(not(feature = "no_std"), target_os = "redox"))]
#[path = "redox/mod.rs"]
pub(crate) mod sys;

#[cfg(test)]
mod test;

pub use self::rlb::{Line, RawLineBuffer};
pub mod rlb;

#[cfg(target_os = "linux")]
pub mod auxv_defs;

#[cfg(target_os = "redox")]
pub use redox_rt::auxv_defs;

use self::types::*;
pub mod types;

/// The global `errno` variable used internally in relibc.
#[thread_local]
pub static ERRNO: Cell<c_int> = Cell::new(0);

/// The `argv` argument available to a program's `main` function.
#[allow(non_upper_case_globals)]
pub static mut argv: *mut *mut c_char = ptr::null_mut();
#[allow(non_upper_case_globals)]
pub static inner_argv: RawCell<Vec<*mut c_char>> = RawCell::new(Vec::new());
#[allow(non_upper_case_globals)]
pub static mut program_invocation_name: *mut c_char = ptr::null_mut();
#[allow(non_upper_case_globals)]
pub static mut program_invocation_short_name: *mut c_char = ptr::null_mut();

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut environ: *mut *mut c_char = ptr::null_mut();

pub static OUR_ENVIRON: RawCell<Vec<*mut c_char>> = RawCell::new(Vec::new());

pub fn environ_iter() -> impl Iterator<Item = *mut c_char> + 'static {
    unsafe {
        let mut ptrs = environ;

        core::iter::from_fn(move || {
            if ptrs.is_null() {
                None
            } else {
                let ptr = ptrs.read();
                if ptr.is_null() {
                    None
                } else {
                    ptrs = ptrs.add(1);
                    Some(ptr)
                }
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

pub struct FileWriter(pub c_int, Option<Errno>);

impl FileWriter {
    pub fn new(fd: c_int) -> Self {
        Self(fd, None)
    }

    pub fn write(&mut self, buf: &[u8]) -> fmt::Result {
        let _ = Sys::write(self.0, buf).map_err(|err| {
            self.1 = Some(err);
            fmt::Error
        })?;
        Ok(())
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
    // TODO: This is a bad interface. Rustify
    pub fn read(&mut self, buf: &mut [u8]) -> isize {
        Sys::read(self.0, buf)
            .map(|u| u as isize)
            .or_minus_one_errno()
    }
}

impl Read for FileReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let i = Sys::read(self.0, buf)
            .map(|u| u as isize)
            .or_minus_one_errno(); // TODO
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
unsafe fn auxv_iter<'a>(ptr: *const usize) -> impl Iterator<Item = [usize; 2]> + 'a {
    struct St(*const usize);
    impl Iterator for St {
        type Item = [usize; 2];

        fn next(&mut self) -> Option<Self::Item> {
            unsafe {
                if self.0.read() == self::auxv_defs::AT_NULL {
                    return None;
                }
                let kind = self.0.read();
                let value = self.0.add(1).read();
                self.0 = self.0.add(2);

                Some([kind, value])
            }
        }
    }
    St(ptr)
}

#[cold]
pub unsafe fn get_auxvs(ptr: *const usize) -> Box<[[usize; 2]]> {
    //traverse the stack and collect argument environment variables
    let mut auxvs = auxv_iter(ptr).collect::<Vec<_>>();

    auxvs.sort_unstable_by_key(|[kind, _]| *kind);
    auxvs.into_boxed_slice()
}
// TODO: Find an auxv replacement for Redox's execv protocol
#[cold]
pub unsafe fn get_auxv_raw(ptr: *const usize, requested_kind: usize) -> Option<usize> {
    auxv_iter(ptr).find_map(|[kind, value]| Some(value).filter(|_| kind == requested_kind))
}
pub fn get_auxv(auxvs: &[[usize; 2]], key: usize) -> Option<usize> {
    auxvs
        .binary_search_by_key(&key, |[entry_key, _]| *entry_key)
        .ok()
        .map(|idx| auxvs[idx][1])
}

#[cold]
#[cfg(target_os = "redox")]
// SAFETY: Must only be called when only one thread exists.
pub unsafe fn init(auxvs: Box<[[usize; 2]]>) {
    use self::auxv_defs::*;
    use crate::header::sys_stat::S_ISVTX;
    use redox_rt::proc::FdGuard;
    use syscall::MODE_PERM;

    let Some(proc_fd) = get_auxv(&auxvs, AT_REDOX_PROC_FD) else {
        panic!("Missing proc and thread fd!");
    };
    redox_rt::initialize(FdGuard::new(proc_fd));

    // TODO: Is it safe to assume setup_sighandler has been called at this point?
    redox_rt::sys::this_proc_call(
        &mut [],
        syscall::CallFlags::empty(),
        &[redox_rt::protocol::ProcCall::SyncSigPctl as u64],
    )
    .expect("failed to sync signal pctl");

    if let (Some(cwd_ptr), Some(cwd_len)) = (
        get_auxv(&auxvs, AT_REDOX_INITIAL_CWD_PTR),
        get_auxv(&auxvs, AT_REDOX_INITIAL_CWD_LEN),
    ) {
        let cwd_bytes: &'static [u8] = core::slice::from_raw_parts(cwd_ptr as *const u8, cwd_len);
        if let Ok(cwd) = core::str::from_utf8(cwd_bytes) {
            self::sys::path::set_cwd_manual(cwd.into());
        }
    }

    if let (Some(scheme_ptr), Some(scheme_len)) = (
        get_auxv(&auxvs, AT_REDOX_INITIAL_DEFAULT_SCHEME_PTR),
        get_auxv(&auxvs, AT_REDOX_INITIAL_DEFAULT_SCHEME_LEN),
    ) {
        let scheme_bytes: &'static [u8] =
            unsafe { core::slice::from_raw_parts(scheme_ptr as *const u8, scheme_len) };
        if let Ok(scheme) = core::str::from_utf8(scheme_bytes) {
            self::sys::path::set_default_scheme_manual(scheme.into());
        }
    }

    let mut inherited_sigignmask = 0_u64;
    if let Some(mask) = get_auxv(&auxvs, AT_REDOX_INHERITED_SIGIGNMASK) {
        inherited_sigignmask |= mask as u64;
    }
    #[cfg(target_pointer_width = "32")]
    if let Some(mask) = get_auxv(&auxvs, AT_REDOX_INHERITED_SIGIGNMASK_HI) {
        inherited_sigignmask |= (mask as u64) << 32;
    }
    redox_rt::signal::apply_inherited_sigignmask(inherited_sigignmask);

    let mut inherited_sigprocmask = 0_u64;

    if let Some(mask) = get_auxv(&auxvs, AT_REDOX_INHERITED_SIGPROCMASK) {
        inherited_sigprocmask |= mask as u64;
    }
    #[cfg(target_pointer_width = "32")]
    if let Some(mask) = get_auxv(&auxvs, AT_REDOX_INHERITED_SIGPROCMASK_HI) {
        inherited_sigprocmask |= (mask as u64) << 32;
    }
    redox_rt::signal::set_sigmask(Some(inherited_sigprocmask), None).unwrap();

    if let Some(umask) = get_auxv(&auxvs, AT_REDOX_UMASK) {
        let _ =
            redox_rt::sys::swap_umask((umask as u32) & u32::from(MODE_PERM) & !(S_ISVTX as u32));
    }
}
#[cfg(not(target_os = "redox"))]
pub unsafe fn init(auxvs: Box<[[usize; 2]]>) {}
