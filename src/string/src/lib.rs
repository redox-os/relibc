//! string implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/string.h.html

#![no_std]

extern crate errno;
extern crate platform;
extern crate stdlib;

use platform::types::*;
use errno::*;
use core::cmp;
use core::usize;
use core::ptr;

#[no_mangle]
pub unsafe extern "C" fn memccpy(
    dest: *mut c_void,
    src: *const c_void,
    c: c_int,
    n: usize,
) -> *mut c_void {
    let to = memchr(src, c, n);
    if to.is_null() {
        return to;
    }
    let dist = (to as usize) - (src as usize);
    if memcpy(dest, src, dist).is_null() {
        return ptr::null_mut();
    }
    (dest as *mut u8).offset(dist as isize + 1) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn memchr(s: *const c_void, c: c_int, n: usize) -> *mut c_void {
    let s = s as *mut u8;
    let c = c as u8;
    for i in 0..n {
        if *s.offset(i as isize) == c {
            return s.offset(i as isize) as *mut c_void;
        }
    }
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> c_int {
    let mut i = 0;
    while i < n {
        let a = *(s1 as *const u8).offset(i as isize);
        let b = *(s2 as *const u8).offset(i as isize);
        if a != b {
            return a as i32 - b as i32;
        }
        i += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(s1: *mut c_void, s2: *const c_void, n: usize) -> *mut c_void {
    platform::memcpy(s1, s2, n)
}

#[no_mangle]
pub unsafe extern "C" fn memmove(s1: *mut c_void, s2: *const c_void, n: usize) -> *mut c_void {
    if s2 < s1 as *const c_void {
        // copy from end
        let mut i = n;
        while i != 0 {
            i -= 1;
            *(s1 as *mut u8).offset(i as isize) = *(s2 as *const u8).offset(i as isize);
        }
    } else {
        // copy from beginning
        let mut i = 0;
        while i < n {
            *(s1 as *mut u8).offset(i as isize) = *(s2 as *const u8).offset(i as isize);
            i += 1;
        }
    }
    s1
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void {
    let mut i = 0;
    while i < n {
        *(s as *mut u8).offset(i as isize) = c as u8;
        i += 1;
    }
    s
}

#[no_mangle]
pub unsafe extern "C" fn strcat(s1: *mut c_char, s2: *const c_char) -> *mut c_char {
    strncat(s1, s2, usize::MAX)
}

#[no_mangle]
pub unsafe extern "C" fn strchr(s: *const c_char, c: c_int) -> *mut c_char {
    let c = c as i8;
    let mut i = 0;
    while *s.offset(i) != 0 {
        if *s.offset(i) == c {
            return s.offset(i) as *mut c_char;
        }
        i += 1;
    }
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int {
    strncmp(s1, s2, usize::MAX)
}

#[no_mangle]
pub extern "C" fn strcoll(s1: *const c_char, s2: *const c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn strcpy(s1: *mut c_char, s2: *const c_char) -> *mut c_char {
    strncpy(s1, s2, usize::MAX)
}

#[no_mangle]
pub unsafe extern "C" fn strcspn(s1: *const c_char, s2: *const c_char) -> size_t {
    use core::mem;

    let s1 = s1 as *const u8;
    let s2 = s2 as *const u8;

    // The below logic is effectively ripped from the musl implementation

    let mut byteset = [0usize; 32 / mem::size_of::<usize>()];

    let mut i = 0;
    while *s2.offset(i) != 0 {
        byteset[(*s2.offset(i) as usize) / (8 * byteset.len())] |=
            1 << (*s2.offset(i) as usize % (8 * byteset.len()));
        i += 1;
    }

    i = 0; // reset
    while *s1.offset(i) != 0 {
        if byteset[(*s1.offset(i) as usize) / (8 * byteset.len())]
            & 1 << (*s1.offset(i) as usize % (8 * byteset.len())) > 0
        {
            break;
        }
        i += 1;
    }
    i as size_t
}

#[no_mangle]
pub unsafe extern "C" fn strdup(s1: *const c_char) -> *mut c_char {
    strndup(s1, usize::MAX)
}

#[no_mangle]
pub unsafe extern "C" fn strndup(s1: *const c_char, size: usize) -> *mut c_char {
    let len = strnlen(s1, size);

    // the "+ 1" is to account for the NUL byte
    let buffer = stdlib::malloc(len + 1) as *mut c_char;
    if buffer.is_null() {
        platform::errno = ENOMEM as c_int;
    } else {
        //memcpy(buffer, s1, len)
        for i in 0..len as isize {
            *buffer.offset(i) = *s1.offset(i);
        }
        *buffer.offset(len as isize) = 0;
    }

    buffer
}

#[no_mangle]
pub unsafe extern "C" fn strerror(errnum: c_int) -> *mut c_char {
    use core::fmt::Write;

    static mut strerror_buf: [u8; 256] = [0; 256];

    let mut w = platform::StringWriter(strerror_buf.as_mut_ptr(), strerror_buf.len());

    if errnum >= 0 && errnum < STR_ERROR.len() as c_int {
        w.write_str(STR_ERROR[errnum as usize]);
    } else {
        w.write_fmt(format_args!("Unknown error {}", errnum));
    }

    strerror_buf.as_mut_ptr() as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const c_char) -> size_t {
    strnlen(s, usize::MAX)
}

#[no_mangle]
pub unsafe extern "C" fn strnlen(s: *const c_char, size: usize) -> size_t {
    platform::c_str_n(s, size).len() as size_t
}

#[no_mangle]
pub unsafe extern "C" fn strncat(s1: *mut c_char, s2: *const c_char, n: usize) -> *mut c_char {
    let mut idx = strlen(s1 as *const _) as isize;
    for i in 0..n as isize {
        if *s2.offset(i) == 0 {
            break;
        }

        *s1.offset(idx) = *s2.offset(i);
        idx += 1;
    }
    *s1.offset(idx) = 0;

    s1
}

#[no_mangle]
pub unsafe extern "C" fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int {
    let s1 = core::slice::from_raw_parts(s1 as *const c_uchar, n);
    let s2 = core::slice::from_raw_parts(s2 as *const c_uchar, n);

    for (&a, &b) in s1.iter().zip(s2.iter()) {
        let val = (a as c_int) - (b as c_int);
        if a != b || a == 0 {
            return val;
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn strncpy(s1: *mut c_char, s2: *const c_char, n: usize) -> *mut c_char {
    let s2_slice = platform::c_str_n(s2, n);
    let s2_len = s2_slice.len();

    //memcpy(s1 as *mut _, s2 as *const _, cmp::min(n, s2_len));
    let mut idx = 0;
    for _ in 0..cmp::min(n, s2_len) {
        *s1.offset(idx as isize) = s2_slice[idx] as c_char;
        idx += 1;
    }

    // if length of s2 < n, pad s1 with zeroes
    for _ in cmp::min(n, s2_len)..n {
        *s1.offset(idx as isize) = 0;
        idx += 1;
    }

    s1
}

#[no_mangle]
pub unsafe extern "C" fn strpbrk(s1: *const c_char, s2: *const c_char) -> *mut c_char {
    let p = s1.offset(strcspn(s1, s2) as isize);
    if *p != 0 {
        p as *mut c_char
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn strrchr(s: *const c_char, c: c_int) -> *mut c_char {
    let len = strlen(s) as isize;
    let c = c as i8;
    let mut i = len - 1;
    while i >= 0 {
        if *s.offset(i) == c {
            return s.offset(i) as *mut c_char;
        }
        i -= 1;
    }
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn strspn(s1: *const c_char, s2: *const c_char) -> size_t {
    use core::mem;

    let s1 = s1 as *const u8;
    let s2 = s2 as *const u8;

    // The below logic is effectively ripped from the musl implementation

    let mut byteset = [0usize; 32 / mem::size_of::<usize>()];

    let mut i = 0;
    while *s2.offset(i) != 0 {
        byteset[(*s2.offset(i) as usize) / (8 * byteset.len())] |=
            1 << (*s2.offset(i) as usize % (8 * byteset.len()));
        i += 1;
    }

    i = 0; // reset
    while *s1.offset(i) != 0 {
        if byteset[(*s1.offset(i) as usize) / (8 * byteset.len())]
            & 1 << (*s1.offset(i) as usize % (8 * byteset.len())) < 1
        {
            break;
        }
        i += 1;
    }
    i as size_t
}

#[no_mangle]
pub unsafe extern "C" fn strstr(s1: *const c_char, s2: *const c_char) -> *mut c_char {
    let mut i = 0;
    while *s1.offset(i) != 0 {
        let mut j = 0;
        while *s2.offset(j) != 0 && *s1.offset(j + i) != 0 {
            if *s2.offset(j) != *s1.offset(j + i) {
                break;
            }
            j += 1;
            if *s2.offset(j) == 0 {
                return s1.offset(i) as *mut c_char;
            }
        }
        i += 1;
    }
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn strtok(s1: *mut c_char, delimiter: *const c_char) -> *mut c_char {
    static mut HAYSTACK: *mut c_char = ptr::null_mut();
    unsafe {
        return strtok_r(s1, delimiter, &mut HAYSTACK);
    }
}

#[no_mangle]
pub extern "C" fn strtok_r(
    s: *mut c_char,
    delimiter: *const c_char,
    lasts: *mut *mut c_char,
) -> *mut c_char {
    // Loosely based on GLIBC implementation
    unsafe {
        let mut haystack = s;
        if haystack.is_null() {
            if (*lasts).is_null() {
                return ptr::null_mut();
            }
            haystack = *lasts;
        }

        // Skip past any extra delimiter left over from previous call
        haystack = haystack.add(strspn(haystack, delimiter));
        if *haystack == 0 {
            *lasts = ptr::null_mut();
            return ptr::null_mut();
        }

        // Build token by injecting null byte into delimiter
        let token = haystack;
        haystack = strpbrk(token, delimiter);
        if !haystack.is_null() {
            haystack.write(0);
            haystack = haystack.add(1);
            *lasts = haystack;
        } else {
            *lasts = ptr::null_mut();
        }

        return token;
    }
}

#[no_mangle]
pub extern "C" fn strxfrm(s1: *mut c_char, s2: *const c_char, n: usize) -> size_t {
    unimplemented!();
}
