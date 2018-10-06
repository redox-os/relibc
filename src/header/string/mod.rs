//! string implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/string.h.html

use core::mem;
use core::ptr;
use core::usize;

use cbitset::BitSet256;

use header::errno::*;
use header::signal;
use platform;
use platform::types::*;

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
    let mut s = s as *const u8;
    let c = c as u8;
    let mut n = n;
    // read one byte at a time until s is aligned
    while s as usize % mem::size_of::<usize>() != 0 {
        if n == 0 {
            return ptr::null_mut();
        }
        if *s == c {
            return s as *mut c_void;
        }
        n -= 1;
        s = s.offset(1);
    }
    let mut s = s as *const usize;
    let lowbits = !0 as usize / 255;
    let highbits = lowbits * 0x80;
    let repeated_c = lowbits * c as usize;
    while n >= mem::size_of::<usize>() {
        // read multiple bytes at a time
        // turn the requested byte into 8 zero bits
        let m = *s ^ repeated_c;
        // subtracting one from zero flips high bit from 0 to 1
        if (m.wrapping_sub(lowbits) & !m & highbits) != 0 {
            break;
        }
        n -= mem::size_of::<usize>();
        s = s.offset(1);
    }
    let mut s = s as *const u8;
    while n > 0 {
        if *s == c {
            return s as *mut c_void;
        }
        n -= 1;
        s = s.offset(1);
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
    let mut i = 0;
    while i + 7 < n {
        *(s1.offset(i as isize) as *mut u64) = *(s2.offset(i as isize) as *const u64);
        i += 8;
    }
    while i < n {
        *(s1 as *mut u8).offset(i as isize) = *(s2 as *const u8).offset(i as isize);
        i += 1;
    }
    s1
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
pub unsafe extern "C" fn strchr(mut s: *const c_char, c: c_int) -> *mut c_char {
    let c = c as c_char;
    while *s != 0 {
        if *s == c {
            return s as *mut c_char;
        }
        s = s.offset(1);
    }
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int {
    strncmp(s1, s2, usize::MAX)
}

#[no_mangle]
pub unsafe extern "C" fn strcoll(s1: *const c_char, s2: *const c_char) -> c_int {
    // relibc has no locale stuff (yet)
    strcmp(s1, s2)
}

#[no_mangle]
pub unsafe extern "C" fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char {
    let mut i = 0;

    loop {
        let byte = *src.offset(i);
        *dst.offset(i) = byte;

        if byte == 0 {
            break;
        }

        i += 1;
    }

    dst
}

pub unsafe fn inner_strspn(s1: *const c_char, s2: *const c_char, cmp: bool) -> size_t {
    let mut s1 = s1 as *const u8;
    let mut s2 = s2 as *const u8;

    // The below logic is effectively ripped from the musl implementation. It
    // works by placing each byte as it's own bit in an array of numbers. Each
    // number can hold up to 8 * mem::size_of::<usize>() bits. We need 256 bits
    // in total, to fit one byte.

    let mut set = BitSet256::new();

    while *s2 != 0 {
        set.insert(*s2 as usize);
        s2 = s2.offset(1);
    }

    let mut i = 0;
    while *s1 != 0 {
        if set.contains(*s1 as usize) != cmp {
            break;
        }
        i += 1;
        s1 = s1.offset(1);
    }
    i
}

#[no_mangle]
pub unsafe extern "C" fn strcspn(s1: *const c_char, s2: *const c_char) -> size_t {
    inner_strspn(s1, s2, false)
}

#[no_mangle]
pub unsafe extern "C" fn strdup(s1: *const c_char) -> *mut c_char {
    strndup(s1, usize::MAX)
}

#[no_mangle]
pub unsafe extern "C" fn strndup(s1: *const c_char, size: usize) -> *mut c_char {
    let len = strnlen(s1, size);

    // the "+ 1" is to account for the NUL byte
    let buffer = platform::alloc(len + 1) as *mut c_char;
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
    let mut i = 0;
    while i < size {
        if *s.offset(i as isize) == 0 {
            break;
        }
        i += 1;
    }
    i as size_t
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
pub unsafe extern "C" fn strncpy(dst: *mut c_char, src: *const c_char, n: usize) -> *mut c_char {
    let mut i = 0;

    while *src.offset(i) != 0 && (i as usize) < n {
        *dst.offset(i) = *src.offset(i);
        i += 1;
    }

    for i in i..n as isize {
        *dst.offset(i) = 0;
    }

    dst
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
pub unsafe extern "C" fn strsignal(sig: c_int) -> *mut c_char {
    // Mutating this is undefined behavior I believe. But I just can't create a
    // &'static mut str. Alternative is allocating everything on the heap...
    signal::_signal_strings[sig as usize].as_ptr() as *const c_char as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn strspn(s1: *const c_char, s2: *const c_char) -> size_t {
    inner_strspn(s1, s2, true)
}

unsafe fn inner_strstr(mut haystack: *const c_char, mut needle: *const c_char, mask: c_char) -> *mut c_char {
    while *haystack != 0 {
        let mut i = 0;
        loop {
            if *needle.offset(i) == 0 {
                // We reached the end of the needle, everything matches this far
                return haystack as *mut c_char;
            }
            if *haystack.offset(i) & mask != *needle.offset(i) & mask {
                break;
            }

            i += 1;
        }

        haystack = haystack.offset(1);
    }
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char {
    inner_strstr(haystack, needle, !0)
}
#[no_mangle]
pub unsafe extern "C" fn strcasestr(haystack: *const c_char, needle: *const c_char) -> *mut c_char {
    inner_strstr(haystack, needle, !32)
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
pub unsafe extern "C" fn strxfrm(s1: *mut c_char, s2: *const c_char, n: usize) -> size_t {
    // relibc has no locale stuff (yet)
    let len = strlen(s2);
    if len < n {
        strcpy(s1, s2);
    }
    len
}
