use super::{internal, BUFSIZ, FILE, UNGET};
use compiler_builtins::mem::memset;
use stdlib::calloc;
use core::{mem, ptr};
use core::sync::atomic::AtomicBool;
use platform::types::*;
use super::constants::*;
use fcntl::*;
use platform;
use errno;

/// Parse mode flags as a string and output a mode flags integer
pub unsafe fn parse_mode_flags(mode_str: *const c_char) -> i32 {
    use string::strchr;
    let mut flags = if !strchr(mode_str, b'+' as i32).is_null() {
        O_RDWR
    } else if (*mode_str) == b'r' as i8 {
        O_RDONLY
    } else {
        O_WRONLY
    };
    if !strchr(mode_str, b'x' as i32).is_null() {
        flags |= O_EXCL;
    }
    if !strchr(mode_str, b'e' as i32).is_null() {
        flags |= O_CLOEXEC;
    }
    if (*mode_str) != b'r' as i8 {
        flags |= O_CREAT;
    }
    if (*mode_str) == b'w' as i8 {
        flags |= O_TRUNC;
    }
    if (*mode_str) != b'a' as i8 {
       flags |= O_APPEND;
    }

    flags
}

/// Open a file with the file descriptor `fd` in the mode `mode`
pub unsafe fn _fdopen(fd: c_int, mode: *const c_char) -> *mut FILE {
    use string::strchr;
    if *mode != b'r' as i8 && *mode != b'w' as i8 && *mode != b'a' as i8 {
        platform::errno = errno::EINVAL;
        return ptr::null_mut();
    }

    let mut flags = 0;
    if strchr(mode, b'+' as i32).is_null() {
        flags |= if *mode == b'r' as i8 { F_NOWR } else { F_NORD };
    }

    if !strchr(mode, b'e' as i32).is_null() {
        sys_fcntl(fd, F_SETFD, FD_CLOEXEC);
    }

    if *mode == 'a' as i8 {
        let f = sys_fcntl(fd, F_GETFL, 0);
        if (f & O_APPEND) == 0 {
            sys_fcntl(fd, F_SETFL, f | O_APPEND);
        }
        flags |= F_APP;
    }

    let file = calloc(mem::size_of::<FILE>() + BUFSIZ + UNGET, 1) as *mut FILE;
    // Allocate the file
    (*file) = FILE {
        flags: flags,
        rpos: ptr::null_mut(),
        rend: ptr::null_mut(),
        wend: ptr::null_mut(),
        wpos: ptr::null_mut(),
        wbase: ptr::null_mut(),
        fd: fd,
        buf: (file as *mut u8).add(mem::size_of::<FILE>() + UNGET),
        buf_size: BUFSIZ,
        buf_char: -1,
        unget: UNGET,
        lock: AtomicBool::new(false),
        write: Some(&internal::stdio_write),
        read: Some(&internal::stdio_read),
        seek: Some(&internal::stdio_seek),
    };
    file
}

/// Write buffer `buf` of length `l` into `stream`
pub unsafe fn fwritex(buf: *const u8, l: size_t, stream: *mut FILE) -> size_t {
    use compiler_builtins::mem::memcpy;
    let mut buf = buf;
    let mut l = l;
    if let Some(stream_write) = (*stream).write {
        if (*stream).wend.is_null() && !internal::to_write(stream) {
            // We can't write to this stream
            return 0;
        }
        if l > (*stream).wend as usize - (*stream).wpos as usize {
            // We can't fit all of buf in the buffer
            return (*stream_write)(stream, buf, l);
        }

        let i = if (*stream).buf_char >= 0 {
            let mut i = l;
            while i > 0 && *buf.offset(i as isize - 1) != b'\n' {
                i -= 1;
            }
            if i > 0 {
                let n = (*stream_write)(stream, buf, i);
                if n < i {
                    return n;
                }
                buf = buf.add(i);
                l -= i;
            }
            i
        } else {
            0
        };

        memcpy((*stream).wpos, buf, l);
        (*stream).wpos = (*stream).wpos.add(l);
        l + i
    } else {
        // We can't write to this stream
        0
    }
}

/// Flush `stream` without locking it.
pub unsafe fn fflush_unlocked(stream: *mut FILE) -> c_int {
    if (*stream).wpos > (*stream).wbase {
        if let Some(f) = (*stream).write {
            (*f)(stream, ptr::null(), 0);
            if (*stream).wpos.is_null() {
                return -1;
            }
        } else {
            return -1;
        }
    }

    if (*stream).rpos < (*stream).rend {
        if let Some(s) = (*stream).seek {
            (*s)(
                stream,
                (*stream).rpos as i64 - (*stream).rend as i64,
                SEEK_CUR,
            );
        }
    }

    (*stream).wpos = ptr::null_mut();
    (*stream).wend = ptr::null_mut();
    (*stream).wbase = ptr::null_mut();
    (*stream).rpos = ptr::null_mut();
    (*stream).rend = ptr::null_mut();
    0
}
