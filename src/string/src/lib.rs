//! string implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/string.h.html

#![feature(compiler_builtins_lib)]
#![no_std]

extern crate compiler_builtins;
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
    use compiler_builtins::mem::memcpy;
    let dest = dest as *mut u8;
    let to = memchr(src, c, n);
    if to.is_null() {
        return to;
    }
    let src = src as *mut u8;
    let dist = (to as usize) - (src as usize);
    if memcpy(dest, src, dist).is_null() {
        return ptr::null_mut();
    }
    dest.offset(dist as isize + 1) as *mut c_void
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

// #[no_mangle]
// pub extern "C" fn memcmp(
//     s1: *const c_void,
//     s2: *const c_void,
//     n: usize,
// ) -> c_int {
//     unimplemented!();
// }

// #[no_mangle]
// pub extern "C" fn memcpy(
//     s1: *mut c_void,
//     s2: *const c_void,
//     n: usize,
// ) -> *mut c_void {
//     unimplemented!();
// }

// #[no_mangle]
// pub extern "C" fn memmove(
//     s1: *mut c_void,
//     s2: *const c_void,
//     n: usize,
// ) -> *mut c_void {
//     unimplemented!();
// }

// #[no_mangle]
// pub extern "C" fn memset(
//     s: *mut c_void,
//     c: c_int,
//     n: usize,
// ) -> *mut c_void {
//     unimplemented!();
// }

#[no_mangle]
pub unsafe extern "C" fn strcat(s1: *mut c_char, s2: *const c_char) -> *mut c_char {
    strncat(s1, s2, usize::MAX)
}

#[no_mangle]
pub unsafe extern "C" fn strchr(s: *const c_char, c: c_int) -> *mut c_char {
    let c = c as i8;
    let mut i = 0;
    loop {
        let cur = *s.offset(i);
        if cur == c {
            return s.offset(i) as *mut c_char;
        }
        if cur == 0 {
            return ptr::null_mut();
        }
        i += 1;
    }
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
pub unsafe extern "C" fn strcspn(s1: *const c_char, s2: *const c_char) -> c_ulong {
    use core::mem;

    let s1 = s1 as *const u8;
    let s2 = s2 as *const u8;
    
    // The below logic is effectively ripped from the musl implementation

    let mut byteset = [0u8; 32/mem::size_of::<usize>()]; 

    let mut i = 0;
    while *s2.offset(i) != 0 {
        byteset[(*s2.offset(i) as usize)/(8*mem::size_of::<usize>())] |= 1 << (*s2.offset(i) as usize % (8*mem::size_of::<usize>()));
        i += 1;
    }

    i = 0; // reset
    while *s2.offset(i) != 0 {
        if byteset[(*s2.offset(i) as usize)/(8*mem::size_of::<usize>())] & 1 << (*s2.offset(i) as usize % (8*mem::size_of::<usize>())) > 0 {
            break;
        }
        i += 1;
    }
    i as u64
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
pub extern "C" fn strpbrk(s1: *const c_char, s2: *const c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strrchr(s: *const c_char, c: c_int) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn strspn(s1: *const c_char, s2: *const c_char) -> c_ulong {
    use core::mem;

    let s1 = s1 as *const u8;
    let s2 = s2 as *const u8;
    
    // The below logic is effectively ripped from the musl implementation

    let mut byteset = [0u8; 32/mem::size_of::<usize>()]; 

    let mut i = 0;
    while *s2.offset(i) != 0 {
        byteset[(*s2.offset(i) as usize)/(8*mem::size_of::<usize>())] |= 1 << (*s2.offset(i) as usize % (8*mem::size_of::<usize>()));
        i += 1;
    }

    i = 0; // reset
    while *s2.offset(i) != 0 {
        if byteset[(*s2.offset(i) as usize)/(8*mem::size_of::<usize>())] & 1 << (*s2.offset(i) as usize % (8*mem::size_of::<usize>())) < 1 {
            break;
        }
        i += 1;
    }
    i as u64
}

#[no_mangle]
pub extern "C" fn strstr(s1: *const c_char, s2: *const c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strtok(s1: *mut c_char, s2: *const c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strtok_r(
    s: *mut c_char,
    sep: *const c_char,
    lasts: *mut *mut c_char,
) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strxfrm(s1: *mut c_char, s2: *const c_char, n: usize) -> c_ulong {
    unimplemented!();
}

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
