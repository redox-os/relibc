// https://pubs.opengroup.org/onlinepubs/9699919799/functions/getline.html

use alloc::{string::String, vec::Vec};
use core::{fmt::Write, intrinsics::unlikely, ops::Deref, ptr};

use crate::{
    header::{
        errno::{EINVAL, ENOMEM, ENOSPC, EOVERFLOW},
        stdio,
        stdio::FILE,
        stdlib,
    },
    io::BufRead,
    platform,
    platform::types::*,
};

use crate::{
    header::stdio::{default_stdout, feof, ferror, F_EOF, F_ERR},
    platform::ERRNO,
};

/// see getdelim (getline is a special case of getdelim with delim == '\n')
#[no_mangle]
pub unsafe extern "C" fn getline(
    lineptr: *mut *mut c_char,
    n: *mut size_t,
    stream: *mut FILE,
) -> ssize_t {
    getdelim(lineptr, n, b'\n' as c_int, stream)
}

// One *could* read the standard as 'getdelim sets the stream error flag on *any* error, though
// since glibc doesn't seem to do this, I won't either

/// https://pubs.opengroup.org/onlinepubs/9699919799/functions/getline.html
///
/// # Safety
/// - `lineptr, *lineptr, `n`, `stream` pointers must be valid and have to be aligned.
/// - `stream` has to be a valid file handle returned by fopen and likes.
///
/// # Deviation from POSIX
/// - **EINVAL is set on stream being NULL or delim not fitting into char** (POSIX allows UB)
/// - **`*n` can contain invalid data.** The buffer size `n` is not read, instead realloc is called each time. That is in principle
/// inefficent since the buffer is reallocated in memory for every call, but if `n` is by mistake
/// bigger than the number of bytes allocated for the buffer, there can be no out-of-bounds write.
/// - On non-stream-related errors, the error indicator of the stream is *not* set. Posix states
/// "If an error occurs, the error indicator for the stream shall be set, and the function shall
/// return -1 and set errno to indicate the error." but in cases that produce EINVAL even glibc
/// doesn't seem to set the error indicator, so we also don't.
#[no_mangle]
pub unsafe extern "C" fn getdelim(
    lineptr: *mut *mut c_char,
    n: *mut size_t,
    delim: c_int,
    stream: *mut FILE,
) -> ssize_t {
    let (lineptr, n, stream) =
        if let (Some(ptr), Some(n), Some(file)) = (lineptr.as_mut(), n.as_mut(), stream.as_mut()) {
            (ptr, n, file)
        } else {
            ERRNO.set(EINVAL);
            return -1 as ssize_t;
        };

    if feof(stream) != 0 || ferror(stream) != 0 {
        return -1 as ssize_t;
    }

    // POSIX specifies UB but we test anyway
    // returning EINVAL in that case
    let delim: u8 = if let Ok(delim) = delim.try_into() {
        delim
    } else {
        ERRNO.set(EINVAL);
        return -1;
    };

    //TODO: More efficient algorithm using lineptr and n instead of this vec
    let mut buf = Vec::new();
    let count = {
        let mut stream = (*stream).lock();
        match stream.read_until(delim, &mut buf) {
            Ok(ok) => ok,
            Err(err) => {
                stream.flags &= F_ERR;
                return -1;
            }
        }
    };

    // "[EOVERFLOW]
    // The number of bytes to be written into the buffer, including the delimiter character (if encountered), would exceed {SSIZE_MAX}."
    if unlikely(count > ssize_t::MAX as usize) {
        ERRNO.set(EOVERFLOW);
        return -1;
    }

    // we reached EOF if either
    // - we have no last elem (because vec is empty), or
    // - the last elem doesn't match the delimiter
    let eof_reached = if let Some(last) = buf.last() {
        *last == delim
    } else {
        true
    };

    // "If the end-of-file indicator for the stream is set, or if no characters were read and the
    // stream is at end-of-file, the end-of-file indicator for the stream shall be set and the
    // function shall return -1."
    if eof_reached {
        stream.flags &= F_EOF;
        if count == 0 {
            return -1;
        }
    }

    //TODO: Check errors and improve safety
    {
        // Allocate lineptr to size of buf plus NUL byte and set n to size of lineptr
        *n = count + 1;
        // The advantage in always realloc'ing is that even if the user supplies a wrong n, this
        // doesn't break
        *lineptr = stdlib::realloc(*lineptr as *mut c_void, *n) as *mut c_char;
        if unlikely(lineptr.is_null() && *n != 0usize) {
            // memory error; realloc returns NULL on alloc'ing 0 bytes
            ERRNO.set(ENOMEM);
            return -1;
        }

        // Copy buf to lineptr
        ptr::copy(buf.as_ptr(), *lineptr as *mut u8, count);

        // NUL terminate lineptr
        *lineptr.offset(count as isize) = 0;

        // TODO remove
        /*eprintln!(
            "[DBG]{}: {}, {:?}, {:?}, {:?}", line!(),
            String::from_utf8(buf).unwrap(), count, *n, *lineptr
        );*/
        // Return allocated size
        count as ssize_t
    }
}
