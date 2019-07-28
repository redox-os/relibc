use alloc::vec::Vec;
use core::ptr;

use crate::{
    header::{stdio::FILE, stdlib},
    io::BufRead,
    platform::types::*,
};

#[no_mangle]
pub unsafe extern "C" fn __getline(
    lineptr: *mut *mut c_char,
    n: *mut size_t,
    stream: *mut FILE,
) -> ssize_t {
    __getdelim(lineptr, n, b'\n' as c_int, stream)
}

#[no_mangle]
pub unsafe extern "C" fn __getdelim(
    lineptr: *mut *mut c_char,
    n: *mut size_t,
    delim: c_int,
    stream: *mut FILE,
) -> ssize_t {
    let lineptr = &mut *lineptr;
    let n = &mut *n;
    let delim = delim as u8;

    //TODO: More efficient algorithm using lineptr and n instead of this vec
    let mut buf = Vec::new();
    let count = {
        let mut stream = (*stream).lock();
        match stream.read_until(delim, &mut buf) {
            Ok(ok) => ok,
            Err(err) => return -1,
        }
    };

    //TODO: Check errors and improve safety
    {
        // Allocate lineptr to size of buf and set n to size of lineptr
        *n = count + 1;
        *lineptr = stdlib::realloc(*lineptr as *mut c_void, *n) as *mut c_char;

        // Copy buf to lineptr
        ptr::copy(buf.as_ptr(), *lineptr as *mut u8, count);

        // NUL terminate lineptr
        *lineptr.offset(count as isize) = 0;

        // Return allocated size
        *n as ssize_t
    }
}
