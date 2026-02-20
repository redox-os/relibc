//! `wchar.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/wchar.h.html>.

use core::{char, ffi::VaList as va_list, mem, ptr, slice, usize};

use crate::{
    c_str::WStr,
    header::{
        ctype::isspace,
        errno::{EILSEQ, ENOMEM, ERANGE},
        stdio::*,
        stdlib::{MB_CUR_MAX, MB_LEN_MAX, malloc},
        string,
        time::*,
        wchar::{lookaheadreader::LookAheadReader, utf8::get_char_encoded_length},
        wctype::*,
    },
    iter::{NulTerminated, NulTerminatedInclusive},
    platform::{
        self, ERRNO,
        types::{
            c_char, c_double, c_int, c_long, c_longlong, c_uchar, c_ulong, c_ulonglong, c_void,
            intmax_t, size_t, uintmax_t, wchar_t, wint_t,
        },
    },
};

mod lookaheadreader;
mod utf8;
mod wprintf;
mod wscanf;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/wchar.h.html>.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct mbstate_t;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/btowc.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn btowc(c: c_int) -> wint_t {
    //Check for EOF
    if c == EOF {
        return WEOF;
    }

    let uc = c as u8;
    let c = uc as c_char;
    let mut ps: mbstate_t = mbstate_t;
    let mut wc: wchar_t = 0;
    let saved_errno = platform::ERRNO.get();
    let status = unsafe { mbrtowc(&mut wc, ptr::from_ref::<c_char>(&c), 1, &mut ps) };
    if status == usize::MAX || status == usize::MAX - 1 {
        platform::ERRNO.set(saved_errno);
        return WEOF;
    }
    wc as wint_t
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fgetwc.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fgetwc(stream: *mut FILE) -> wint_t {
    // TODO: Process locale
    let mut buf: [c_uchar; MB_CUR_MAX as usize] = [0; MB_CUR_MAX as usize];
    let mut encoded_length = 0;
    let mut bytes_read = 0;
    let mut wc: wchar_t = 0;

    loop {
        let nread = unsafe {
            fread(
                buf[bytes_read..bytes_read + 1]
                    .as_mut_ptr()
                    .cast::<c_void>(),
                1,
                1,
                stream,
            )
        };

        if nread != 1 {
            ERRNO.set(EILSEQ);
            return WEOF;
        }

        bytes_read += 1;

        if bytes_read == 1 {
            encoded_length = if let Some(el) = get_char_encoded_length(buf[0]) {
                el
            } else {
                ERRNO.set(EILSEQ);
                return WEOF;
            };
        }

        if bytes_read >= encoded_length {
            break;
        }
    }

    unsafe {
        mbrtowc(
            &mut wc,
            buf.as_ptr().cast::<c_char>(),
            encoded_length,
            ptr::null_mut(),
        )
    };

    wc as wint_t
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fgetws.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fgetws(ws: *mut wchar_t, n: c_int, stream: *mut FILE) -> *mut wchar_t {
    //TODO: lock
    let mut i = 0;
    while ((i + 1) as c_int) < n {
        let wc = unsafe { fgetwc(stream) };
        if wc == WEOF {
            return ptr::null_mut();
        }
        unsafe { *ws.add(i) = wc as wchar_t };
        i += 1;
    }
    while (i as c_int) < n {
        unsafe { *ws.add(i) = 0 };
        i += 1;
    }
    ws
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fputwc.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fputwc(wc: wchar_t, stream: *mut FILE) -> wint_t {
    //Convert wchar_t to multibytes first
    static mut INTERNAL: mbstate_t = mbstate_t;
    let mut bytes: [c_char; MB_CUR_MAX as usize] = [0; MB_CUR_MAX as usize];

    let amount = unsafe { wcrtomb(bytes.as_mut_ptr(), wc, &raw mut INTERNAL) };

    for i in 0..amount {
        unsafe { fputc(c_int::from(bytes[i]), &mut *stream) };
    }

    wc as wint_t
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fputws.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fputws(ws: *const wchar_t, stream: *mut FILE) -> c_int {
    let mut i = 0;
    loop {
        let wc = unsafe { *ws.add(i) };
        if wc == 0 {
            return 0;
        }
        if unsafe { fputwc(wc, stream) } == WEOF {
            return -1;
        }
        i += 1;
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fwide.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fwide(stream: *mut FILE, mode: c_int) -> c_int {
    unsafe { (*stream).try_set_orientation(mode) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fwscanf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fwscanf(
    stream: *mut FILE,
    format: *const wchar_t,
    mut __valist: ...
) -> c_int {
    unsafe { vfwscanf(stream, format, __valist.as_va_list()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getwc.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getwc(stream: *mut FILE) -> wint_t {
    unsafe { fgetwc(stream) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getwchar.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getwchar() -> wint_t {
    unsafe { fgetwc(stdin) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mbsinit.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mbsinit(ps: *const mbstate_t) -> c_int {
    //Add a check for the state maybe
    if ps.is_null() { 1 } else { 0 }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mbrlen.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mbrlen(s: *const c_char, n: size_t, ps: *mut mbstate_t) -> size_t {
    static mut INTERNAL: mbstate_t = mbstate_t;
    unsafe { mbrtowc(ptr::null_mut(), s, n, &raw mut INTERNAL) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mbrtowc.html>.
///
/// Only works for UTF8 at the moment.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mbrtowc(
    pwc: *mut wchar_t,
    s: *const c_char,
    n: size_t,
    ps: *mut mbstate_t,
) -> size_t {
    static mut INTERNAL: mbstate_t = mbstate_t;

    if ps.is_null() {
        let ps = &raw mut INTERNAL;
    }
    if s.is_null() {
        let xs: [c_char; 1] = [0];
        unsafe { utf8::mbrtowc(pwc, ptr::from_ref::<c_char>(&xs[0]), 1, ps) }
    } else {
        unsafe { utf8::mbrtowc(pwc, s, n, ps) }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mbsnrtowcs.html>.
///
/// Convert a multibyte string to a wide string with a limited amount of bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mbsnrtowcs(
    dst_ptr: *mut wchar_t,
    src_ptr: *mut *const c_char,
    src_len: size_t,
    dst_len: size_t,
    ps: *mut mbstate_t,
) -> size_t {
    static mut INTERNAL: mbstate_t = mbstate_t;

    if ps.is_null() {
        let ps = &raw mut INTERNAL;
    }

    let mut src = unsafe { *src_ptr };

    let mut dst_offset: usize = 0;
    let mut src_offset: usize = 0;

    while (dst_ptr.is_null() || dst_offset < dst_len) && src_offset < src_len {
        let ps_copy = unsafe { *ps };
        let mut wc: wchar_t = 0;
        let amount = unsafe { mbrtowc(&mut wc, src.add(src_offset), src_len - src_offset, ps) };

        // Stop in the event a decoding error occured.
        if amount == -1isize as usize {
            unsafe { *src_ptr = src.add(src_offset) };
            return 1isize as usize;
        }

        // Stop decoding early in the event we encountered a partial character.
        if amount == -2isize as usize {
            unsafe { *ps = ps_copy };
            break;
        }

        // Store the decoded wide character in the destination buffer.
        if !dst_ptr.is_null() {
            unsafe { *dst_ptr.add(dst_offset) = wc };
        }

        // Stop decoding after decoding a null character and return a NULL
        // source pointer to the caller, not including the null character in the
        // number of characters stored in the destination buffer.
        if wc == 0 {
            src = ptr::null();
            src_offset = 0;
            break;
        }

        dst_offset += 1;
        src_offset += amount;
    }

    unsafe { *src_ptr = src.add(src_offset) };
    dst_offset
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mbsrtowcs.html>.
///
/// Convert a multibyte string to a wide string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mbsrtowcs(
    dst: *mut wchar_t,
    src: *mut *const c_char,
    len: size_t,
    ps: *mut mbstate_t,
) -> size_t {
    unsafe { mbsnrtowcs(dst, src, size_t::MAX, len, ps) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/putwc.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn putwc(wc: wchar_t, stream: *mut FILE) -> wint_t {
    unsafe { fputwc(wc, &mut *stream) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/putwchar.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn putwchar(wc: wchar_t) -> wint_t {
    unsafe { fputwc(wc, &mut *stdout) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/vswscanf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vswscanf(
    s: *const wchar_t,
    format: *const wchar_t,
    __valist: va_list,
) -> c_int {
    let reader = (s.cast::<wint_t>()).into();
    unsafe { wscanf::scanf(reader, format, __valist) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fwscanf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn swscanf(
    s: *const wchar_t,
    format: *const wchar_t,
    mut __valist: ...
) -> c_int {
    unsafe { vswscanf(s, format, __valist.as_va_list()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ungetwc.html>.
///
/// Push wide character `wc` back onto `stream` so it'll be read next
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ungetwc(wc: wint_t, stream: &mut FILE) -> wint_t {
    if wc == WEOF {
        return wc;
    }
    static mut INTERNAL: mbstate_t = mbstate_t;
    let mut bytes: [c_char; MB_CUR_MAX as usize] = [0; MB_CUR_MAX as usize];

    let amount = unsafe { wcrtomb(bytes.as_mut_ptr(), wc as wchar_t, &raw mut INTERNAL) };
    if amount == usize::MAX {
        return WEOF;
    }

    /*
    We might have unget multiple bytes for a single wchar, eg, `ç` is [195, 167].
    We need to unget them in reversed, so they are pused as [..., 167, 195, ...]
    When we do fgetwc, we pop from the Vec, getting the write order of bytes [195, 167].
    If we called ungetc in the non-reversed order, we would get [167, 195]
    */
    for i in 0..amount {
        unsafe { ungetc(c_int::from(bytes[amount - 1 - i]), &mut *stream) };
    }

    wc
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/vfwprintf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vfwprintf(
    stream: *mut FILE,
    format: *const wchar_t,
    arg: va_list,
) -> c_int {
    let mut stream = unsafe { (*stream).lock() };
    if (*stream).try_set_wide_orientation_unlocked().is_err() {
        return -1;
    }

    unsafe { wprintf::wprintf(&mut *stream, WStr::from_ptr(format), arg) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fwprintf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fwprintf(
    stream: *mut FILE,
    format: *const wchar_t,
    mut __valist: ...
) -> c_int {
    unsafe { vfwprintf(stream, format, __valist.as_va_list()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/vfwprintf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vwprintf(format: *const wchar_t, arg: va_list) -> c_int {
    unsafe { vfwprintf(&mut *stdout, format, arg) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fwprintf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wprintf(format: *const wchar_t, mut __valist: ...) -> c_int {
    unsafe { vfwprintf(&mut *stdout, format, __valist.as_va_list()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/vfwprintf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vswprintf(
    s: *mut wchar_t,
    n: size_t,
    format: *const wchar_t,
    arg: va_list,
) -> c_int {
    //TODO: implement vswprintf. This is not as simple as wprintf, since the output is not UTF-8
    // but instead is a wchar array.
    todo_skip!(0, "vswprintf not implemented");
    -1
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fwprintf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn swprintf(
    s: *mut wchar_t,
    n: size_t,
    format: *const wchar_t,
    mut __valist: ...
) -> c_int {
    unsafe { vswprintf(s, n, format, __valist.as_va_list()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcpcpy.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcpcpy(d: *mut wchar_t, s: *const wchar_t) -> *mut wchar_t {
    unsafe { (wcscpy(d, s)).add(wcslen(s)) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcpncpy.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcpncpy(d: *mut wchar_t, s: *const wchar_t, n: size_t) -> *mut wchar_t {
    unsafe { (wcsncpy(d, s, n)).add(wcsnlen(s, n)) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcrtomb.html>.
///
/// widechar to multibyte.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcrtomb(s: *mut c_char, wc: wchar_t, ps: *mut mbstate_t) -> size_t {
    let mut buffer: [c_char; MB_CUR_MAX as usize] = [0; MB_CUR_MAX as usize];
    let (s_cpy, wc_cpy) = if s.is_null() {
        (buffer.as_mut_ptr(), 0)
    } else {
        (s, wc)
    };

    unsafe { utf8::wcrtomb(s_cpy, wc_cpy, ps) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcsdup.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcsdup(s: *const wchar_t) -> *mut wchar_t {
    let l = unsafe { wcslen(s) };

    let d = unsafe { malloc((l + 1) * mem::size_of::<wchar_t>()) }.cast::<wchar_t>();

    if d.is_null() {
        ERRNO.set(ENOMEM);
        return ptr::null_mut();
    }

    unsafe { wmemcpy(d, s, l + 1) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcsrtombs.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcsrtombs(
    s: *mut c_char,
    ws: *mut *const wchar_t,
    n: size_t,
    mut st: *mut mbstate_t,
) -> size_t {
    let mut mbs = mbstate_t {};
    if st.is_null() {
        st = &mut mbs;
    }
    unsafe { wcsnrtombs(s, ws, size_t::MAX, n, st) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcscat.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcscat(ws1: *mut wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    unsafe { wcsncat(ws1, ws2, usize::MAX) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcschr.html>.
///
/// # Safety
/// The caller is required to ensure that `ws` is a valid pointer to a buffer
/// containing at least one nul value. The pointed-to buffer must not be
/// modified for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcschr(ws: *const wchar_t, wc: wchar_t) -> *mut wchar_t {
    // We iterate over non-mut references and thus need to coerce the
    // resulting reference via a *const pointer before we can get our *mut.
    // SAFETY: the caller is required to ensure that ws points to a valid
    // nul-terminated buffer.
    let ptr: *const wchar_t =
        match unsafe { NulTerminatedInclusive::new(ws) }.find(|&&wsc| wsc == wc) {
            Some(wsc_ref) => wsc_ref,
            None => ptr::null(),
        };
    ptr.cast_mut()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcscmp.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcscmp(ws1: *const wchar_t, ws2: *const wchar_t) -> c_int {
    unsafe { wcsncmp(ws1, ws2, usize::MAX) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcscoll.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcscoll(ws1: *const wchar_t, ws2: *const wchar_t) -> c_int {
    //TODO: locale comparison
    unsafe { wcscmp(ws1, ws2) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcscpy.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcscpy(ws1: *mut wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    let mut i = 0;
    loop {
        let wc = unsafe { *ws2.add(i) };
        unsafe { *ws1.add(i) = wc };
        i += 1;
        if wc == 0 {
            return ws1;
        }
    }
}

unsafe fn inner_wcsspn(mut wcs: *const wchar_t, set: *const wchar_t, reject: bool) -> size_t {
    let mut count = 0;
    while unsafe { *wcs } != 0 && unsafe { wcschr(set, *wcs).is_null() } == reject {
        wcs = unsafe { wcs.add(1) };
        count += 1;
    }
    count
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcscspn.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcscspn(wcs: *const wchar_t, set: *const wchar_t) -> size_t {
    unsafe { inner_wcsspn(wcs, set, true) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcsftime.html>.
#[unsafe(no_mangle)]
pub extern "C" fn wcsftime(
    wcs: *mut wchar_t,
    maxsize: size_t,
    format: *const wchar_t,
    timptr: *const tm,
) -> size_t {
    todo_skip!(0, "wcsftime is not implemented");
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcslen.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcslen(ws: *const wchar_t) -> size_t {
    unsafe { NulTerminated::new(ws).unwrap() }.count()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcsncat.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcsncat(
    ws1: *mut wchar_t,
    ws2: *const wchar_t,
    n: size_t,
) -> *mut wchar_t {
    let len = unsafe { wcslen(ws1) };
    let dest = unsafe { ws1.add(len) };
    let mut i = 0;
    while i < n {
        let wc = unsafe { *ws2.add(i) };
        if wc == 0 {
            break;
        }
        unsafe { *dest.add(i) = wc };
        i += 1;
    }
    unsafe { *dest.add(i) = 0 };
    ws1
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcsncmp.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcsncmp(ws1: *const wchar_t, ws2: *const wchar_t, n: size_t) -> c_int {
    for i in 0..n {
        let wc1 = unsafe { *ws1.add(i) };
        let wc2 = unsafe { *ws2.add(i) };
        if wc1 != wc2 {
            return wc1 - wc2;
        } else if wc1 == 0 {
            break;
        }
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcsncpy.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcsncpy(
    ws1: *mut wchar_t,
    ws2: *const wchar_t,
    n: size_t,
) -> *mut wchar_t {
    let mut i = 0;
    while i < n {
        let wc = unsafe { *ws2.add(i) };
        unsafe { *ws1.add(i) = wc };
        i += 1;
        if wc == 0 {
            break;
        }
    }
    while i < n {
        unsafe { *ws1.add(i) = 0 };
        i += 1;
    }
    ws1
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcsnlen.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcsnlen(mut s: *const wchar_t, maxlen: size_t) -> size_t {
    let mut len = 0;

    while len < maxlen {
        if unsafe { *s } == 0 {
            break;
        }

        len += 1;
        s = unsafe { s.offset(1) };
    }

    len
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcsnrtombs.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcsnrtombs(
    mut dest: *mut c_char,
    src: *mut *const wchar_t,
    nwc: size_t,
    len: size_t,
    mut ps: *mut mbstate_t,
) -> size_t {
    let mut written = 0;
    let mut read = 0;
    let mut buf: [c_char; MB_LEN_MAX as usize] = [0; MB_LEN_MAX as usize];
    let mut mbs = mbstate_t {};

    if ps.is_null() {
        ps = &mut mbs;
    }

    while read < nwc {
        buf.fill(0);

        let ret = unsafe { wcrtomb(buf.as_mut_ptr(), **src, ps) };

        if ret == size_t::MAX {
            ERRNO.set(EILSEQ);
            return size_t::MAX;
        }

        if !dest.is_null() && len < written + ret {
            return written;
        }

        if !dest.is_null() {
            unsafe { ptr::copy_nonoverlapping(buf.as_ptr(), dest, ret) };
            dest = unsafe { dest.add(ret) };
        }

        if unsafe { **src } == '\0' as wchar_t {
            unsafe { *src = ptr::null() };
            return written;
        }

        unsafe { *src = (*src).add(1) };
        read += 1;
        written += ret;
    }
    written
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcspbrk.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcspbrk(mut wcs: *const wchar_t, set: *const wchar_t) -> *mut wchar_t {
    wcs = unsafe { wcs.add(wcscspn(wcs, set)) };
    if unsafe { *wcs } == 0 {
        ptr::null_mut()
    } else {
        // Once again, C wants us to transmute a const pointer to a
        // mutable one...
        wcs.cast_mut()
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcsrchr.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcsrchr(ws1: *const wchar_t, wc: wchar_t) -> *mut wchar_t {
    let mut last_matching_wc = ptr::null::<wchar_t>();
    let mut i = 0;

    while unsafe { *ws1.add(i) } != 0 {
        if unsafe { *ws1.add(i) } == wc {
            last_matching_wc = unsafe { ws1.add(i) };
        }
        i += 1;
    }

    last_matching_wc.cast_mut()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcsspn.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcsspn(wcs: *const wchar_t, set: *const wchar_t) -> size_t {
    unsafe { inner_wcsspn(wcs, set, false) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcsstr.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcsstr(ws1: *const wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    // Get length of ws2, not including null terminator
    let ws2_len = unsafe { wcslen(ws2) };

    // The standard says that we must return ws1 if ws2 has length 0
    if ws2_len == 0 {
        ws1.cast_mut()
    } else {
        let ws1_len = unsafe { wcslen(ws1) };

        // Construct slices without null terminator
        let ws1_slice = unsafe { slice::from_raw_parts(ws1, ws1_len) };
        let ws2_slice = unsafe { slice::from_raw_parts(ws2, ws2_len) };

        /* Sliding ws2-sized window iterator on ws1. The iterator
         * returns None if ws2 is longer than ws1. */
        let mut ws1_windows = ws1_slice.windows(ws2_len);

        /* Find the first offset into ws1 where the window is equal to
         * the ws2 contents. Return null pointer if no match is found. */
        match ws1_windows.position(|ws1_window| ws1_window == ws2_slice) {
            Some(pos) => unsafe { ws1.add(pos).cast_mut() },
            None => ptr::null_mut(),
        }
    }
}

macro_rules! skipws {
    ($ptr:expr) => {
        while isspace(unsafe { *$ptr }) != 0 {
            $ptr = unsafe { $ptr.add(1) };
        }
    };
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcstod.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcstod(mut ptr: *const wchar_t, end: *mut *mut wchar_t) -> c_double {
    const RADIX: u32 = 10;

    skipws!(ptr);
    let negative = unsafe { *ptr } == '-' as wchar_t;
    if negative {
        ptr = unsafe { ptr.add(1) };
    }

    let mut result: c_double = 0.0;
    while let Some(digit) = char::from_u32(unsafe { *ptr } as _).and_then(|c| c.to_digit(RADIX)) {
        result *= 10.0;
        if negative {
            result -= c_double::from(digit);
        } else {
            result += c_double::from(digit);
        }
        ptr = unsafe { ptr.add(1) };
    }
    if unsafe { *ptr } == '.' as wchar_t {
        ptr = unsafe { ptr.add(1) };

        let mut scale = 1.0;
        while let Some(digit) = char::from_u32(unsafe { *ptr } as _).and_then(|c| c.to_digit(RADIX))
        {
            scale /= 10.0;
            if negative {
                result -= c_double::from(digit) * scale;
            } else {
                result += c_double::from(digit) * scale;
            }
            ptr = unsafe { ptr.add(1) };
        }
    }
    if !end.is_null() {
        unsafe { *end = ptr.cast_mut() };
    }
    result
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcstok.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcstok(
    mut wcs: *mut wchar_t,
    delim: *const wchar_t,
    state: *mut *mut wchar_t,
) -> *mut wchar_t {
    // Choose starting position
    if wcs.is_null() {
        if (unsafe { *state }).is_null() {
            // There was no next token
            return ptr::null_mut();
        }
        wcs = unsafe { *state };
    }

    // Advance past any delimiters
    wcs = unsafe { wcs.add(wcsspn(wcs, delim)) };

    // Check end
    if unsafe { *wcs } == 0 {
        unsafe { *state = ptr::null_mut() };
        return ptr::null_mut();
    }

    // Advance *to* any delimiters
    let end = unsafe { wcspbrk(wcs, delim) };
    if end.is_null() {
        unsafe { *state = ptr::null_mut() };
    } else {
        unsafe { *end = 0 };
        unsafe { *state = end.add(1) };
    }
    wcs
}

macro_rules! strtou_impl {
    ($type:ident, $ptr:expr, $base:expr) => {
        strtou_impl!($type, $ptr, $base, false)
    };
    ($type:ident, $ptr:expr, $base:expr, $negative:expr) => {{
        let mut base = $base;

        if (base == 16 || base == 0)
            && unsafe { *$ptr } == '0' as wchar_t
            && (unsafe { *$ptr.add(1) } == 'x' as wchar_t
                || unsafe { *$ptr.add(1) } == 'X' as wchar_t)
        {
            $ptr = unsafe { $ptr.add(2) };
            base = 16;
        }

        if base == 0 {
            base = if unsafe { *$ptr } == '0' as wchar_t {
                8
            } else {
                10
            };
        };

        let mut result: $type = 0;
        while let Some(digit) =
            char::from_u32(unsafe { *$ptr } as u32).and_then(|c| c.to_digit(base as u32))
        {
            let new = result.checked_mul(base as $type).and_then(|result| {
                if $negative {
                    result.checked_sub(digit as $type)
                } else {
                    result.checked_add(digit as $type)
                }
            });
            result = match new {
                Some(new) => new,
                None => {
                    platform::ERRNO.set(ERANGE);
                    return !0;
                }
            };

            $ptr = unsafe { $ptr.add(1) };
        }
        result
    }};
}
macro_rules! strto_impl {
    ($type:ident, $ptr:expr, $base:expr) => {{
        let negative = unsafe { *$ptr } == '-' as wchar_t;
        if negative {
            $ptr = unsafe { $ptr.add(1) };
        }
        strtou_impl!($type, $ptr, $base, negative)
    }};
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcstol.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcstol(
    mut ptr: *const wchar_t,
    end: *mut *mut wchar_t,
    base: c_int,
) -> c_long {
    skipws!(ptr);
    let result = strto_impl!(c_long, ptr, base);
    if !end.is_null() {
        unsafe { *end = ptr.cast_mut() };
    }
    result
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcstoll.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcstoll(
    mut ptr: *const wchar_t,
    end: *mut *mut wchar_t,
    base: c_int,
) -> c_longlong {
    skipws!(ptr);
    let result = strto_impl!(c_longlong, ptr, base);
    if !end.is_null() {
        unsafe { *end = ptr.cast_mut() };
    }
    result
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcstoimax.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcstoimax(
    mut ptr: *const wchar_t,
    end: *mut *mut wchar_t,
    base: c_int,
) -> intmax_t {
    skipws!(ptr);
    let result = strto_impl!(intmax_t, ptr, base);
    if !end.is_null() {
        unsafe { *end = ptr.cast_mut() };
    }
    result
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcstoul.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcstoul(
    mut ptr: *const wchar_t,
    end: *mut *mut wchar_t,
    base: c_int,
) -> c_ulong {
    skipws!(ptr);
    let result = strtou_impl!(c_ulong, ptr, base);
    if !end.is_null() {
        unsafe { *end = ptr.cast_mut() };
    }
    result
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcstoull.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcstoull(
    mut ptr: *const wchar_t,
    end: *mut *mut wchar_t,
    base: c_int,
) -> c_ulonglong {
    skipws!(ptr);
    let result = strtou_impl!(c_ulonglong, ptr, base);
    if !end.is_null() {
        unsafe { *end = ptr.cast_mut() };
    }
    result
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcstoimax.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcstoumax(
    mut ptr: *const wchar_t,
    end: *mut *mut wchar_t,
    base: c_int,
) -> uintmax_t {
    skipws!(ptr);
    let result = strtou_impl!(uintmax_t, ptr, base);
    if !end.is_null() {
        unsafe { *end = ptr.cast_mut() };
    }
    result
}

/// See <https://pubs.opengroup.org/onlinepubs/009604499/functions/wcswcs.html>.
///
/// Marked legacy in issue 6.
/// Encouraged to use `wcsstr` instead, which this implementation simply forwards to.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcswcs(ws1: *const wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    unsafe { wcsstr(ws1, ws2) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcswidth.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcswidth(pwcs: *const wchar_t, n: size_t) -> c_int {
    let mut total_width = 0;
    for i in 0..n {
        let wc_width = wcwidth(unsafe { *pwcs.add(i) });
        if wc_width < 0 {
            return -1;
        }
        total_width += wc_width;
    }
    total_width
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcsxfrm.html>.
#[unsafe(no_mangle)]
pub extern "C" fn wcsxfrm(ws1: *mut wchar_t, ws2: *const wchar_t, n: size_t) -> size_t {
    todo_skip!(0, "wcsxfrm is not implemented");
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wctob.html>.
#[unsafe(no_mangle)]
pub extern "C" fn wctob(c: wint_t) -> c_int {
    if c <= 0x7F { c as c_int } else { EOF }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcwidth.html>.
#[unsafe(no_mangle)]
pub extern "C" fn wcwidth(wc: wchar_t) -> c_int {
    match char::from_u32(wc as u32) {
        Some(c) => match unicode_width::UnicodeWidthChar::width(c) {
            Some(width) => width as c_int,
            None => -1,
        },
        None => -1,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wmemchr.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wmemchr(ws: *const wchar_t, wc: wchar_t, n: size_t) -> *mut wchar_t {
    for i in 0..n {
        if unsafe { *ws.add(i) } == wc {
            return unsafe { ws.add(i) }.cast_mut();
        }
    }
    ptr::null_mut()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wmemcmp.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wmemcmp(ws1: *const wchar_t, ws2: *const wchar_t, n: size_t) -> c_int {
    for i in 0..n {
        let wc1 = unsafe { *ws1.add(i) };
        let wc2 = unsafe { *ws2.add(i) };
        if wc1 != wc2 {
            return wc1 - wc2;
        }
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wmemcpy.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wmemcpy(
    ws1: *mut wchar_t,
    ws2: *const wchar_t,
    n: size_t,
) -> *mut wchar_t {
    (unsafe {
        string::memcpy(
            ws1.cast::<c_void>(),
            ws2.cast::<c_void>(),
            n * mem::size_of::<wchar_t>(),
        )
    })
    .cast::<wchar_t>()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wmemmove.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wmemmove(
    ws1: *mut wchar_t,
    ws2: *const wchar_t,
    n: size_t,
) -> *mut wchar_t {
    (unsafe {
        string::memmove(
            ws1.cast::<c_void>(),
            ws2.cast::<c_void>(),
            n * mem::size_of::<wchar_t>(),
        )
    })
    .cast::<wchar_t>()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wmemset.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wmemset(ws: *mut wchar_t, wc: wchar_t, n: size_t) -> *mut wchar_t {
    for i in 0..n {
        unsafe { *ws.add(i) = wc };
    }
    ws
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/vfwscanf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vfwscanf(
    stream: *mut FILE,
    format: *const wchar_t,
    __valist: va_list,
) -> c_int {
    let mut file = unsafe { (*stream).lock() };
    if file.try_set_byte_orientation_unlocked().is_err() {
        return -1;
    }

    let f: &mut FILE = &mut file;
    let reader: LookAheadReader = f.into();
    unsafe { wscanf::scanf(reader, format, __valist) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/vwscanf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vwscanf(format: *const wchar_t, __valist: va_list) -> c_int {
    unsafe { vfwscanf(stdin, format, __valist) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wscanf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wscanf(format: *const wchar_t, mut __valist: ...) -> c_int {
    unsafe { vfwscanf(stdin, format, __valist.as_va_list()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcscasecmp.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcscasecmp(mut s1: *const wchar_t, mut s2: *const wchar_t) -> c_int {
    unsafe {
        while *s1 != 0 && *s2 != 0 {
            if towlower(*s1 as wint_t) != towlower(*s2 as wint_t) {
                break;
            }
            s1 = s1.add(1);
            s2 = s2.add(1);
        }
        let result = towlower(*s1 as wint_t).wrapping_sub(towlower(*s2 as wint_t));
        result as c_int
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcsncasecmp.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcsncasecmp(
    mut s1: *const wchar_t,
    mut s2: *const wchar_t,
    n: size_t,
) -> c_int {
    if n == 0 {
        return 0;
    }
    unsafe {
        for _ in 0..n {
            if *s1 == 0 || *s2 == 0 || towlower(*s1 as wint_t) != towlower(*s2 as wint_t) {
                return towlower(*s1 as wint_t).wrapping_sub(towlower(*s2 as wint_t)) as c_int;
            }
            s1 = s1.add(1);
            s2 = s2.add(1);
        }
        0
    }
}
