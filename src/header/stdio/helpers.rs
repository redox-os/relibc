use core::sync::atomic::AtomicBool;
use core::{mem, ptr};

use header::errno;
use header::fcntl::*;
use header::string::strchr;
use platform;
use platform::types::*;

use super::constants::*;
use super::{BUFSIZ, FILE, UNGET};

/// Parse mode flags as a string and output a mode flags integer
pub unsafe fn parse_mode_flags(mode_str: *const c_char) -> i32 {
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
    } else if (*mode_str) == b'a' as i8 {
        flags |= O_APPEND;
    }

    flags
}

/// Open a file with the file descriptor `fd` in the mode `mode`
pub unsafe fn _fdopen(fd: c_int, mode: *const c_char) -> Option<*mut FILE> {
    if *mode != b'r' as i8 && *mode != b'w' as i8 && *mode != b'a' as i8 {
        platform::errno = errno::EINVAL;
        return None;
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

    let f = platform::alloc(mem::size_of::<FILE>()) as *mut FILE;
    // Allocate the file
    if f.is_null() {
        None
    } else {
        ptr::write(
            f,
            FILE {
                flags: flags,
                read: None,
                write: None,
                fd: fd,
                buf: vec![0u8; BUFSIZ + UNGET],
                buf_char: -1,
                unget: UNGET,
                lock: AtomicBool::new(false),
            },
        );
        Some(f)
    }
}

/// Write buffer `buf` of length `l` into `stream`
pub fn fwritex(buf: *const u8, l: size_t, stream: &mut FILE) -> size_t {
    use core::ptr::copy_nonoverlapping;
    use core::slice;

    let buf: &'static [u8] = unsafe { slice::from_raw_parts(buf, l) };
    let mut l = l;
    let mut advance = 0;

    if stream.write.is_none() && !stream.can_write() {
        // We can't write to this stream
        return 0;
    }
    if let Some((wbase, mut wpos, wend)) = stream.write {
        if l > wend - wpos {
            // We can't fit all of buf in the buffer
            return stream.write(buf);
        }

        let i = if stream.buf_char >= 0 {
            let mut i = l;
            while i > 0 && buf[i - 1] != b'\n' {
                i -= 1;
            }
            if i > 0 {
                let n = stream.write(buf);
                match stream.write {
                    Some((_, new_wpos, _)) => wpos = new_wpos,
                    None => unreachable!("stream.write should never be None after a write call")
                }

                if n < i {
                    return n;
                }
                advance += i;
                l -= i;
            }
            i
        } else {
            0
        };

        unsafe {
            copy_nonoverlapping(
                &buf[advance..] as *const _ as *const u8,
                &mut stream.buf[wpos..] as *mut _ as *mut u8,
                l,
            );
        }
        stream.write = Some((wbase, wpos + l, wend));
        l + i
    } else {
        0
    }
}

/// Flush `stream` without locking it.
pub fn fflush_unlocked(stream: &mut FILE) -> c_int {
    if let Some((wbase, wpos, _)) = stream.write {
        if wpos > wbase {
            stream.write(&[]);
            /*
            if stream.wpos.is_null() {
            return -1;
        }
             */
        }
    }

    if let Some((rpos, rend)) = stream.read {
        if rpos < rend {
            stream.seek(rpos as i64 - rend as i64, SEEK_CUR);
        }
    }

    stream.write = None;
    stream.read = None;
    0
}
