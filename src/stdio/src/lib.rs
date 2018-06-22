//! stdio implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/stdio.h.html

#![no_std]
#![feature(alloc)]

extern crate alloc;
extern crate errno;
extern crate fcntl;
extern crate platform;
extern crate stdlib;
extern crate string;
extern crate va_list as vl;

use core::str;
use core::ptr;
use core::fmt::{self, Error, Result};
use core::fmt::Write as WriteFmt;
use core::sync::atomic::{AtomicBool, Ordering};

use platform::types::*;
use platform::{c_str, errno, Read, Write};
use errno::STR_ERROR;
use vl::VaList as va_list;

mod scanf;
mod printf;

mod default;
pub use default::*;

mod constants;
pub use constants::*;

mod helpers;

mod internal;

#[repr(C)]
pub struct FILE {
    flags: c_int,
    rpos: *mut u8,
    rend: *mut u8,
    wend: *mut u8,
    wpos: *mut u8,
    wbase: *mut u8,
    fd: c_int,
    buf: *mut u8,
    buf_size: size_t,
    buf_char: i8,
    lock: AtomicBool,
    unget: size_t,
}

impl FILE {
    pub fn can_read(&mut self) -> bool {
        if self.flags & constants::F_BADJ > 0 {
            // Static and needs unget region
            self.buf = unsafe { self.buf.add(self.unget) };
            self.flags &= !constants::F_BADJ;
        }

        if self.wpos > self.wbase {
            self.write(&[]);
        }
        self.wpos = ptr::null_mut();
        self.wbase = ptr::null_mut();
        self.wend = ptr::null_mut();
        if self.flags & constants::F_NORD > 0 {
            self.flags |= constants::F_ERR;
            return false;
        }
        self.rpos = unsafe { self.buf.offset(self.buf_size as isize - 1) };
        self.rend = unsafe { self.buf.offset(self.buf_size as isize - 1) };
        if self.flags & constants::F_EOF > 0 {
            false
        } else {
            true
        }
    }
    pub fn can_write(&mut self) -> bool {
        if self.flags & constants::F_BADJ > 0 {
            // Static and needs unget region
            self.buf = unsafe { self.buf.add(self.unget) };
            self.flags &= !constants::F_BADJ;
        }

        if self.flags & constants::F_NOWR > 0 {
            self.flags &= constants::F_ERR;
            return false;
        }
        // Buffer repositioning
        self.rpos = ptr::null_mut();
        self.rend = ptr::null_mut();
        self.wpos = self.buf;
        self.wbase = self.buf;
        self.wend = unsafe { self.buf.offset(self.buf_size as isize - 1) };
        return true;
    }
    pub fn write(&mut self, to_write: &[u8]) -> usize {
        use core::slice;
        use core::mem;
        let len = self.wpos as usize - self.wbase as usize;
        let mut advance = 0;
        let mut f_buf: &'static _ = unsafe { slice::from_raw_parts(self.wbase, len) };
        let mut f_filled = false;
        let mut rem = f_buf.len() + to_write.len();
        loop {
            let mut count = if f_filled {
                platform::write(self.fd, &f_buf[advance..])
            } else {
                platform::write(self.fd, &f_buf[advance..]) + platform::write(self.fd, to_write)
            };
            if count == rem as isize {
                self.wend = unsafe { self.buf.add(self.buf_size - 1) };
                self.wpos = self.buf;
                self.wbase = self.buf;
                return to_write.len();
            }
            if count < 0 {
                self.wpos = ptr::null_mut();
                self.wbase = ptr::null_mut();
                self.wend = ptr::null_mut();
                self.flags |= constants::F_ERR;
                return 0;
            }
            rem -= count as usize;
            if count as usize > len {
                count -= len as isize;
                f_buf = unsafe { mem::transmute(to_write) };
                f_filled = true;
                advance = 0;
            }
            advance += count as usize;
        }
    }
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        use core::slice;
        // let buff = slice::from_raw_parts_mut(buf, size - !((*stream).buf_size == 0) as usize);
        let adj = !(self.buf_size == 0) as usize;
        let mut file_buf: &'static mut _ =
            unsafe { slice::from_raw_parts_mut(self.buf, self.buf_size) };
        let count = if buf.len() <= 1 + adj {
            platform::read(self.fd, file_buf)
        } else {
            platform::read(self.fd, buf) + platform::read(self.fd, file_buf)
        };
        if count <= 0 {
            self.flags |= if count == 0 {
                constants::F_EOF
            } else {
                constants::F_ERR
            };
            return 0;
        }
        if count as usize <= buf.len() - adj {
            return count as usize;
        }
        unsafe {
            // Adjust pointers
            self.rpos = self.buf;
            self.rend = self.buf.offset(count);
            buf[buf.len() - 1] = *self.rpos;
            self.rpos = self.rpos.add(1);
        }
        buf.len()
    }
    pub fn seek(&self, off: off_t, whence: c_int) -> off_t {
        unsafe { platform::lseek(self.fd, off, whence) }
    }
}
impl fmt::Write for FILE {
    fn write_str(&mut self, s: &str) -> Result {
        let s = s.as_bytes();
        if self.write(s) != s.len() {
            Err(Error)
        } else {
            Ok(())
        }
    }
}
impl Write for FILE {
    fn write_u8(&mut self, byte: u8) -> Result {
        if self.write(&[byte]) != 1 {
            Err(Error)
        } else {
            Ok(())
        }
    }
}
impl Read for FILE {
    fn read_u8(&mut self, byte: &mut u8) -> bool {
        let mut buf = [*byte];
        let n = self.read(&mut buf);
        *byte = buf[0];
        n > 0
    }
}

/// Clears EOF and ERR indicators on a stream
#[no_mangle]
pub extern "C" fn clearerr(stream: &mut FILE) {
    stream.flags &= !(F_EOF | F_ERR);
}

#[no_mangle]
pub extern "C" fn ctermid(s: *mut c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn cuserid(s: *mut c_char) -> *mut c_char {
    unimplemented!();
}

/// Close a file
/// This function does not guarentee that the file buffer will be flushed or that the file
/// descriptor will be closed, so if it is important that the file be written to, use `fflush()`
/// prior to using this function.
#[no_mangle]
pub extern "C" fn fclose(stream: &mut FILE) -> c_int {
    use stdlib::free;
    flockfile(stream);
    let r = helpers::fflush_unlocked(stream) | platform::close(stream.fd);
    if stream.flags & constants::F_PERM == 0 {
        // Not one of stdin, stdout or stderr
        unsafe {
            free(stream as *mut _ as *mut _);
        }
    }
    r
}

/// Open a file from a file descriptor
#[no_mangle]
pub extern "C" fn fdopen(fildes: c_int, mode: *const c_char) -> *mut FILE {
    unsafe { helpers::_fdopen(fildes, mode) }
}

/// Check for EOF
#[no_mangle]
pub extern "C" fn feof(stream: &mut FILE) -> c_int {
    flockfile(stream);
    let ret = stream.flags & F_EOF;
    funlockfile(stream);
    ret
}

/// Check for ERR
#[no_mangle]
pub extern "C" fn ferror(stream: &mut FILE) -> c_int {
    flockfile(stream);
    let ret = stream.flags & F_ERR;
    funlockfile(stream);
    ret
}

/// Flush output to stream, or sync read position
/// Ensure the file is unlocked before calling this function, as it will attempt to lock the file
/// itself.
#[no_mangle]
pub unsafe extern "C" fn fflush(stream: &mut FILE) -> c_int {
    flockfile(stream);

    let ret = helpers::fflush_unlocked(stream);

    funlockfile(stream);
    ret
}

/// Get a single char from a stream
#[no_mangle]
pub extern "C" fn fgetc(stream: &mut FILE) -> c_int {
    flockfile(stream);
    let c = getc_unlocked(stream);
    funlockfile(stream);
    c
}

/// Get the position of the stream and store it in pos
#[no_mangle]
pub extern "C" fn fgetpos(stream: &mut FILE, pos: *mut fpos_t) -> c_int {
    let off = internal::ftello(stream);
    if off < 0 {
        return -1;
    }
    unsafe {
        (*pos) = off;
    }
    0
}

/// Get a string from the stream
#[no_mangle]
pub extern "C" fn fgets(s: *mut c_char, n: c_int, stream: &mut FILE) -> *mut c_char {
    use string::memchr;
    use core::ptr::copy_nonoverlapping;

    flockfile(stream);
    let mut ptr = s as *mut u8;
    let mut n = n;

    if n <= 1 {
        funlockfile(stream);
        if n == 0 {
            return ptr::null_mut();
        }
        unsafe {
            (*s) = b'\0' as i8;
        }
        return s;
    }
    while n > 0 {
        let z = unsafe {
            memchr(
                stream.rpos as *const c_void,
                b'\n' as c_int,
                stream.rend as usize - stream.rpos as usize,
            ) as *mut u8
        };
        let k = if z.is_null() {
            stream.rend as usize - stream.rpos as usize
        } else {
            z as usize - stream.rpos as usize + 1
        };
        let k = if k as i32 > n { n as usize } else { k };
        unsafe {
            // Copy
            copy_nonoverlapping(stream.rpos, ptr, k);
            // Reposition pointers
            stream.rpos = stream.rpos.add(k);
            ptr = ptr.add(k);
        }
        n -= k as i32;
        if !z.is_null() || n < 1 {
            break;
        }
        let c = getc_unlocked(stream);
        if c < 0 {
            break;
        }
        n -= 1;

        unsafe {
            // Pointer stuff
            *ptr = c as u8;
            ptr = ptr.add(1);
        }

        if c as u8 == b'\n' {
            break;
        }
    }
    if !s.is_null() {
        unsafe {
            *ptr = 0;
        }
    }
    funlockfile(stream);
    s
}

/// Get the underlying file descriptor
#[no_mangle]
pub extern "C" fn fileno(stream: &mut FILE) -> c_int {
    flockfile(stream);
    funlockfile(stream);
    stream.fd
}

/// Lock the file
/// Do not call any functions other than those with the `_unlocked` postfix while the file is
/// locked
#[no_mangle]
pub extern "C" fn flockfile(file: &mut FILE) {
    while ftrylockfile(file) != 0 {}
}

/// Open the file in mode `mode`
#[no_mangle]
pub unsafe extern "C" fn fopen(filename: *const c_char, mode: *const c_char) -> *mut FILE {
    use core::ptr;
    let initial_mode = *mode;
    if initial_mode != b'r' as i8 && initial_mode != b'w' as i8 && initial_mode != b'a' as i8 {
        platform::errno = errno::EINVAL;
        return ptr::null_mut();
    }

    let flags = helpers::parse_mode_flags(mode);

    let fd = fcntl::sys_open(filename, flags, 0o666);
    if fd < 0 {
        return ptr::null_mut();
    }

    if flags & fcntl::O_CLOEXEC > 0 {
        fcntl::sys_fcntl(fd, fcntl::F_SETFD, fcntl::FD_CLOEXEC);
    }

    let f = helpers::_fdopen(fd, mode);
    if f.is_null() {
        platform::close(fd);
        return ptr::null_mut();
    }
    f
}

/// Insert a character into the stream
#[no_mangle]
pub extern "C" fn fputc(c: c_int, stream: &mut FILE) -> c_int {
    flockfile(stream);
    let c = putc_unlocked(c, stream);
    funlockfile(stream);
    c
}

/// Insert a string into a stream
#[no_mangle]
pub extern "C" fn fputs(s: *const c_char, stream: &mut FILE) -> c_int {
    extern "C" {
        fn strlen(s: *const c_char) -> size_t;
    }
    let len = unsafe { strlen(s) };
    (fwrite(s as *const c_void, 1, len, stream) == len) as c_int - 1
}

/// Read `nitems` of size `size` into `ptr` from `stream`
#[no_mangle]
pub extern "C" fn fread(ptr: *mut c_void, size: usize, nitems: usize, stream: &mut FILE) -> usize {
    use core::ptr::copy_nonoverlapping;
    use core::slice;
    let mut dest = ptr as *mut u8;
    let len = size * nitems;
    let mut l = len as isize;

    flockfile(stream);

    if stream.rend > stream.rpos {
        // We have some buffered data that can be read
        let diff = stream.rend as usize - stream.rpos as usize;
        let k = if diff < l as usize { diff } else { l as usize };
        unsafe {
            // Copy data
            copy_nonoverlapping(stream.rpos, dest, k);
            // Reposition pointers
            stream.rpos = stream.rpos.add(k);
            dest = dest.add(k);
        }
        l -= k as isize;
    }

    while l > 0 {
        let k = if !stream.can_read() {
            0
        } else {
            stream.read(unsafe { slice::from_raw_parts_mut(dest, l as usize) })
        };

        if k == 0 {
            funlockfile(stream);
            return (len - l as usize) / 2;
        }

        l -= k as isize;
        unsafe {
            // Reposition
            dest = dest.add(k);
        }
    }

    funlockfile(stream);
    nitems
}

#[no_mangle]
pub extern "C" fn freopen(
    filename: *const c_char,
    mode: *const c_char,
    stream: &mut FILE,
) -> *mut FILE {
    let mut flags = unsafe { helpers::parse_mode_flags(mode) };
    flockfile(stream);

    helpers::fflush_unlocked(stream);
    if filename.is_null() {
        // Reopen stream in new mode
        if flags & fcntl::O_CLOEXEC > 0 {
            fcntl::sys_fcntl(stream.fd, fcntl::F_SETFD, fcntl::FD_CLOEXEC);
        }
        flags &= !(fcntl::O_CREAT | fcntl::O_EXCL | fcntl::O_CLOEXEC);
        if fcntl::sys_fcntl(stream.fd, fcntl::F_SETFL, flags) < 0 {
            funlockfile(stream);
            fclose(stream);
            return ptr::null_mut();
        }
    } else {
        let new = unsafe { fopen(filename, mode) };
        if new.is_null() {
            funlockfile(stream);
            fclose(stream);
            return ptr::null_mut();
        }
        let new = unsafe { &mut *new }; // Should be safe, new is not null
        if new.fd == stream.fd {
            new.fd = -1;
        } else if platform::dup2(new.fd, stream.fd) < 0
            || fcntl::sys_fcntl(stream.fd, fcntl::F_SETFL, flags & fcntl::O_CLOEXEC) < 0
        {
            fclose(new);
            funlockfile(stream);
            fclose(stream);
            return ptr::null_mut();
        }
        stream.flags = (stream.flags & constants::F_PERM) | new.flags;
        fclose(new);
    }
    funlockfile(stream);
    stream
}

/// Seek to an offset `offset` from `whence`
#[no_mangle]
pub extern "C" fn fseek(stream: &mut FILE, offset: c_long, whence: c_int) -> c_int {
    if fseeko(stream, offset as off_t, whence) != -1 {
        return 0;
    }
    -1
}

/// Seek to an offset `offset` from `whence`
#[no_mangle]
pub extern "C" fn fseeko(stream: &mut FILE, offset: off_t, whence: c_int) -> c_int {
    let mut off = offset;
    flockfile(stream);
    // Adjust for what is currently in the buffer
    if whence == SEEK_CUR {
        off -= (stream.rend as usize - stream.rpos as usize) as i64;
    }
    if stream.wpos > stream.wbase {
        stream.write(&[]);
        if stream.wpos.is_null() {
            return -1;
        }
    }
    stream.wpos = ptr::null_mut();
    stream.wend = ptr::null_mut();
    stream.wbase = ptr::null_mut();
    if stream.seek(off, whence) < 0 {
        return -1;
    }
    stream.rpos = ptr::null_mut();
    stream.rend = ptr::null_mut();
    stream.flags &= !F_EOF;
    funlockfile(stream);
    0
}

/// Seek to a position `pos` in the file from the beginning of the file
#[no_mangle]
pub unsafe extern "C" fn fsetpos(stream: &mut FILE, pos: *const fpos_t) -> c_int {
    fseek(stream, *pos as off_t, SEEK_SET)
}

/// Get the current position of the cursor in the file
#[no_mangle]
pub unsafe extern "C" fn ftell(stream: &mut FILE) -> c_long {
    ftello(stream) as c_long
}

/// Get the current position of the cursor in the file
#[no_mangle]
pub extern "C" fn ftello(stream: &mut FILE) -> off_t {
    flockfile(stream);
    let pos = internal::ftello(stream);
    funlockfile(stream);
    pos
}

/// Try to lock the file. Returns 0 for success, 1 for failure
#[no_mangle]
pub extern "C" fn ftrylockfile(file: &mut FILE) -> c_int {
    file.lock.compare_and_swap(false, true, Ordering::Acquire) as c_int
}

/// Unlock the file
#[no_mangle]
pub extern "C" fn funlockfile(file: &mut FILE) {
    file.lock.store(false, Ordering::Release);
}

/// Write `nitems` of size `size` from `ptr` to `stream`
#[no_mangle]
pub extern "C" fn fwrite(
    ptr: *const c_void,
    size: usize,
    nitems: usize,
    stream: &mut FILE,
) -> usize {
    let l = size * nitems;
    let nitems = if size == 0 { 0 } else { nitems };
    flockfile(stream);
    let k = helpers::fwritex(ptr as *const u8, l, stream);
    funlockfile(stream);
    if k == l {
        nitems
    } else {
        k / size
    }
}

/// Get a single char from a stream
#[no_mangle]
pub extern "C" fn getc(stream: &mut FILE) -> c_int {
    flockfile(stream);
    let c = getc_unlocked(stream);
    funlockfile(stream);
    c
}

/// Get a single char from `stdin`
#[no_mangle]
pub unsafe extern "C" fn getchar() -> c_int {
    fgetc(&mut *stdin)
}

/// Get a char from a stream without locking the stream
#[no_mangle]
pub extern "C" fn getc_unlocked(stream: &mut FILE) -> c_int {
    if stream.rpos < stream.rend {
        unsafe {
            let ret = *stream.rpos as c_int;
            stream.rpos = stream.rpos.add(1);
            ret
        }
    } else {
        let mut c = [0u8; 1];
        if stream.can_read() && stream.read(&mut c) == 1 {
            c[0] as c_int
        } else {
            -1
        }
    }
}

/// Get a char from `stdin` without locking `stdin`
#[no_mangle]
pub unsafe extern "C" fn getchar_unlocked() -> c_int {
    getc_unlocked(&mut *stdin)
}

/// Get a string from `stdin`
#[no_mangle]
pub unsafe extern "C" fn gets(s: *mut c_char) -> *mut c_char {
    use core::i32;
    fgets(s, i32::MAX, &mut *stdin)
}

/// Get an integer from `stream`
#[no_mangle]
pub extern "C" fn getw(stream: &mut FILE) -> c_int {
    use core::mem;
    let mut ret: c_int = 0;
    if fread(
        &mut ret as *mut c_int as *mut c_void,
        mem::size_of_val(&ret),
        1,
        stream,
    ) > 0
    {
        ret
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn pclose(stream: &mut FILE) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn perror(s: *const c_char) {
    let s_str = str::from_utf8_unchecked(c_str(s));

    let mut w = platform::FileWriter(2);
    if errno >= 0 && errno < STR_ERROR.len() as c_int {
        w.write_fmt(format_args!("{}: {}\n", s_str, STR_ERROR[errno as usize]));
    } else {
        w.write_fmt(format_args!("{}: Unknown error {}\n", s_str, errno));
    }
}

#[no_mangle]
pub extern "C" fn popen(command: *const c_char, mode: *const c_char) -> *mut FILE {
    unimplemented!();
}

/// Put a character `c` into `stream`
#[no_mangle]
pub extern "C" fn putc(c: c_int, stream: &mut FILE) -> c_int {
    flockfile(stream);
    let ret = putc_unlocked(c, stream);
    funlockfile(stream);
    ret
}

/// Put a character `c` into `stdout`
#[no_mangle]
pub unsafe extern "C" fn putchar(c: c_int) -> c_int {
    fputc(c, &mut *stdout)
}

/// Put a character `c` into `stream` without locking `stream`
#[no_mangle]
pub extern "C" fn putc_unlocked(c: c_int, stream: &mut FILE) -> c_int {
    if c as i8 != stream.buf_char && stream.wpos < stream.wend {
        unsafe {
            *stream.wpos = c as u8;
            stream.wpos = stream.wpos.add(1);
            c
        }
    } else {
        if stream.wend.is_null() && stream.can_write() {
            -1
        } else if c as i8 != stream.buf_char && stream.wpos < stream.wend {
            unsafe {
                *stream.wpos = c as u8;
                stream.wpos = stream.wpos.add(1);
                c
            }
        } else if stream.write(&[c as u8]) != 1 {
            -1
        } else {
            c
        }
    }
}

/// Put a character `c` into `stdout` without locking `stdout`
#[no_mangle]
pub unsafe extern "C" fn putchar_unlocked(c: c_int) -> c_int {
    putc_unlocked(c, &mut *stdout)
}

/// Put a string `s` into `stdout`
#[no_mangle]
pub unsafe extern "C" fn puts(s: *const c_char) -> c_int {
    let ret = (fputs(s, &mut *stdout) > 0) || (putchar_unlocked(b'\n' as c_int) > 0);
    if ret {
        0
    } else {
        -1
    }
}

/// Put an integer `w` into `stream`
#[no_mangle]
pub extern "C" fn putw(w: c_int, stream: &mut FILE) -> c_int {
    use core::mem;
    fwrite(&w as *const i32 as _, mem::size_of_val(&w), 1, stream) as i32 - 1
}

/// Delete file or directory `path`
#[no_mangle]
pub extern "C" fn remove(path: *const c_char) -> c_int {
    let r = platform::unlink(path);
    if r == -errno::EISDIR {
        platform::rmdir(path)
    } else {
        r
    }
}

#[no_mangle]
pub extern "C" fn rename(oldpath: *const c_char, newpath: *const c_char) -> c_int {
    platform::rename(oldpath, newpath)
}

/// Rewind `stream` back to the beginning of it
#[no_mangle]
pub extern "C" fn rewind(stream: &mut FILE) {
    fseeko(stream, 0, SEEK_SET);
    flockfile(stream);
    stream.flags &= !F_ERR;
    funlockfile(stream);
}

/// Reset `stream` to use buffer `buf`. Buffer must be `BUFSIZ` in length
#[no_mangle]
pub extern "C" fn setbuf(stream: &mut FILE, buf: *mut c_char) {
    unsafe {
        setvbuf(
            stream,
            buf,
            if buf.is_null() { _IONBF } else { _IOFBF },
            BUFSIZ as usize,
        )
    };
}

/// Reset `stream` to use buffer `buf` of size `size`
/// If this isn't the meaning of unsafe, idk what is
#[no_mangle]
pub unsafe extern "C" fn setvbuf(
    stream: &mut FILE,
    buf: *mut c_char,
    mode: c_int,
    size: usize,
) -> c_int {
    // TODO: Check correctness
    use stdlib::calloc;
    let mut buf = buf;
    if buf.is_null() && mode != _IONBF {
        buf = calloc(size, 1) as *mut c_char;
    }
    (*stream).buf_size = size;
    (*stream).buf_char = -1;
    if mode == _IONBF {
        (*stream).buf_size = 0;
    } else if mode == _IOLBF {
        (*stream).buf_char = b'\n' as i8;
    }
    (*stream).flags |= F_SVB;
    (*stream).buf = buf as *mut u8;
    0
}

#[no_mangle]
pub extern "C" fn tempnam(dir: *const c_char, pfx: *const c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn tmpfile() -> *mut FILE {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn tmpnam(s: *mut c_char) -> *mut c_char {
    unimplemented!();
}

/// Push character `c` back onto `stream` so it'll be read next
#[no_mangle]
pub extern "C" fn ungetc(c: c_int, stream: &mut FILE) -> c_int {
    if c < 0 {
        c
    } else {
        flockfile(stream);
        if stream.rpos.is_null() {
            stream.can_read();
        }
        if stream.rpos.is_null() || stream.rpos <= unsafe { stream.buf.sub(stream.unget) } {
            funlockfile(stream);
            return -1;
        }

        unsafe {
            stream.rpos = stream.rpos.sub(1);
            *stream.rpos = c as u8;
        }
        stream.flags &= !F_EOF;

        funlockfile(stream);
        c
    }
}

#[no_mangle]
pub unsafe extern "C" fn vfprintf(file: &mut FILE, format: *const c_char, ap: va_list) -> c_int {
    printf::printf(file, format, ap)
}

#[no_mangle]
pub unsafe extern "C" fn vprintf(format: *const c_char, ap: va_list) -> c_int {
    vfprintf(&mut *stdout, format, ap)
}

#[no_mangle]
pub unsafe extern "C" fn vsnprintf(
    s: *mut c_char,
    n: usize,
    format: *const c_char,
    ap: va_list,
) -> c_int {
    printf::printf(
        &mut platform::StringWriter(s as *mut u8, n as usize),
        format,
        ap,
    )
}

#[no_mangle]
pub unsafe extern "C" fn vsprintf(s: *mut c_char, format: *const c_char, ap: va_list) -> c_int {
    printf::printf(&mut platform::UnsafeStringWriter(s as *mut u8), format, ap)
}

#[no_mangle]
pub unsafe extern "C" fn vfscanf(file: &mut FILE, format: *const c_char, ap: va_list) -> c_int {
    scanf::scanf(file, format, ap)
}

#[no_mangle]
pub unsafe extern "C" fn vscanf(format: *const c_char, ap: va_list) -> c_int {
    vfscanf(&mut *stdin, format, ap)
}

#[no_mangle]
pub unsafe extern "C" fn vsscanf(s: *const c_char, format: *const c_char, ap: va_list) -> c_int {
    scanf::scanf(&mut platform::UnsafeStringReader(s as *const u8), format, ap)
}
