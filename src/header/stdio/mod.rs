//! `stdio.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stdio.h.html>.

use alloc::{
    borrow::{Borrow, BorrowMut},
    boxed::Box,
    vec::Vec,
};
use core::{
    cmp,
    ffi::VaList as va_list,
    fmt::{self, Write as WriteFmt},
    i32, mem,
    ops::{Deref, DerefMut},
    ptr, slice, str,
};

use crate::{
    c_str::CStr,
    c_vec::CVec,
    error::{ResultExt, ResultExtPtrMut},
    fs::File,
    header::{
        errno::{self, STR_ERROR},
        fcntl,
        pthread::RlctMutex,
        pwd, stdlib,
        string::{self, strlen, strncpy},
        unistd,
    },
    io::{self, BufRead, BufWriter, LineWriter, Read, Write},
    out::Out,
    platform::{
        self, ERRNO, Pal, Sys, WriteByte,
        types::{c_char, c_int, c_long, c_uint, c_ulonglong, c_void, off_t, size_t},
    },
    sync::Mutex,
};

pub use self::constants::*;
mod constants;

pub use self::default::*;
mod default;

pub use self::getdelim::*;
mod getdelim;

mod ext;
mod helpers;
mod lookaheadreader;
pub mod printf;
mod scanf;
use lookaheadreader::LookAheadReader;
static mut TMPNAM_BUF: [c_char; L_tmpnam as usize + 1] = [0; L_tmpnam as usize + 1];

enum Buffer<'a> {
    Borrowed(&'a mut [u8]),
    Owned(Vec<u8>),
}

impl<'a> Deref for Buffer<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Buffer::Borrowed(inner) => inner,
            Buffer::Owned(inner) => inner.borrow(),
        }
    }
}

impl<'a> DerefMut for Buffer<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Buffer::Borrowed(inner) => inner,
            Buffer::Owned(inner) => inner.borrow_mut(),
        }
    }
}

pub trait Pending {
    fn pending(&self) -> size_t;
}

impl<W: crate::io::Write> Pending for BufWriter<W> {
    fn pending(&self) -> size_t {
        self.buf.len() as size_t
    }
}

impl<W: crate::io::Write> Pending for LineWriter<W> {
    fn pending(&self) -> size_t {
        self.inner.buf.len() as size_t
    }
}

pub trait Writer: Write + Pending {
    fn purge(&mut self);
}

impl<W: crate::io::Write> Writer for BufWriter<W> {
    fn purge(&mut self) {
        self.buf.clear();
    }
}

impl<W: crate::io::Write> Writer for LineWriter<W> {
    fn purge(&mut self) {
        self.inner.buf.clear();
    }
}

/// This struct gets exposed to the C API.
pub struct FILE {
    lock: RlctMutex,

    file: File,
    // pub for stdio_ext
    pub(crate) flags: c_int,

    // TODO: Is the read_buf dropped?
    read_buf: Buffer<'static>,

    read_pos: usize,
    read_size: usize,
    unget: Vec<u8>,
    // pub for stdio_ext

    // TODO: To support const fn initialization, use static dispatch (perhaps partially)?
    pub(crate) writer: Box<dyn Writer + Send>,

    // Optional pid for use with popen/pclose
    pid: Option<c_int>,

    // wchar support
    pub(crate) orientation: c_int,
}

impl Read for FILE {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let unget_read_size = cmp::min(out.len(), self.unget.len());
        for i in 0..unget_read_size {
            out[i] = self.unget.pop().unwrap();
        }
        if unget_read_size != 0 {
            return Ok(unget_read_size);
        }

        let len = {
            let buf = self.fill_buf()?;
            let len = buf.len().min(out.len());

            out[..len].copy_from_slice(&buf[..len]);
            len
        };
        self.consume(len);
        Ok(len)
    }
}

impl BufRead for FILE {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.read_pos == self.read_size {
            self.read_size = match self.file.read(&mut self.read_buf) {
                Ok(0) => {
                    self.flags |= F_EOF;
                    0
                }
                Ok(n) => n,
                Err(err) => {
                    self.flags |= F_ERR;
                    return Err(err);
                }
            };
            self.read_pos = 0;
        }
        Ok(&self.read_buf[self.read_pos..self.read_size])
    }
    fn consume(&mut self, i: usize) {
        self.read_pos = (self.read_pos + i).min(self.read_size);
    }
}

impl Write for FILE {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.writer.write(buf) {
            Ok(n) => Ok(n),
            Err(err) => {
                self.flags |= F_ERR;
                Err(err)
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match self.writer.flush() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.flags |= F_ERR;
                Err(err)
            }
        }
    }
}

impl WriteFmt for FILE {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_all(s.as_bytes())
            .map(|_| ())
            .map_err(|_| fmt::Error)
    }
}

impl WriteByte for FILE {
    fn write_u8(&mut self, c: u8) -> fmt::Result {
        self.write_all(&[c]).map(|_| ()).map_err(|_| fmt::Error)
    }
}

impl FILE {
    pub fn lock(&mut self) -> LockGuard {
        unsafe {
            flockfile(self);
        }
        LockGuard(self)
    }

    pub fn try_set_orientation(&mut self, mode: c_int) -> c_int {
        let stream = self.lock();
        stream.0.try_set_orientation_unlocked(mode)
    }

    pub fn try_set_orientation_unlocked(&mut self, mode: c_int) -> c_int {
        if self.orientation == 0 {
            self.orientation = match mode {
                1..=i32::MAX => 1,
                i32::MIN..=-1 => -1,
                0 => self.orientation,
            };
        }
        self.orientation
    }

    pub fn try_set_byte_orientation_unlocked(&mut self) -> core::result::Result<(), c_int> {
        match self.try_set_orientation_unlocked(-1) {
            i32::MIN..=-1 => Ok(()),
            x => Err(x),
        }
    }

    pub fn try_set_wide_orientation_unlocked(&mut self) -> core::result::Result<(), c_int> {
        match self.try_set_orientation_unlocked(1) {
            1..=i32::MAX => Ok(()),
            x => Err(x),
        }
    }

    pub fn purge(&mut self) {
        // Purge read buffer
        self.read_pos = 0;
        self.read_size = 0;
        // Purge unget
        self.unget.clear();
        // Purge write buffer
        self.writer.purge();
    }
}

pub struct LockGuard<'a>(&'a mut FILE);

impl<'a> Deref for LockGuard<'a> {
    type Target = FILE;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for LockGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<'a> Drop for LockGuard<'a> {
    fn drop(&mut self) {
        unsafe {
            funlockfile(self.0);
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/clearerr.html>.
///
/// Clears EOF and ERR indicators on a stream
#[unsafe(no_mangle)]
pub unsafe extern "C" fn clearerr(stream: *mut FILE) {
    let mut stream = unsafe { (*stream).lock() };
    stream.flags &= !(F_EOF | F_ERR);
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ctermid.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ctermid(s: *mut c_char) -> *mut c_char {
    static mut TERMID: [u8; L_ctermid] = *b"/dev/tty\0";

    if s.is_null() {
        return &raw mut TERMID as *mut c_char;
    }

    unsafe { strncpy(s, &raw mut TERMID as *mut c_char, L_ctermid) }
}

/// See <https://pubs.opengroup.org/onlinepubs/7908799/xsh/cuserid.html>
///
/// Marked legacy in SUS Version 2.
// #[unsafe(no_mangle)]
pub unsafe extern "C" fn cuserid(s: *mut c_char) -> *mut c_char {
    let mut buf: Vec<c_char> = vec![0; 256];
    let mut pwd: pwd::passwd = unsafe { mem::zeroed() };
    let mut pwdbuf: *mut pwd::passwd = unsafe { mem::zeroed() };
    if s != ptr::null_mut() {
        unsafe {
            *s.add(0) = 0;
        }
    }
    unsafe {
        pwd::getpwuid_r(
            unistd::geteuid(),
            &mut pwd,
            buf.as_mut_ptr(),
            buf.len(),
            &mut pwdbuf,
        )
    };
    if pwdbuf == ptr::null_mut() {
        return s;
    }

    if s != ptr::null_mut() {
        unsafe { strncpy(s, (*pwdbuf).pw_name, unistd::L_cuserid) };
        return s;
    }

    unsafe { (*pwdbuf).pw_name }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fclose.html>.
///
/// Close a file
/// This function does not guarentee that the file buffer will be flushed or that the file
/// descriptor will be closed, so if it is important that the file be written to, use `fflush()`
/// prior to using this function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fclose(stream: *mut FILE) -> c_int {
    let stream = unsafe { &mut *stream };
    unsafe { flockfile(stream) };

    let mut r = stream.flush().is_err();
    // TODO: better error handling
    let close = Sys::close(*stream.file).map(|()| 0).or_minus_one_errno() == -1;
    r = r || close;

    if stream.flags & constants::F_PERM == 0 {
        // Not one of stdin, stdout or stderr
        let mut stream = unsafe { Box::from_raw(stream) };
        // Reference files aren't closed on drop, so pretend to be a reference
        stream.file.reference = true;
    } else {
        unsafe { funlockfile(stream) };
    }

    r as c_int
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fdopen.html>.
///
/// Open a file from a file descriptor
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fdopen(fildes: c_int, mode: *const c_char) -> *mut FILE {
    helpers::_fdopen(fildes, unsafe { CStr::from_ptr(mode) })
        .map(|f| Box::into_raw(f))
        .or_errno_null_mut()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/feof.html>.
///
/// Check for EOF
#[unsafe(no_mangle)]
pub unsafe extern "C" fn feof(stream: *mut FILE) -> c_int {
    let stream = unsafe { (*stream).lock() };
    stream.flags & F_EOF
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ferror.html>.
///
/// Check for ERR
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ferror(stream: *mut FILE) -> c_int {
    let stream = unsafe { (*stream).lock() };
    stream.flags & F_ERR
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fflush.html>.
///
/// Flush output to stream, or sync read position
/// Ensure the file is unlocked before calling this function, as it will attempt to lock the file
/// itself.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fflush(stream: *mut FILE) -> c_int {
    if stream.is_null() {
        //TODO: flush all files!

        if unsafe { fflush(stdout) } != 0 {
            return EOF;
        }

        if unsafe { fflush(stderr) } != 0 {
            return EOF;
        }
    } else {
        let mut stream = unsafe { unsafe { (*stream).lock() } };
        if stream.flush().is_err() {
            return EOF;
        }
    }

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fgetc.html>.
///
/// Get a single char from a stream
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fgetc(stream: *mut FILE) -> c_int {
    let mut stream = unsafe { (*stream).lock() };
    if let Err(_) = (*stream).try_set_byte_orientation_unlocked() {
        return -1;
    }

    unsafe { getc_unlocked(&mut *stream) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fgetpos.html>.
///
/// Get the position of the stream and store it in pos
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fgetpos(stream: *mut FILE, pos: *mut fpos_t) -> c_int {
    let off = unsafe { ftello(stream) };
    if off < 0 {
        return -1;
    }
    unsafe { *pos = off };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fgets.html>.
///
/// Get a string from the stream
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fgets(
    original: *mut c_char,
    max: c_int,
    stream: *mut FILE,
) -> *mut c_char {
    let mut stream = unsafe { (*stream).lock() };
    if let Err(_) = (*stream).try_set_byte_orientation_unlocked() {
        return ptr::null_mut();
    }

    let mut out = original;
    let max = max as usize;
    let mut left = max.saturating_sub(1); // Make space for the terminating NUL-byte
    let mut wrote = false;

    if left >= 1 {
        let unget_read_size = cmp::min(left, stream.unget.len());
        for _ in 0..unget_read_size {
            unsafe { *out = stream.unget.pop().unwrap() as c_char };
            out = unsafe { out.offset(1) };
        }
        left -= unget_read_size;
    }

    loop {
        if left == 0 {
            break;
        }

        // TODO: When NLL is a thing, this block can be flattened out
        let (read, exit) = {
            let buf = match stream.fill_buf() {
                Ok(buf) => buf,
                Err(_) => return ptr::null_mut(),
            };
            if buf.is_empty() {
                break;
            }
            wrote = true;
            let len = buf.len().min(left);

            let newline = buf[..len].iter().position(|&c| c == b'\n');
            let len = newline.map(|i| i + 1).unwrap_or(len);

            unsafe { ptr::copy_nonoverlapping(buf.as_ptr(), out as *mut u8, len) };

            (len, newline.is_some())
        };

        stream.consume(read);

        out = unsafe { out.add(read) };
        left -= read;

        if exit {
            break;
        }
    }

    if max >= 1 {
        // Write the NUL byte
        unsafe { *out = 0 };
    }
    if wrote { original } else { ptr::null_mut() }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fileno.html>.
///
/// Get the underlying file descriptor
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fileno(stream: *mut FILE) -> c_int {
    let stream = unsafe { (*stream).lock() };
    *stream.file
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/flockfile.html>.
///
/// Lock the file
/// Do not call any functions other than those with the `_unlocked` postfix while the file is
/// locked
#[unsafe(no_mangle)]
pub unsafe extern "C" fn flockfile(file: *mut FILE) {
    if let Err(e) = unsafe { (*file).lock.lock() } {
        println!("RELIBC: flockfile error {}", e)
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fopen.html>.
///
/// Open the file in mode `mode`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fopen(filename: *const c_char, mode: *const c_char) -> *mut FILE {
    let initial_mode = unsafe { *mode };
    if initial_mode != b'r' as c_char
        && initial_mode != b'w' as c_char
        && initial_mode != b'a' as c_char
    {
        platform::ERRNO.set(errno::EINVAL);
        return ptr::null_mut();
    }

    let flags = helpers::parse_mode_flags(unsafe { CStr::from_ptr(mode) });

    let new_mode = if flags & fcntl::O_CREAT == fcntl::O_CREAT {
        0o666
    } else {
        0
    };

    let fd = unsafe { fcntl::open(filename, flags, new_mode) };
    if fd < 0 {
        return ptr::null_mut();
    }

    if flags & fcntl::O_CLOEXEC > 0 {
        unsafe { fcntl::fcntl(fd, fcntl::F_SETFD, fcntl::FD_CLOEXEC as c_ulonglong) };
    }

    helpers::_fdopen(fd, unsafe { CStr::from_ptr(mode) })
        .map(|f| Box::into_raw(f))
        .map_err(|err| {
            // TODO: guard type
            Sys::close(fd);
            err
        })
        .or_errno_null_mut()
}

/// See <https://www.man7.org/linux/man-pages/man3/fpurge.3.html>.
///
/// Non-POSIX. From Solaris.
///
/// Clear the buffers of a stream
/// Ensure the file is unlocked before calling this function, as it will attempt to lock the file
/// itself.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __fpurge(stream: *mut FILE) {
    if !stream.is_null() {
        let mut stream = unsafe { (*stream).lock() };
        stream.purge();
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fputc.html>.
///
/// Insert a character into the stream
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fputc(c: c_int, stream: *mut FILE) -> c_int {
    let mut stream = unsafe { (*stream).lock() };
    if let Err(_) = (*stream).try_set_byte_orientation_unlocked() {
        return -1;
    }

    unsafe { putc_unlocked(c, &mut *stream) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fputs.html>.
///
/// Insert a string into a stream
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fputs(s: *const c_char, stream: *mut FILE) -> c_int {
    let mut stream = unsafe { (*stream).lock() };
    if let Err(_) = (*stream).try_set_byte_orientation_unlocked() {
        return -1;
    }

    let buf = unsafe { slice::from_raw_parts(s as *mut u8, strlen(s)) };

    if stream.write_all(&buf).is_ok() {
        0
    } else {
        -1
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fread.html>.
///
/// Read `nitems` of size `size` into `ptr` from `stream`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fread(
    ptr: *mut c_void,
    size: size_t,
    nitems: size_t,
    stream: *mut FILE,
) -> size_t {
    if size == 0 || nitems == 0 {
        return 0;
    }

    let mut stream = unsafe { (*stream).lock() };
    if let Err(_) = (*stream).try_set_byte_orientation_unlocked() {
        return 0;
    }

    let buf = unsafe { slice::from_raw_parts_mut(ptr as *mut u8, size as usize * nitems as usize) };
    let mut read = 0;
    while read < buf.len() {
        match stream.read(&mut buf[read..]) {
            Ok(0) | Err(_) => break,
            Ok(n) => read += n,
        }
    }
    (read / size as usize) as size_t
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/freopen.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn freopen(
    filename: *const c_char,
    mode: *const c_char,
    stream: &mut FILE,
) -> *mut FILE {
    let mut flags = helpers::parse_mode_flags(unsafe { CStr::from_ptr(mode) });
    unsafe { flockfile(stream) };

    let _ = stream.flush();
    if filename.is_null() {
        // Reopen stream in new mode
        if flags & fcntl::O_CLOEXEC > 0 {
            unsafe {
                fcntl::fcntl(
                    *stream.file,
                    fcntl::F_SETFD,
                    fcntl::FD_CLOEXEC as c_ulonglong,
                )
            };
        }
        flags &= !(fcntl::O_CREAT | fcntl::O_EXCL | fcntl::O_CLOEXEC);
        if unsafe { fcntl::fcntl(*stream.file, fcntl::F_SETFL, flags as c_ulonglong) } < 0 {
            unsafe { funlockfile(stream) };
            unsafe { fclose(stream) };
            return ptr::null_mut();
        }
    } else {
        let new = unsafe { fopen(filename, mode) };
        if new.is_null() {
            unsafe { funlockfile(stream) };
            unsafe { fclose(stream) };
            return ptr::null_mut();
        }
        let new = unsafe { &mut *new }; // Should be safe, new is not null
        if *new.file == *stream.file {
            new.file.fd = -1;
        } else if Sys::dup2(*new.file, *stream.file).or_minus_one_errno() == -1
            || unsafe {
                fcntl::fcntl(
                    *stream.file,
                    fcntl::F_SETFL,
                    (flags & fcntl::O_CLOEXEC) as c_ulonglong,
                )
            } < 0
        {
            unsafe { funlockfile(stream) };
            unsafe { fclose(new) };
            unsafe { fclose(stream) };
            return ptr::null_mut();
        }
        stream.flags = (stream.flags & constants::F_PERM) | new.flags;
        unsafe { fclose(new) };
    }
    stream.orientation = 0;
    unsafe { funlockfile(stream) };
    stream
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fseek.html>.
///
/// Seek to an offset `offset` from `whence`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fseek(stream: *mut FILE, offset: c_long, whence: c_int) -> c_int {
    unsafe { fseeko(stream, offset as off_t, whence) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fseek.html>.
///
/// Seek to an offset `offset` from `whence`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fseeko(stream: *mut FILE, off: off_t, whence: c_int) -> c_int {
    let mut stream = unsafe { (*stream).lock() };
    unsafe { fseek_locked(&mut *stream, off, whence) }
}

pub unsafe fn fseek_locked(stream: &mut FILE, mut off: off_t, whence: c_int) -> c_int {
    if whence == SEEK_CUR {
        // Since it's a buffered writer, our actual cursor isn't where the user
        // thinks
        off -= (stream.read_size - stream.read_pos) as off_t;
    }

    // Flush write buffer before seek
    if stream.flush().is_err() {
        return -1;
    }

    let err = Sys::lseek(*stream.file, off, whence).or_minus_one_errno();
    if err < 0 {
        return err as c_int;
    }

    stream.flags &= !(F_EOF | F_ERR);
    stream.read_pos = 0;
    stream.read_size = 0;
    stream.unget = Vec::new();
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fsetpos.html>.
///
/// Seek to a position `pos` in the file from the beginning of the file
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fsetpos(stream: *mut FILE, pos: *const fpos_t) -> c_int {
    unsafe { fseeko(stream, *pos, SEEK_SET) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ftell.html>.
///
/// Get the current position of the cursor in the file
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ftell(stream: *mut FILE) -> c_long {
    unsafe { ftello(stream) as c_long }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ftell.html>.
///
/// Get the current position of the cursor in the file
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ftello(stream: *mut FILE) -> off_t {
    let mut stream = unsafe { (*stream).lock() };
    unsafe { ftell_locked(&mut *stream) }
}

pub unsafe extern "C" fn ftell_locked(stream: &mut FILE) -> off_t {
    let pos = Sys::lseek(*stream.file, 0, SEEK_CUR).or_minus_one_errno();
    if pos < 0 {
        return -1;
    }

    // Adjust for read buffer, ungetc, and write buffer
    pos - (stream.read_size - stream.read_pos) as off_t - stream.unget.len() as off_t
        + stream.writer.pending() as off_t
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/flockfile.html>.
///
/// Try to lock the file. Returns 0 for success, 1 for failure
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ftrylockfile(file: *mut FILE) -> c_int {
    if unsafe { (*file).lock.try_lock() }.is_ok() {
        0
    } else {
        1
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/flockfile.html>.
///
/// Unlock the file
#[unsafe(no_mangle)]
pub unsafe extern "C" fn funlockfile(file: *mut FILE) {
    if let Err(e) = unsafe { (*file).lock.unlock() } {
        println!("RELIBC: funlockfile error {}", e)
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fwrite.html>.
///
/// Write `nitems` of size `size` from `ptr` to `stream`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fwrite(
    ptr: *const c_void,
    size: size_t,
    nitems: size_t,
    stream: *mut FILE,
) -> size_t {
    if size == 0 || nitems == 0 {
        return 0;
    }
    let mut stream = unsafe { (*stream).lock() };
    if let Err(_) = (*stream).try_set_byte_orientation_unlocked() {
        return 0;
    }

    let buf = unsafe { slice::from_raw_parts(ptr as *const u8, size as usize * nitems as usize) };
    let mut written = 0;
    while written < buf.len() {
        match stream.write(&buf[written..]) {
            Ok(0) | Err(_) => break,
            Ok(n) => written += n,
        }
    }
    (written / size as usize) as size_t
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getc.html>.
///
/// Get a single char from a stream
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getc(stream: *mut FILE) -> c_int {
    let mut stream = unsafe { (*stream).lock() };
    unsafe { getc_unlocked(&mut *stream) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getchar.html>.
///
/// Get a single char from `stdin`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getchar() -> c_int {
    unsafe { fgetc(&mut *stdin) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getc_unlocked.html>.
///
/// Get a char from a stream without locking the stream
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getc_unlocked(stream: *mut FILE) -> c_int {
    if let Err(_) = unsafe { (*stream).try_set_byte_orientation_unlocked() } {
        return -1;
    }

    let mut buf = [0];

    match unsafe { (*stream).read(&mut buf) } {
        Ok(0) | Err(_) => EOF,
        Ok(_) => buf[0] as c_int,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getc_unlocked.html>.
///
/// Get a char from `stdin` without locking `stdin`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getchar_unlocked() -> c_int {
    unsafe { getc_unlocked(&mut *stdin) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/gets.html>.
///
/// Marked obsolescent in issue 7.
/// `fgets` is recommended instead, which is what this implementation calls.
///
/// Get a string from `stdin`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gets(s: *mut c_char) -> *mut c_char {
    unsafe { fgets(s, c_int::max_value(), &mut *stdin) }
}

/// See <https://pubs.opengroup.org/onlinepubs/7908799/xsh/getw.html>.
///
/// Was marked legacy and removed in issue 6.
///
/// Get an integer from `stream`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getw(stream: *mut FILE) -> c_int {
    let mut ret: c_int = 0;
    if unsafe {
        fread(
            &mut ret as *mut _ as *mut c_void,
            mem::size_of_val(&ret),
            1,
            stream,
        )
    } > 0
    {
        ret
    } else {
        -1
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pclose.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pclose(stream: *mut FILE) -> c_int {
    // TODO: rusty error handling?
    let pid = {
        let mut stream = unsafe { (*stream).lock() };

        if let Some(pid) = stream.pid.take() {
            pid
        } else {
            ERRNO.set(errno::ECHILD);
            return -1;
        }
    };

    unsafe { fclose(stream) };

    let mut wstatus = 0;
    if Sys::waitpid(pid, Some(Out::from_mut(&mut wstatus)), 0).or_minus_one_errno() == -1 {
        return -1;
    }

    wstatus
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/perror.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn perror(s: *const c_char) {
    let err = ERRNO.get();
    let err_str = if err >= 0 && err < STR_ERROR.len() as c_int {
        STR_ERROR[err as usize]
    } else {
        "Unknown error"
    };
    let mut w = platform::FileWriter::new(2);

    // The prefix, `s`, is optional (empty or NULL) according to the spec
    match unsafe { CStr::from_nullable_ptr(s) }
        .and_then(|s_cstr| str::from_utf8(s_cstr.to_bytes()).ok())
    {
        Some(s_str) if !s_str.is_empty() => w
            .write_fmt(format_args!("{}: {}\n", s_str, err_str))
            .unwrap(),
        _ => w.write_fmt(format_args!("{}\n", err_str)).unwrap(),
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/popen.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn popen(command: *const c_char, mode: *const c_char) -> *mut FILE {
    //TODO: share code with system

    let mode = unsafe { CStr::from_ptr(mode) };

    let mut cloexec = false;
    let mut write_opt = None;
    for b in mode.to_bytes().iter() {
        match b {
            b'e' => cloexec = true,
            b'r' if write_opt.is_none() => write_opt = Some(false),
            b'w' if write_opt.is_none() => write_opt = Some(true),
            _ => {
                ERRNO.set(errno::EINVAL);
                return ptr::null_mut();
            }
        }
    }

    let write = match write_opt {
        Some(some) => some,
        None => {
            ERRNO.set(errno::EINVAL);
            return ptr::null_mut();
        }
    };

    let mut pipes = [-1, -1];
    if unsafe { unistd::pipe(pipes.as_mut_ptr()) } != 0 {
        return ptr::null_mut();
    }

    let child_pid = unsafe { unistd::fork() };
    if child_pid == 0 {
        let command_nonnull = if command.is_null() {
            "exit 0\0".as_ptr()
        } else {
            command as *const u8
        };

        let shell = "/bin/sh\0".as_ptr();

        let args = [
            "sh\0".as_ptr(),
            "-c\0".as_ptr(),
            command_nonnull,
            ptr::null(),
        ];

        // Setup up stdin or stdout
        //TODO: dup errors are ignored, should they be?
        {
            if write {
                match unistd::dup2(pipes[0], 0) {
                    0 => {}
                    e => unsafe { stdlib::exit(127) },
                }
            } else {
                match unistd::dup2(pipes[1], 1) {
                    1 => {}
                    e => unsafe { stdlib::exit(127) },
                }
            }

            unistd::close(pipes[0]);
            unistd::close(pipes[1]);
        }

        unsafe { unistd::execv(shell as *const c_char, args.as_ptr() as *const *mut c_char) };

        unsafe { stdlib::exit(127) };

        unreachable!();
    } else if child_pid > 0 {
        let (fd, fd_mode): (_, CStr) = if write {
            unistd::close(pipes[0]);
            (pipes[1], if cloexec { c"we".into() } else { c"w".into() })
        } else {
            unistd::close(pipes[1]);
            (pipes[0], if cloexec { c"re".into() } else { c"r".into() })
        };

        helpers::_fdopen(fd, fd_mode)
            .map(|mut f| {
                f.pid = Some(child_pid);
                Box::into_raw(f)
            })
            .or_errno_null_mut()
    } else {
        ptr::null_mut()
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/putc.html>.
///
/// Put a character `c` into `stream`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn putc(c: c_int, stream: *mut FILE) -> c_int {
    let mut stream = unsafe { (*stream).lock() };
    unsafe { putc_unlocked(c, &mut *stream) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/putchar.html>.
///
/// Put a character `c` into `stdout`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn putchar(c: c_int) -> c_int {
    unsafe { fputc(c, &mut *stdout) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getc_unlocked.html>.
///
/// Put a character `c` into `stream` without locking `stream`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn putc_unlocked(c: c_int, stream: *mut FILE) -> c_int {
    if let Err(_) = unsafe { (*stream).try_set_byte_orientation_unlocked() } {
        return -1;
    }

    match unsafe { (*stream).write(&[c as u8]) } {
        Ok(0) | Err(_) => EOF,
        Ok(_) => c,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getc_unlocked.html>.
///
/// Put a character `c` into `stdout` without locking `stdout`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn putchar_unlocked(c: c_int) -> c_int {
    unsafe { putc_unlocked(c, stdout) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/puts.html>.
///
/// Put a string `s` into `stdout`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn puts(s: *const c_char) -> c_int {
    let mut stream = unsafe { (&mut *stdout).lock() };
    if let Err(_) = (*stream).try_set_byte_orientation_unlocked() {
        return -1;
    }

    let buf = unsafe { slice::from_raw_parts(s as *mut u8, strlen(s)) };

    if stream.write_all(&buf).is_err() {
        return -1;
    }
    if stream.write(&[b'\n']).is_err() {
        return -1;
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/7908799/xsh/putw.html>.
///
/// Marked legacy in SUS Version 2.
///
/// Put an integer `w` into `stream`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn putw(w: c_int, stream: *mut FILE) -> c_int {
    (unsafe { fwrite(&w as *const c_int as _, mem::size_of_val(&w), 1, stream) }) as i32 - 1
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/remove.html>.
///
/// Delete file or directory `path`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn remove(path: *const c_char) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    Sys::unlink(path)
        .or_else(|_err| Sys::rmdir(path))
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/rename.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rename(oldpath: *const c_char, newpath: *const c_char) -> c_int {
    let oldpath = unsafe { CStr::from_ptr(oldpath) };
    let newpath = unsafe { CStr::from_ptr(newpath) };
    Sys::rename(oldpath, newpath)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/renameat.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn renameat(
    old_dir: c_int,
    old_path: *const c_char,
    new_dir: c_int,
    new_path: *const c_char,
) -> c_int {
    let old_path = unsafe { CStr::from_ptr(old_path) };
    let new_path = unsafe { CStr::from_ptr(new_path) };
    Sys::renameat(old_dir, old_path, new_dir, new_path)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://www.man7.org/linux/man-pages/man2/rename.2.html>.
///
/// Non-POSIX. Seems to be a GNU extension.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn renameat2(
    old_dir: c_int,
    old_path: *const c_char,
    new_dir: c_int,
    new_path: *const c_char,
    flags: c_uint,
) -> c_int {
    let old_path = unsafe { CStr::from_ptr(old_path) };
    let new_path = unsafe { CStr::from_ptr(new_path) };
    Sys::renameat2(old_dir, old_path, new_dir, new_path, flags)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/rewind.html>.
///
/// Rewind `stream` back to the beginning of it
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rewind(stream: *mut FILE) {
    unsafe { fseeko(stream, 0, SEEK_SET) };
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setbuf.html>.
///
/// Reset `stream` to use buffer `buf`. Buffer must be `BUFSIZ` in length
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setbuf(stream: *mut FILE, buf: *mut c_char) {
    unsafe {
        setvbuf(
            stream,
            buf,
            if buf.is_null() { _IONBF } else { _IOFBF },
            BUFSIZ as usize,
        )
    };
}

/// See <https://www.man7.org/linux/man-pages/man3/setlinebuf.3.html>.
///
/// Non-POSIX.
///
/// Set buffering of `stream` to line buffered
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setlinebuf(stream: *mut FILE) {
    unsafe { setvbuf(stream, ptr::null_mut(), _IOLBF, 0) };
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setvbuf.html>.
///
/// Reset `stream` to use buffer `buf` of size `size`
/// If this isn't the meaning of unsafe, idk what is
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setvbuf(
    stream: *mut FILE,
    buf: *mut c_char,
    mode: c_int,
    mut size: size_t,
) -> c_int {
    let mut stream = unsafe { (*stream).lock() };
    // Set a buffer of size `size` if no buffer is given
    stream.read_buf = if buf.is_null() || size == 0 {
        if size == 0 {
            size = BUFSIZ as usize;
        }
        // TODO: Make it unbuffered if _IONBF
        // if mode == _IONBF {
        // } else {
        Buffer::Owned(vec![0; size as usize])
    // }
    } else {
        Buffer::Borrowed(unsafe { slice::from_raw_parts_mut(buf as *mut u8, size) })
    };
    stream.flags |= F_SVB;
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/009604599/functions/tempnam.html>.
///
/// Marked obsolescent in issue 7.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tempnam(dir: *const c_char, pfx: *const c_char) -> *mut c_char {
    unsafe fn is_appropriate(pos_dir: *const c_char) -> bool {
        !pos_dir.is_null() && unsafe { unistd::access(pos_dir, unistd::W_OK) } == 0
    }

    // directory search order is env!(TMPDIR), dir, P_tmpdir, "/tmp"
    let dirname = {
        let tmpdir = unsafe { stdlib::getenv(b"TMPDIR\0".as_ptr() as _) };
        [tmpdir, dir, P_tmpdir.as_ptr() as _]
            .iter()
            .copied()
            .skip_while(|&d| !unsafe { is_appropriate(d) })
            .next()
            .unwrap_or(b"/tmp\0".as_ptr() as _)
    };
    let dirname_len = unsafe { string::strlen(dirname) };

    let prefix_len = unsafe { string::strnlen_s(pfx, 5) };

    // allocate enough for dirname "/" prefix "XXXXXX\0"
    let mut out_buf =
        unsafe { platform::alloc(dirname_len + 1 + prefix_len + L_tmpnam as usize + 1) }
            as *mut c_char;

    if !out_buf.is_null() {
        // copy the directory name and prefix into the allocated buffer
        unsafe { out_buf.copy_from_nonoverlapping(dirname, dirname_len) };
        unsafe { *out_buf.add(dirname_len) = b'/' as _ };
        unsafe {
            out_buf
                .add(dirname_len + 1)
                .copy_from_nonoverlapping(pfx, prefix_len)
        };

        // use the same mechanism as tmpnam to get the file name
        if unsafe { tmpnam_inner(out_buf, dirname_len + 1 + prefix_len) }.is_null() {
            // failed to find a valid file name, so we need to free the buffer
            unsafe { platform::free(out_buf as _) };
            out_buf = ptr::null_mut();
        }
    }

    out_buf
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tmpfile.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tmpfile() -> *mut FILE {
    let mut file_name = *b"/tmp/tmpfileXXXXXX\0";
    let file_name = file_name.as_mut_ptr() as *mut c_char;
    let fd = unsafe { stdlib::mkstemp(file_name) };

    if fd < 0 {
        return ptr::null_mut();
    }

    let fp = unsafe { fdopen(fd, c"w+".as_ptr()) };
    {
        let file_name = unsafe { CStr::from_ptr(file_name) };
        Sys::unlink(file_name);
    }

    if fp.is_null() {
        Sys::close(fd);
    }

    fp
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tmpnam.html>.
///
/// Marked obsolescent in issue 7.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tmpnam(s: *mut c_char) -> *mut c_char {
    let buf = if s.is_null() {
        &raw mut TMPNAM_BUF as *mut _
    } else {
        s
    };

    unsafe { *buf = b'/' as _ };
    unsafe { tmpnam_inner(buf, 1) }
}

unsafe extern "C" fn tmpnam_inner(buf: *mut c_char, offset: usize) -> *mut c_char {
    const TEMPLATE: &[u8] = b"XXXXXX\0";

    unsafe {
        buf.add(offset)
            .copy_from_nonoverlapping(TEMPLATE.as_ptr() as _, TEMPLATE.len())
    };

    let err = platform::ERRNO.get();
    unsafe { stdlib::mktemp(buf) };
    platform::ERRNO.set(err);

    if unsafe { *buf } == 0 {
        ptr::null_mut()
    } else {
        buf
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ungetc.html>.
///
/// Push character `c` back onto `stream` so it'll be read next
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ungetc(c: c_int, stream: *mut FILE) -> c_int {
    let mut stream = unsafe { (*stream).lock() };
    if let Err(_) = (*stream).try_set_byte_orientation_unlocked() {
        return -1;
    }

    stream.unget.push(c as u8);
    c
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/vfprintf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vfprintf(file: *mut FILE, format: *const c_char, ap: va_list) -> c_int {
    let mut file = unsafe { (*file).lock() };
    if let Err(_) = file.try_set_byte_orientation_unlocked() {
        return -1;
    }

    unsafe { printf::printf(&mut *file, CStr::from_ptr(format), ap) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fprintf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fprintf(
    file: *mut FILE,
    format: *const c_char,
    mut __valist: ...
) -> c_int {
    unsafe { vfprintf(file, format, __valist.as_va_list()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/vdprintf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vdprintf(fd: c_int, format: *const c_char, ap: va_list) -> c_int {
    let mut f = File::new(fd);

    // We don't want to close the file on drop; we're merely
    // borrowing the file descriptor here
    f.reference = true;

    unsafe { printf::printf(f, CStr::from_ptr(format), ap) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/dprintf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dprintf(fd: c_int, format: *const c_char, mut __valist: ...) -> c_int {
    unsafe { vdprintf(fd, format, __valist.as_va_list()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/vfprintf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vprintf(format: *const c_char, ap: va_list) -> c_int {
    unsafe { vfprintf(&mut *stdout, format, ap) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fprintf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn printf(format: *const c_char, mut __valist: ...) -> c_int {
    unsafe { vfprintf(&mut *stdout, format, __valist.as_va_list()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/vfprintf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vasprintf(
    strp: *mut *mut c_char,
    format: *const c_char,
    ap: va_list,
) -> c_int {
    let mut alloc_writer = CVec::new();
    let ret = unsafe { printf::printf(&mut alloc_writer, CStr::from_ptr(format), ap) };
    alloc_writer.push(0).unwrap();
    alloc_writer.shrink_to_fit().unwrap();
    unsafe { *strp = alloc_writer.leak() as *mut c_char };
    ret
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fprintf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn asprintf(
    strp: *mut *mut c_char,
    format: *const c_char,
    mut __valist: ...
) -> c_int {
    unsafe { vasprintf(strp, format, __valist.as_va_list()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/vfprintf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vsnprintf(
    s: *mut c_char,
    n: size_t,
    format: *const c_char,
    ap: va_list,
) -> c_int {
    unsafe {
        printf::printf(
            &mut platform::StringWriter(s as *mut u8, n as usize),
            CStr::from_ptr(format),
            ap,
        )
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fprintf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn snprintf(
    s: *mut c_char,
    n: size_t,
    format: *const c_char,
    mut __valist: ...
) -> c_int {
    unsafe {
        printf::printf(
            &mut platform::StringWriter(s as *mut u8, n as usize),
            CStr::from_ptr(format),
            __valist.as_va_list(),
        )
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/vfprintf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vsprintf(s: *mut c_char, format: *const c_char, ap: va_list) -> c_int {
    unsafe {
        printf::printf(
            &mut platform::UnsafeStringWriter(s as *mut u8),
            CStr::from_ptr(format),
            ap,
        )
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fprintf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sprintf(
    s: *mut c_char,
    format: *const c_char,
    mut __valist: ...
) -> c_int {
    unsafe {
        printf::printf(
            &mut platform::UnsafeStringWriter(s as *mut u8),
            CStr::from_ptr(format),
            __valist.as_va_list(),
        )
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/vfscanf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vfscanf(file: *mut FILE, format: *const c_char, ap: va_list) -> c_int {
    let ret = {
        let mut file = unsafe { (*file).lock() };
        if let Err(_) = file.try_set_byte_orientation_unlocked() {
            return -1;
        }

        let f: &mut FILE = &mut *file;
        let reader: LookAheadReader = f.into();
        unsafe { scanf::scanf(reader, format, ap) }
    };
    ret
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fscanf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fscanf(
    file: *mut FILE,
    format: *const c_char,
    mut __valist: ...
) -> c_int {
    unsafe { vfscanf(file, format, __valist.as_va_list()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/vfscanf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vscanf(format: *const c_char, ap: va_list) -> c_int {
    unsafe { vfscanf(&mut *stdin, format, ap) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fscanf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scanf(format: *const c_char, mut __valist: ...) -> c_int {
    unsafe { vfscanf(&mut *stdin, format, __valist.as_va_list()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/vfscanf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vsscanf(s: *const c_char, format: *const c_char, ap: va_list) -> c_int {
    let reader = (s as *const u8).into();
    unsafe { scanf::scanf(reader, format, ap) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fscanf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sscanf(
    s: *const c_char,
    format: *const c_char,
    mut __valist: ...
) -> c_int {
    let reader = (s as *const u8).into();
    unsafe { scanf::scanf(reader, format, __valist.as_va_list()) }
}

pub unsafe fn flush_io_streams() {
    let flush = |stream: *mut FILE| {
        let stream = unsafe { &mut *stream };
        let _ = stream.flush();
    };
    flush(unsafe { stdout });
    flush(unsafe { stderr });
}
