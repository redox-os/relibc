//! `string.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/string.h.html>.

use core::{
    iter::once,
    mem::{self, MaybeUninit},
    ptr, slice, usize,
};

use cbitset::BitSet256;

use crate::{
    header::{errno::*, signal},
    iter::{NulTerminated, NulTerminatedInclusive, SrcDstPtrIter},
    platform::{
        self,
        types::{c_char, c_int, c_void, size_t},
    },
    raw_cell::RawCell,
};

use super::locale::{THREAD_LOCALE, locale_t};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/memccpy.html>.
///
/// # Safety
/// The caller must ensure that:
/// - `n` is not longer than the memory area pointed to by `s1`, and
/// - `n` is not longer than the memory area pointed to by `s2`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memccpy(
    s1: *mut c_void,
    s2: *const c_void,
    c: c_int,
    n: size_t,
) -> *mut c_void {
    let to = unsafe { memchr(s2, c, n) };
    let dist = if to.is_null() {
        n
    } else {
        ((to as usize) - (s2 as usize)) + 1
    };
    unsafe { memcpy(s1, s2, dist) };
    if to.is_null() {
        ptr::null_mut()
    } else {
        unsafe { (s1 as *mut u8).add(dist) as *mut c_void }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/memchr.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memchr(
    haystack: *const c_void,
    needle: c_int,
    len: size_t,
) -> *mut c_void {
    let haystack = unsafe { slice::from_raw_parts(haystack as *const u8, len as usize) };

    match memchr::memchr(needle as u8, haystack) {
        Some(index) => haystack[index..].as_ptr() as *mut c_void,
        None => ptr::null_mut(),
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/memcmp.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> c_int {
    let (div, rem) = (n / mem::size_of::<usize>(), n % mem::size_of::<usize>());
    let mut a = s1.cast::<usize>();
    let mut b = s2.cast::<usize>();
    for _ in 0..div {
        // SAFETY: `s1` and `s2` are `*const c_void`, which only guarantees byte
        // alignment. Hence `a` and `b` may be unaligned.
        if unsafe { a.read_unaligned() } != unsafe { b.read_unaligned() } {
            for i in 0..mem::size_of::<usize>() {
                let c = unsafe { *(a.cast::<u8>()).add(i) };
                let d = unsafe { *(b.cast::<u8>()).add(i) };
                if c != d {
                    return c as c_int - d as c_int;
                }
            }
            unreachable!()
        }
        a = unsafe { a.offset(1) };
        b = unsafe { b.offset(1) };
    }

    let mut a = a.cast::<u8>();
    let mut b = b.cast::<u8>();
    for _ in 0..rem {
        if unsafe { *a } != unsafe { *b } {
            return unsafe { *a } as c_int - unsafe { *b } as c_int;
        }
        a = unsafe { a.offset(1) };
        b = unsafe { b.offset(1) };
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/memcpy.html>.
///
/// # Safety
/// The caller must ensure that *either*:
/// - `n` is 0, *or*
///     - `s1` is convertible to a `&mut [MaybeUninit<u8>]` with length `n`,
///       and
///     - `s2` is convertible to a `&[MaybeUninit<u8>]` with length `n`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(s1: *mut c_void, s2: *const c_void, n: size_t) -> *mut c_void {
    for i in 0..n {
        unsafe { *(s1 as *mut u8).add(i) = *(s2 as *const u8).add(i) };
    }
    s1
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/memmem.html>.
///
/// # Safety
/// The caller must ensure that:
/// - `haystack` is convertible to a `&[u8]` with length `haystacklen`, and
/// - `needle` is convertible to a `&[u8]` with length `needlelen`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmem(
    haystack: *const c_void,
    haystacklen: size_t,
    needle: *const c_void,
    needlelen: size_t,
) -> *mut c_void {
    match needlelen {
        // Required to satisfy spec (would otherwise cause .windows() to panic)
        0 => haystack,
        _ => {
            // SAFETY: the caller is required to ensure that the provided
            // pointers are valid.
            let haystack_slice =
                unsafe { slice::from_raw_parts(haystack.cast::<u8>(), haystacklen) };
            let needle_slice = unsafe { slice::from_raw_parts(needle.cast::<u8>(), needlelen) };

            // At this point, .windows() will receive a nonzero `needlelen` and
            // thus not panic.
            match haystack_slice
                .windows(needlelen)
                .find(|&haystack_window| haystack_window == needle_slice)
            {
                Some(match_slice) => match_slice.as_ptr().cast(),
                None => ptr::null(),
            }
        }
    }
    .cast_mut()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/memmove.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(s1: *mut c_void, s2: *const c_void, n: size_t) -> *mut c_void {
    if s2 < s1 as *const c_void {
        // copy from end
        let mut i = n;
        while i != 0 {
            i -= 1;
            unsafe { *(s1 as *mut u8).add(i) = *(s2 as *const u8).add(i) };
        }
    } else {
        // copy from beginning
        let mut i = 0;
        while i < n {
            unsafe { *(s1 as *mut u8).add(i) = *(s2 as *const u8).add(i) };
            i += 1;
        }
    }
    s1
}

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/memchr.3.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memrchr(
    haystack: *const c_void,
    needle: c_int,
    len: size_t,
) -> *mut c_void {
    let haystack = unsafe { slice::from_raw_parts(haystack as *const u8, len as usize) };

    match memchr::memrchr(needle as u8, haystack) {
        Some(index) => haystack[index..].as_ptr() as *mut c_void,
        None => ptr::null_mut(),
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/memset.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(s: *mut c_void, c: c_int, n: size_t) -> *mut c_void {
    for i in 0..n {
        unsafe { *(s as *mut u8).add(i) = c as u8 };
    }
    s
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strcpy.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stpcpy(mut s1: *mut c_char, mut s2: *const c_char) -> *mut c_char {
    loop {
        unsafe { *s1 = *s2 };

        if unsafe { *s1 } == 0 {
            break;
        }

        s1 = unsafe { s1.add(1) };
        s2 = unsafe { s2.add(1) };
    }

    s1
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strncpy.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stpncpy(
    mut s1: *mut c_char,
    mut s2: *const c_char,
    mut n: size_t,
) -> *mut c_char {
    while n > 0 {
        unsafe { *s1 = *s2 };

        if unsafe { *s1 } == 0 {
            break;
        }

        n -= 1;
        s1 = unsafe { s1.add(1) };
        s2 = unsafe { s2.add(1) };
    }

    unsafe { memset(s1.cast(), 0, n) };

    s1
}

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/strstr.3.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcasestr(haystack: *const c_char, needle: *const c_char) -> *mut c_char {
    unsafe { inner_strstr(haystack, needle, !32) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strcat.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcat(s1: *mut c_char, s2: *const c_char) -> *mut c_char {
    unsafe { strncat(s1, s2, usize::MAX) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strchr.html>.
///
/// # Safety
/// The caller is required to ensure that `s` is a valid pointer to a buffer
/// containing at least one nul value. The pointed-to buffer must not be
/// modified for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strchr(mut s: *const c_char, c: c_int) -> *mut c_char {
    let c_as_c_char = c as c_char;

    // We iterate over non-mut references and thus need to coerce the
    // resulting reference via a *const pointer before we can get our *mut.
    // SAFETY: the caller is required to ensure that s points to a valid
    // nul-terminated buffer.
    let ptr: *const c_char =
        match unsafe { NulTerminatedInclusive::new(s) }.find(|&&sc| sc == c_as_c_char) {
            Some(sc_ref) => sc_ref,
            None => ptr::null(),
        };
    ptr.cast_mut()
}

/// Non-POSIX, see <https://man7.org/linux/man-pages/man3/strchr.3.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strchrnul(s: *const c_char, c: c_int) -> *mut c_char {
    let mut s = s.cast_mut();
    loop {
        if unsafe { *s } == c as _ {
            break;
        }
        if unsafe { *s } == 0 {
            break;
        }
        s = unsafe { s.add(1) };
    }
    s
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strcmp.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int {
    unsafe { strncmp(s1, s2, usize::MAX) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strcoll_l.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcoll_l(s1: *const c_char, s2: *const c_char, _loc: locale_t) -> c_int {
    // relibc has no locale stuff (yet)
    unsafe { strcmp(s1, s2) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strcoll.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcoll(s1: *const c_char, s2: *const c_char) -> c_int {
    strcoll_l(s1, s2, THREAD_LOCALE as locale_t)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strcpy.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char {
    let src_iter = unsafe { NulTerminated::new(src).unwrap() };
    let src_dest_iter = unsafe { SrcDstPtrIter::new(src_iter.chain(once(&0)), dst) };
    for (src_item, dst_item) in src_dest_iter {
        dst_item.write(*src_item);
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

    while unsafe { *s2 } != 0 {
        set.insert(unsafe { *s2 } as usize);
        s2 = unsafe { s2.offset(1) };
    }

    let mut i = 0;
    while unsafe { *s1 } != 0 {
        if set.contains(unsafe { *s1 } as usize) != cmp {
            break;
        }
        i += 1;
        s1 = unsafe { s1.offset(1) };
    }
    i
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strcspn.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcspn(s1: *const c_char, s2: *const c_char) -> size_t {
    unsafe { inner_strspn(s1, s2, false) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strdup.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strdup(s1: *const c_char) -> *mut c_char {
    unsafe { strndup(s1, usize::MAX) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strerror_l.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strerror_l(errnum: c_int, _loc: locale_t) -> *mut c_char {
    use core::fmt::Write;

    static strerror_buf: RawCell<[u8; STRERROR_MAX]> = RawCell::new([0; STRERROR_MAX]);
    let mut w = platform::StringWriter(unsafe { strerror_buf.unsafe_mut().as_mut_ptr() }, unsafe {
        strerror_buf.unsafe_ref().len()
    });

    let _ = match STR_ERROR.get(errnum as usize) {
        Some(e) => w.write_str(e),
        None => w.write_fmt(format_args!("Unknown error {}", errnum)),
    };

    (unsafe { strerror_buf.unsafe_mut().as_mut_ptr() }) as *mut c_char
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strerror.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strerror(errnum: c_int) -> *mut c_char {
    strerror_l(errnum, THREAD_LOCALE as locale_t)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strerror_r.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strerror_r(errnum: c_int, buf: *mut c_char, buflen: size_t) -> c_int {
    let msg = unsafe { strerror(errnum) };
    let len = unsafe { strlen(msg) };

    if len >= buflen {
        if buflen != 0 {
            unsafe { memcpy(buf as *mut c_void, msg as *const c_void, buflen - 1) };
            unsafe { *buf.add(buflen - 1) = 0 };
        }
        return ERANGE as c_int;
    }
    unsafe { memcpy(buf as *mut c_void, msg as *const c_void, len + 1) };

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strlcat.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strlcat(dst: *mut c_char, src: *const c_char, dstsize: size_t) -> size_t {
    let dst_len = unsafe { strnlen(dst, dstsize) };
    let d = unsafe { dst.offset(dst_len as isize) };
    let src_len = unsafe { strlcpy(d, src, dstsize - dst_len) };
    src_len + if dst_len > dstsize { dstsize } else { dst_len }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strsep(str_: *mut *mut c_char, sep: *const c_char) -> *mut c_char {
    let s = unsafe { *str_ };
    if s.is_null() {
        return ptr::null_mut();
    }
    let mut end = unsafe { s.add(strcspn(s, sep)) };
    if unsafe { *end } != 0 {
        unsafe { *end = 0 };
        end = unsafe { end.add(1) };
    } else {
        end = ptr::null_mut();
    }
    unsafe { *str_ = end };
    s
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strlcat.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strlcpy(dst: *mut c_char, src: *const c_char, dstsize: size_t) -> size_t {
    let mut i = 0;

    if dstsize != 0 {
        while unsafe { *src.add(i) } != 0 && i < dstsize - 1 {
            unsafe {
                *dst.add(i) = *src.add(i);
            }
            i += 1;
        }
        unsafe {
            *dst.add(i) = 0;
        }
    }

    while unsafe { *src.add(i) } != 0 {
        i += 1;
    }

    i as size_t
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strlen.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strlen(s: *const c_char) -> size_t {
    unsafe { NulTerminated::new(s) }
        .map(|s| s.count())
        .unwrap_or(0)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strncat.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strncat(s1: *mut c_char, s2: *const c_char, n: size_t) -> *mut c_char {
    let len = unsafe { strlen(s1.cast()) };
    let mut i = 0;
    while i < n {
        let b = unsafe { *s2.add(i) };
        if b == 0 {
            break;
        }

        unsafe { *s1.add(len + i) = b };
        i += 1;
    }
    unsafe { *s1.add(len + i) = 0 };

    s1
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strncmp.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strncmp(s1: *const c_char, s2: *const c_char, n: size_t) -> c_int {
    for i in 0..n {
        // These must be cast as u8 to have correct comparisons
        let a = unsafe { *s1.add(i) } as u8;
        let b = unsafe { *s2.add(i) } as u8;
        if a != b || a == 0 {
            return (a as c_int) - (b as c_int);
        }
    }

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strncpy.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strncpy(s1: *mut c_char, s2: *const c_char, n: size_t) -> *mut c_char {
    unsafe { stpncpy(s1, s2, n) };
    s1
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strdup.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strndup(s1: *const c_char, size: size_t) -> *mut c_char {
    let len = unsafe { strnlen(s1, size) };

    // the "+ 1" is to account for the NUL byte
    let buffer = unsafe { platform::alloc(len + 1) } as *mut c_char;
    if buffer.is_null() {
        platform::ERRNO.set(ENOMEM as c_int);
    } else {
        //memcpy(buffer, s1, len)
        for i in 0..len {
            unsafe { *buffer.add(i) = *s1.add(i) };
        }
        unsafe { *buffer.add(len) = 0 };
    }

    buffer
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strlen.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strnlen(s: *const c_char, size: size_t) -> size_t {
    unsafe { NulTerminated::new(s).unwrap() }.take(size).count()
}

/// Non-POSIX, see <https://en.cppreference.com/w/c/string/byte/strlen>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strnlen_s(s: *const c_char, size: size_t) -> size_t {
    if s.is_null() {
        0
    } else {
        unsafe { strnlen(s, size) }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strpbrk.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strpbrk(s1: *const c_char, s2: *const c_char) -> *mut c_char {
    let p = unsafe { s1.add(strcspn(s1, s2)) };
    if unsafe { *p } != 0 {
        p as *mut c_char
    } else {
        ptr::null_mut()
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strrchr.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strrchr(s: *const c_char, c: c_int) -> *mut c_char {
    let len = unsafe { strlen(s) } as isize;
    let c = c as c_char;
    let mut i = len - 1;
    while i >= 0 {
        if unsafe { *s.offset(i) } == c {
            return unsafe { s.offset(i) } as *mut c_char;
        }
        i -= 1;
    }
    ptr::null_mut()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strsignal.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strsignal(sig: c_int) -> *mut c_char {
    signal::SIGNAL_STRINGS
        .get(sig as usize)
        .unwrap_or(&signal::SIGNAL_STRINGS[0]) // Unknown signal message
        .as_ptr() as *mut c_char
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strspn.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strspn(s1: *const c_char, s2: *const c_char) -> size_t {
    unsafe { inner_strspn(s1, s2, true) }
}

unsafe fn inner_strstr(
    mut haystack: *const c_char,
    needle: *const c_char,
    mask: c_char,
) -> *mut c_char {
    while unsafe { *haystack } != 0 {
        let mut i = 0;
        loop {
            if unsafe { *needle.offset(i) } == 0 {
                // We reached the end of the needle, everything matches this far
                return haystack as *mut c_char;
            }
            if unsafe { *haystack.offset(i) } & mask != unsafe { *needle.offset(i) } & mask {
                break;
            }

            i += 1;
        }

        haystack = unsafe { haystack.offset(1) };
    }
    ptr::null_mut()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strstr.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char {
    unsafe { inner_strstr(haystack, needle, !0) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strtok.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strtok(s1: *mut c_char, delimiter: *const c_char) -> *mut c_char {
    static mut HAYSTACK: *mut c_char = ptr::null_mut();
    unsafe { strtok_r(s1, delimiter, &raw mut HAYSTACK) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strtok.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strtok_r(
    s: *mut c_char,
    delimiter: *const c_char,
    lasts: *mut *mut c_char,
) -> *mut c_char {
    // musl returns null if both s and lasts are null, it sets s to lasts otherwise
    let mut haystack = s;
    if haystack.is_null() {
        if (unsafe { *lasts }).is_null() {
            return ptr::null_mut();
        }
        haystack = unsafe { *lasts };
    }

    // Skip past any extra delimiter left over from previous call
    haystack = unsafe { haystack.add(strspn(haystack, delimiter)) };
    if unsafe { *haystack } == 0 {
        unsafe { *lasts = haystack };
        return ptr::null_mut();
    }

    // Build token by injecting null byte into delimiter
    let token = haystack;
    haystack = unsafe { strpbrk(token, delimiter) };
    if !haystack.is_null() {
        unsafe { haystack.write(0) };
        haystack = unsafe { haystack.add(1) };
        unsafe { *lasts = haystack };
    } else {
        unsafe { *lasts = token.add(strlen(token)) };
    }

    token
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strxfrm_l.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strxfrm_l(
    s1: *mut c_char,
    s2: *const c_char,
    n: size_t,
    _loc: locale_t,
) -> size_t {
    // relibc has no locale stuff (yet)
    let len = unsafe { strlen(s2) };
    if len < n {
        unsafe { strcpy(s1, s2) };
    }
    len
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strxfrm.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strxfrm(s1: *mut c_char, s2: *const c_char, n: size_t) -> size_t {
    strxfrm_l(s1, s2, n, THREAD_LOCALE as locale_t)
}
