//! string implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/string.h.html

use core::{mem, ptr, slice, usize};

use cbitset::BitSet256;

use crate::{
    header::{errno::*, signal},
    platform::{self, types::*},
};

#[no_mangle]
pub unsafe extern "C" fn memccpy(
    dest: *mut c_void,
    src: *const c_void,
    c: c_int,
    n: size_t,
) -> *mut c_void {
    let to = memchr(src, c, n);
    if to.is_null() {
        return to;
    }
    let dist = (to as usize) - (src as usize);
    if memcpy(dest, src, dist).is_null() {
        return ptr::null_mut();
    }
    (dest as *mut u8).add(dist + 1) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn memchr(
    haystack: *const c_void,
    needle: c_int,
    len: size_t,
) -> *mut c_void {
    let haystack = slice::from_raw_parts(haystack as *const u8, len as usize);

    match memchr::memchr(needle as u8, haystack) {
        Some(index) => haystack[index..].as_ptr() as *mut c_void,
        None => ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const c_void, s2: *const c_void, n: size_t) -> c_int {
    let (div, rem) = (n / mem::size_of::<usize>(), n % mem::size_of::<usize>());
    let mut a = s1 as *const usize;
    let mut b = s2 as *const usize;
    for _ in 0..div {
        if *a != *b {
            for i in 0..mem::size_of::<usize>() {
                let c = *(a as *const u8).add(i);
                let d = *(b as *const u8).add(i);
                if c != d {
                    return c as c_int - d as c_int;
                }
            }
            unreachable!()
        }
        a = a.offset(1);
        b = b.offset(1);
    }

    let mut a = a as *const u8;
    let mut b = b as *const u8;
    for _ in 0..rem {
        if *a != *b {
            return *a as c_int - *b as c_int;
        }
        a = a.offset(1);
        b = b.offset(1);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(s1: *mut c_void, s2: *const c_void, n: size_t) -> *mut c_void {
    let mut i = 0;
    while i + 7 < n {
        *(s1.add(i) as *mut u64) = *(s2.add(i) as *const u64);
        i += 8;
    }
    while i < n {
        *(s1 as *mut u8).add(i) = *(s2 as *const u8).add(i);
        i += 1;
    }
    s1
}

#[no_mangle]
pub unsafe extern "C" fn memmove(s1: *mut c_void, s2: *const c_void, n: size_t) -> *mut c_void {
    if s2 < s1 as *const c_void {
        // copy from end
        let mut i = n;
        while i != 0 {
            i -= 1;
            *(s1 as *mut u8).add(i) = *(s2 as *const u8).add(i);
        }
    } else {
        // copy from beginning
        let mut i = 0;
        while i < n {
            *(s1 as *mut u8).add(i) = *(s2 as *const u8).add(i);
            i += 1;
        }
    }
    s1
}

#[no_mangle]
pub unsafe extern "C" fn memrchr(
    haystack: *const c_void,
    needle: c_int,
    len: size_t,
) -> *mut c_void {
    let haystack = slice::from_raw_parts(haystack as *const u8, len as usize);

    match memchr::memrchr(needle as u8, haystack) {
        Some(index) => haystack[index..].as_ptr() as *mut c_void,
        None => ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut c_void, c: c_int, n: size_t) -> *mut c_void {
    for i in 0..n {
        *(s as *mut u8).add(i) = c as u8;
    }
    s
}

#[cfg(target_feature = "sse2")]
fn rawmemchr2(n1: u8, n2: u8, haystack: *const u8) -> usize {
    use core::arch::x86_64::*;

    unsafe fn forward_search2(
        start_ptr: *const u8,
        ptr: *const u8,
        vn1: __m128i,
        vn2: __m128i,
    ) -> Option<usize> {
        debug_assert!(start_ptr <= ptr);

        let chunk = _mm_loadu_si128(ptr as *const __m128i);
        let eq1 = _mm_cmpeq_epi8(chunk, vn1);
        let eq2 = _mm_cmpeq_epi8(chunk, vn2);
        if _mm_movemask_epi8(_mm_or_si128(eq1, eq2)) != 0 {
            let mask1 = _mm_movemask_epi8(eq1);
            let mask2 = _mm_movemask_epi8(eq2);
            Some((ptr as usize - start_ptr as usize) + forward_pos2(mask1, mask2))
        } else {
            None
        }
    }

    fn forward_pos2(mask1: i32, mask2: i32) -> usize {
        debug_assert!(mask1 != 0 || mask2 != 0);

        (mask1 | mask2).trailing_zeros() as usize
    }

    const VECTOR_SIZE: usize = mem::size_of::<__m128i>();
    const VECTOR_ALIGN: usize = VECTOR_SIZE - 1;
    const LOOP_SIZE2: usize = 2 * VECTOR_SIZE;

    unsafe {
        let vn1 = _mm_set1_epi8(n1 as i8);
        let vn2 = _mm_set1_epi8(n2 as i8);
        let loop_size = LOOP_SIZE2;
        let start_ptr = haystack;
        let mut ptr = start_ptr;

        if let Some(i) = forward_search2(start_ptr, ptr, vn1, vn2) {
            return i;
    }

        ptr = ptr.add(VECTOR_SIZE - (start_ptr as usize & VECTOR_ALIGN));
        debug_assert!(ptr > start_ptr);
        while loop_size == LOOP_SIZE2 {
            debug_assert_eq!(0, (ptr as usize) % VECTOR_SIZE);

            let a = _mm_load_si128(ptr as *const __m128i);
            let b = _mm_load_si128(ptr.add(VECTOR_SIZE) as *const __m128i);
            let eqa1 = _mm_cmpeq_epi8(vn1, a);
            let eqb1 = _mm_cmpeq_epi8(vn1, b);
            let eqa2 = _mm_cmpeq_epi8(vn2, a);
            let eqb2 = _mm_cmpeq_epi8(vn2, b);
            let or1 = _mm_or_si128(eqa1, eqb1);
            let or2 = _mm_or_si128(eqa2, eqb2);
            let or3 = _mm_or_si128(or1, or2);
            if _mm_movemask_epi8(or3) != 0 {
                let mut at = (ptr as usize - start_ptr as usize);
                let mask1 = _mm_movemask_epi8(eqa1);
                let mask2 = _mm_movemask_epi8(eqa2);
                if mask1 != 0 || mask2 != 0 {
                    return at + forward_pos2(mask1, mask2);
                }

                at += VECTOR_SIZE;
                let mask1 = _mm_movemask_epi8(eqb1);
                let mask2 = _mm_movemask_epi8(eqb2);
                return at + forward_pos2(mask1, mask2);
            }
            ptr = ptr.add(loop_size);
        }
        loop {
            if let Some(i) = forward_search2(start_ptr, ptr, vn1, vn2) {
                return i;
            }
            ptr = ptr.add(VECTOR_SIZE);
        }
    }
}

#[cfg(not(target_feature = "sse2"))]
fn rawmemchr2(n1: u8, n2: u8, haystack: *const u8) -> usize {
    #[inline(always)]
    fn contains_zero_byte(x: usize) -> bool {
        const LO_U64: u64 = 0x0101010101010101;
        const HI_U64: u64 = 0x8080808080808080;

        const LO_USIZE: usize = LO_U64 as usize;
        const HI_USIZE: usize = HI_U64 as usize;

        x.wrapping_sub(LO_USIZE) & !x & HI_USIZE != 0
    }

    #[inline(always)]
    fn repeat_byte(b: u8) -> usize {
        (b as usize) * (usize::MAX / 255)
    }

    #[inline(always)]
    unsafe fn forward_search<F: Fn(u8) -> bool>(
        start_ptr: *const u8,
        mut ptr: *const u8,
        confirm: F,
    ) -> usize {
        debug_assert!(start_ptr <= ptr);

        loop {
            if confirm(*ptr) {
                return ptr as usize - start_ptr as usize;
            }
            ptr = ptr.offset(1);
        }
    }

    unsafe fn ptr_add(ptr: *const u8, amt: usize) -> *const u8 {
        debug_assert!(amt < ::core::isize::MAX as usize);
        ptr.offset(amt as isize)
    }

    unsafe fn read_unaligned_usize(ptr: *const u8) -> usize {
        let mut n: usize = 0;
        core::ptr::copy_nonoverlapping(ptr, &mut n as *mut _ as *mut u8, USIZE_BYTES);
        n
    }

    #[cfg(target_pointer_width = "32")]
    const USIZE_BYTES: usize = 4;

    #[cfg(target_pointer_width = "64")]
    const USIZE_BYTES: usize = 8;

    let vn1 = repeat_byte(n1);
    let vn2 = repeat_byte(n2);
    let confirm = |byte| byte == n1 || byte == n2;
    let align = USIZE_BYTES - 1;
    let start_ptr = haystack;
    let mut ptr = start_ptr;

    unsafe {
        let chunk = read_unaligned_usize(ptr);
        let eq1 = contains_zero_byte(chunk ^ vn1);
        let eq2 = contains_zero_byte(chunk ^ vn2);
        if eq1 || eq2 {
            return forward_search(start_ptr, ptr, confirm);
        }

        ptr = ptr_add(ptr, USIZE_BYTES - (start_ptr as usize & align));
        debug_assert!(ptr > start_ptr);
        loop {
            debug_assert_eq!(0, (ptr as usize) % USIZE_BYTES);

            let chunk = *(ptr as *const usize);
            let eq1 = contains_zero_byte(chunk ^ vn1);
            let eq2 = contains_zero_byte(chunk ^ vn2);
            if eq1 || eq2 {
                break;
            }
            ptr = ptr_add(ptr, USIZE_BYTES);
        }
        forward_search(start_ptr, ptr, confirm)
    }
}

#[no_mangle]
pub unsafe extern "C" fn strchr(s: *const c_char, c: c_int) -> *mut c_char {
    let index = rawmemchr2(c as u8, 0u8, s as *const u8);
    if *s.offset(index) != 0 {
        s.offset(index) as *mut c_char
    }
    else {
        ptr::null_mut()
    }
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
pub unsafe extern "C" fn strndup(s1: *const c_char, size: size_t) -> *mut c_char {
    let len = strnlen(s1, size);

    // the "+ 1" is to account for the NUL byte
    let buffer = platform::alloc(len + 1) as *mut c_char;
    if buffer.is_null() {
        platform::errno = ENOMEM as c_int;
    } else {
        //memcpy(buffer, s1, len)
        for i in 0..len {
            *buffer.add(i) = *s1.add(i);
        }
        *buffer.add(len) = 0;
    }

    buffer
}

#[no_mangle]
pub unsafe extern "C" fn strerror(errnum: c_int) -> *mut c_char {
    use core::fmt::Write;

    static mut strerror_buf: [u8; 256] = [0; 256];

    let mut w = platform::StringWriter(strerror_buf.as_mut_ptr(), strerror_buf.len());

    if errnum >= 0 && errnum < STR_ERROR.len() as c_int {
        let _ = w.write_str(STR_ERROR[errnum as usize]);
    } else {
        let _ = w.write_fmt(format_args!("Unknown error {}", errnum));
    }

    strerror_buf.as_mut_ptr() as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn strerror_r(errnum: c_int, buf: *mut c_char, buflen: size_t) -> c_int {
    let msg = strerror(errnum);
    let len = strlen(msg);

    if len >= buflen {
        if buflen != 0 {
            memcpy(buf as *mut c_void, msg as *const c_void, buflen - 1);
            *buf.add(buflen - 1) = 0;
        }
        return ERANGE as c_int;
    }
    memcpy(buf as *mut c_void, msg as *const c_void, len + 1);

    0
}

#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const c_char) -> size_t {
    rawmemchr2(0u8, 0u8, s as *const u8) as size_t
}

#[no_mangle]
pub unsafe extern "C" fn strnlen(s: *const c_char, size: size_t) -> size_t {
    let haystack = slice::from_raw_parts(s as *const u8, size as usize);

    match memchr::memchr(0u8, haystack) {
        Some(index) => index as size_t,
        None => size,
    }
}

#[no_mangle]
pub unsafe extern "C" fn strnlen_s(s: *const c_char, size: size_t) -> size_t {
    if s.is_null() {
        0
    } else {
        strnlen(s, size)
    }
}

#[no_mangle]
pub unsafe extern "C" fn strcat(s1: *mut c_char, s2: *const c_char) -> *mut c_char {
    strncat(s1, s2, usize::MAX)
}

#[no_mangle]
pub unsafe extern "C" fn strncat(s1: *mut c_char, s2: *const c_char, n: size_t) -> *mut c_char {
    let len = strlen(s1 as *const c_char);
    let mut i = 0;
    while i < n {
        let b = *s2.add(i);
        if b == 0 {
            break;
        }

        *s1.add(len + i) = b;
        i += 1;
    }
    *s1.add(len + i) = 0;

    s1
}

#[no_mangle]
pub unsafe extern "C" fn strncmp(s1: *const c_char, s2: *const c_char, n: size_t) -> c_int {
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
pub unsafe extern "C" fn strncpy(dst: *mut c_char, src: *const c_char, n: size_t) -> *mut c_char {
    let mut i = 0;

    while *src.add(i) != 0 && i < n {
        *dst.add(i) = *src.add(i);
        i += 1;
    }

    for i in i..n {
        *dst.add(i) = 0;
    }

    dst
}

#[no_mangle]
pub unsafe extern "C" fn strpbrk(s1: *const c_char, s2: *const c_char) -> *mut c_char {
    let p = s1.add(strcspn(s1, s2));
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
pub unsafe extern "C" fn strsignal(sig: c_int) -> *const c_char {
    signal::_signal_strings
        .get(sig as usize)
        .unwrap_or(&signal::_signal_strings[0]) // Unknown signal message
        .as_ptr() as *const c_char
}

#[no_mangle]
pub unsafe extern "C" fn strspn(s1: *const c_char, s2: *const c_char) -> size_t {
    inner_strspn(s1, s2, true)
}

unsafe fn inner_strstr(
    mut haystack: *const c_char,
    needle: *const c_char,
    mask: c_char,
) -> *mut c_char {
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
pub unsafe extern "C" fn strtok(s1: *mut c_char, delimiter: *const c_char) -> *mut c_char {
    static mut HAYSTACK: *mut c_char = ptr::null_mut();
    strtok_r(s1, delimiter, &mut HAYSTACK)
}

#[no_mangle]
pub unsafe extern "C" fn strtok_r(
    s: *mut c_char,
    delimiter: *const c_char,
    lasts: *mut *mut c_char,
) -> *mut c_char {
    // Loosely based on GLIBC implementation
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

    token
}

#[no_mangle]
pub unsafe extern "C" fn strxfrm(s1: *mut c_char, s2: *const c_char, n: size_t) -> size_t {
    // relibc has no locale stuff (yet)
    let len = strlen(s2);
    if len < n {
        strcpy(s1, s2);
    }
    len
}
