//! wchar implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/wchar.h.html

use core::{char, ffi::VaList as va_list, mem, ptr, slice, usize};

use crate::{
    header::{
        ctype::isspace,
        errno::{EILSEQ, ENOMEM, ERANGE},
        stdio::*,
        stdlib::{malloc, MB_CUR_MAX, MB_LEN_MAX},
        string,
        time::*,
        wctype::*,
    },
    iter::{NulTerminated, NulTerminatedInclusive},
    platform::{self, types::*, ERRNO},
};

mod lookaheadreader;
mod utf8;
mod wprintf;
mod wscanf;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct mbstate_t;

#[no_mangle]
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
    let status = mbrtowc(&mut wc, &c as *const c_char, 1, &mut ps);
    if status == usize::max_value() || status == usize::max_value() - 1 {
        platform::ERRNO.set(saved_errno);
        return WEOF;
    }
    wc as wint_t
}

#[no_mangle]
pub unsafe extern "C" fn fgetwc(stream: *mut FILE) -> wint_t {
    // TODO: Process locale
    let mut buf: [c_uchar; MB_CUR_MAX as usize] = [0; MB_CUR_MAX as usize];
    let mut encoded_length = 0;
    let mut bytes_read = 0;
    let mut wc: wchar_t = 0;

    loop {
        let nread = fread(
            buf[bytes_read..bytes_read + 1].as_mut_ptr() as *mut c_void,
            1,
            1,
            stream,
        );

        if nread != 1 {
            ERRNO.set(EILSEQ);
            return WEOF;
        }

        bytes_read += 1;

        if bytes_read == 1 {
            encoded_length = if buf[0] >> 7 == 0 {
                1
            } else if buf[0] >> 5 == 6 {
                2
            } else if buf[0] >> 4 == 0xe {
                3
            } else if buf[0] >> 3 == 0x1e {
                4
            } else {
                ERRNO.set(EILSEQ);
                return WEOF;
            };
        }

        if bytes_read >= encoded_length {
            break;
        }
    }

    mbrtowc(
        &mut wc,
        buf.as_ptr() as *const c_char,
        encoded_length,
        ptr::null_mut(),
    );

    wc as wint_t
}

#[no_mangle]
pub unsafe extern "C" fn fgetws(ws: *mut wchar_t, n: c_int, stream: *mut FILE) -> *mut wchar_t {
    //TODO: lock
    let mut i = 0;
    while ((i + 1) as c_int) < n {
        let wc = fgetwc(stream);
        if wc == WEOF {
            return ptr::null_mut();
        }
        *ws.add(i) = wc as wchar_t;
        i += 1;
    }
    while (i as c_int) < n {
        *ws.add(i) = 0;
        i += 1;
    }
    ws
}

#[no_mangle]
pub unsafe extern "C" fn fputwc(wc: wchar_t, stream: *mut FILE) -> wint_t {
    //Convert wchar_t to multibytes first
    static mut INTERNAL: mbstate_t = mbstate_t;
    let mut bytes: [c_char; MB_CUR_MAX as usize] = [0; MB_CUR_MAX as usize];

    let amount = wcrtomb(bytes.as_mut_ptr(), wc, &mut INTERNAL);

    for i in 0..amount {
        fputc(bytes[i] as c_int, &mut *stream);
    }

    wc as wint_t
}

#[no_mangle]
pub unsafe extern "C" fn fputws(ws: *const wchar_t, stream: *mut FILE) -> c_int {
    let mut i = 0;
    loop {
        let wc = *ws.add(i);
        if wc == 0 {
            return 0;
        }
        if fputwc(wc, stream) == WEOF {
            return -1;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn fwide(stream: *mut FILE, mode: c_int) -> c_int {
    (*stream).try_set_orientation(mode)
}

#[no_mangle]
pub unsafe extern "C" fn getwc(stream: *mut FILE) -> wint_t {
    fgetwc(stream)
}

#[no_mangle]
pub unsafe extern "C" fn getwchar() -> wint_t {
    fgetwc(stdin)
}

#[no_mangle]
pub unsafe extern "C" fn mbsinit(ps: *const mbstate_t) -> c_int {
    //Add a check for the state maybe
    if ps.is_null() {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn mbrlen(s: *const c_char, n: size_t, ps: *mut mbstate_t) -> size_t {
    static mut INTERNAL: mbstate_t = mbstate_t;
    mbrtowc(ptr::null_mut(), s, n, &mut INTERNAL)
}

//Only works for UTF8 at the moment
#[no_mangle]
pub unsafe extern "C" fn mbrtowc(
    pwc: *mut wchar_t,
    s: *const c_char,
    n: size_t,
    ps: *mut mbstate_t,
) -> size_t {
    static mut INTERNAL: mbstate_t = mbstate_t;

    if ps.is_null() {
        let ps = &mut INTERNAL;
    }
    if s.is_null() {
        let xs: [c_char; 1] = [0];
        utf8::mbrtowc(pwc, &xs[0] as *const c_char, 1, ps)
    } else {
        utf8::mbrtowc(pwc, s, n, ps)
    }
}

//Convert a multibyte string to a wide string with a limited amount of bytes
//Required for in POSIX.1-2008
#[no_mangle]
pub unsafe extern "C" fn mbsnrtowcs(
    dst_ptr: *mut wchar_t,
    src_ptr: *mut *const c_char,
    src_len: size_t,
    dst_len: size_t,
    ps: *mut mbstate_t,
) -> size_t {
    static mut INTERNAL: mbstate_t = mbstate_t;

    if ps.is_null() {
        let ps = &mut INTERNAL;
    }

    let mut src = *src_ptr;

    let mut dst_offset: usize = 0;
    let mut src_offset: usize = 0;

    while (dst_ptr.is_null() || dst_offset < dst_len) && src_offset < src_len {
        let ps_copy = *ps;
        let mut wc: wchar_t = 0;
        let amount = mbrtowc(&mut wc, src.add(src_offset), src_len - src_offset, ps);

        // Stop in the event a decoding error occured.
        if amount == -1isize as usize {
            *src_ptr = src.add(src_offset);
            return 1isize as usize;
        }

        // Stop decoding early in the event we encountered a partial character.
        if amount == -2isize as usize {
            *ps = ps_copy;
            break;
        }

        // Store the decoded wide character in the destination buffer.
        if !dst_ptr.is_null() {
            *dst_ptr.add(dst_offset) = wc;
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

    *src_ptr = src.add(src_offset);
    dst_offset
}

//Convert a multibyte string to a wide string
#[no_mangle]
pub unsafe extern "C" fn mbsrtowcs(
    dst: *mut wchar_t,
    src: *mut *const c_char,
    len: size_t,
    ps: *mut mbstate_t,
) -> size_t {
    mbsnrtowcs(dst, src, size_t::max_value(), len, ps)
}

#[no_mangle]
pub unsafe extern "C" fn putwc(wc: wchar_t, stream: *mut FILE) -> wint_t {
    fputwc(wc, &mut *stream)
}

#[no_mangle]
pub unsafe extern "C" fn putwchar(wc: wchar_t) -> wint_t {
    fputwc(wc, &mut *stdout)
}

#[no_mangle]
pub unsafe extern "C" fn vswscanf(
    s: *const wchar_t,
    format: *const wchar_t,
    __valist: va_list,
) -> c_int {
    let reader = (s as *const wint_t).into();
    wscanf::scanf(reader, format, __valist)
}

#[no_mangle]
pub unsafe extern "C" fn swscanf(
    s: *const wchar_t,
    format: *const wchar_t,
    mut __valist: ...
) -> c_int {
    vswscanf(s, format, __valist.as_va_list())
}

/// Push wide character `wc` back onto `stream` so it'll be read next
#[no_mangle]
pub unsafe extern "C" fn ungetwc(wc: wint_t, stream: &mut FILE) -> wint_t {
    if wc == WEOF {
        return wc;
    }
    static mut INTERNAL: mbstate_t = mbstate_t;
    let mut bytes: [c_char; MB_CUR_MAX as usize] = [0; MB_CUR_MAX as usize];

    let amount = wcrtomb(bytes.as_mut_ptr(), wc as wchar_t, &mut INTERNAL);
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
        ungetc(bytes[amount - 1 - i] as c_int, &mut *stream);
    }

    wc
}

#[no_mangle]
pub unsafe extern "C" fn vfwprintf(
    stream: *mut FILE,
    format: *const wchar_t,
    arg: va_list,
) -> c_int {
    let mut stream = (*stream).lock();
    if let Err(_) = (*stream).try_set_wide_orientation_unlocked() {
        return -1;
    }

    wprintf::wprintf(&mut *stream, format, arg)
}
#[no_mangle]
pub unsafe extern "C" fn fwprintf(
    stream: *mut FILE,
    format: *const wchar_t,
    mut __valist: ...
) -> c_int {
    vfwprintf(stream, format, __valist.as_va_list())
}

#[no_mangle]
pub unsafe extern "C" fn vwprintf(format: *const wchar_t, arg: va_list) -> c_int {
    vfwprintf(&mut *stdout, format, arg)
}
#[no_mangle]
pub unsafe extern "C" fn wprintf(format: *const wchar_t, mut __valist: ...) -> c_int {
    vfwprintf(&mut *stdout, format, __valist.as_va_list())
}

#[no_mangle]
pub unsafe extern "C" fn vswprintf(
    s: *mut wchar_t,
    n: size_t,
    format: *const wchar_t,
    arg: va_list,
) -> c_int {
    //TODO: implement vswprintf. This is not as simple as wprintf, since the output is not UTF-8
    // but instead is a wchar array.
    eprintln!("vswprintf not implemented");
    -1
}
#[no_mangle]
pub unsafe extern "C" fn swprintf(
    s: *mut wchar_t,
    n: size_t,
    format: *const wchar_t,
    mut __valist: ...
) -> c_int {
    vswprintf(s, n, format, __valist.as_va_list())
}

#[no_mangle]
pub unsafe extern "C" fn wcpcpy(d: *mut wchar_t, s: *const wchar_t) -> *mut wchar_t {
    return (wcscpy(d, s)).offset(wcslen(s) as isize);
}

#[no_mangle]
pub unsafe extern "C" fn wcpncpy(d: *mut wchar_t, s: *const wchar_t, n: size_t) -> *mut wchar_t {
    return (wcsncpy(d, s, n)).offset(wcsnlen(s, n) as isize);
}

//widechar to multibyte
#[no_mangle]
pub unsafe extern "C" fn wcrtomb(s: *mut c_char, wc: wchar_t, ps: *mut mbstate_t) -> size_t {
    let mut buffer: [c_char; MB_CUR_MAX as usize] = [0; MB_CUR_MAX as usize];
    let (s_cpy, wc_cpy) = if s.is_null() {
        (buffer.as_mut_ptr(), 0)
    } else {
        (s, wc)
    };

    utf8::wcrtomb(s_cpy, wc_cpy, ps)
}

#[no_mangle]
pub unsafe extern "C" fn wcsdup(s: *const wchar_t) -> *mut wchar_t {
    let l = wcslen(s);

    let d = malloc((l + 1) * mem::size_of::<wchar_t>()) as *mut wchar_t;

    if d.is_null() {
        ERRNO.set(ENOMEM);
        return ptr::null_mut();
    }

    wmemcpy(d, s, l + 1)
}

#[no_mangle]
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
    wcsnrtombs(s, ws, size_t::MAX, n, st)
}

#[no_mangle]
pub unsafe extern "C" fn wcscat(ws1: *mut wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    wcsncat(ws1, ws2, usize::MAX)
}

/// See <https://pubs.opengroup.org/onlinepubs/7908799/xsh/wcschr.html>.
///
/// # Safety
/// The caller is required to ensure that `ws` is a valid pointer to a buffer
/// containing at least one nul value. The pointed-to buffer must not be
/// modified for the duration of the call.
#[no_mangle]
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

#[no_mangle]
pub unsafe extern "C" fn wcscmp(ws1: *const wchar_t, ws2: *const wchar_t) -> c_int {
    wcsncmp(ws1, ws2, usize::MAX)
}

#[no_mangle]
pub unsafe extern "C" fn wcscoll(ws1: *const wchar_t, ws2: *const wchar_t) -> c_int {
    //TODO: locale comparison
    wcscmp(ws1, ws2)
}

#[no_mangle]
pub unsafe extern "C" fn wcscpy(ws1: *mut wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    let mut i = 0;
    loop {
        let wc = *ws2.add(i);
        *ws1.add(i) = wc;
        i += 1;
        if wc == 0 {
            return ws1;
        }
    }
}

unsafe fn inner_wcsspn(mut wcs: *const wchar_t, set: *const wchar_t, reject: bool) -> size_t {
    let mut count = 0;
    while (*wcs) != 0 && wcschr(set, *wcs).is_null() == reject {
        wcs = wcs.add(1);
        count += 1;
    }
    count
}

#[no_mangle]
pub unsafe extern "C" fn wcscspn(wcs: *const wchar_t, set: *const wchar_t) -> size_t {
    inner_wcsspn(wcs, set, true)
}

// #[no_mangle]
pub extern "C" fn wcsftime(
    wcs: *mut wchar_t,
    maxsize: size_t,
    format: *const wchar_t,
    timptr: *mut tm,
) -> size_t {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn wcslen(ws: *const wchar_t) -> size_t {
    unsafe { NulTerminated::new(ws) }.count()
}

#[no_mangle]
pub unsafe extern "C" fn wcsncat(
    ws1: *mut wchar_t,
    ws2: *const wchar_t,
    n: size_t,
) -> *mut wchar_t {
    let len = wcslen(ws1);
    let dest = ws1.add(len);
    let mut i = 0;
    while i < n {
        let wc = *ws2.add(i);
        if wc == 0 {
            break;
        }
        *dest.add(i) = wc;
        i += 1;
    }
    *dest.add(i) = 0;
    ws1
}

#[no_mangle]
pub unsafe extern "C" fn wcsncmp(ws1: *const wchar_t, ws2: *const wchar_t, n: size_t) -> c_int {
    for i in 0..n {
        let wc1 = *ws1.add(i);
        let wc2 = *ws2.add(i);
        if wc1 != wc2 {
            return wc1 - wc2;
        } else if wc1 == 0 {
            break;
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn wcsncpy(
    ws1: *mut wchar_t,
    ws2: *const wchar_t,
    n: size_t,
) -> *mut wchar_t {
    let mut i = 0;
    while i < n {
        let wc = *ws2.add(i);
        *ws1.add(i) = wc;
        i += 1;
        if wc == 0 {
            break;
        }
    }
    while i < n {
        *ws1.add(i) = 0;
        i += 1;
    }
    ws1
}

#[no_mangle]
pub unsafe extern "C" fn wcsnlen(mut s: *const wchar_t, maxlen: size_t) -> size_t {
    let mut len = 0;

    while len < maxlen {
        if *s == 0 {
            break;
        }

        len = len + 1;
        s = s.offset(1);
    }

    return len;
}

#[no_mangle]
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

        let ret = wcrtomb(buf.as_mut_ptr(), **src, ps);

        if ret == size_t::MAX {
            ERRNO.set(EILSEQ);
            return size_t::MAX;
        }

        if !dest.is_null() && len < written + ret {
            return written;
        }

        if !dest.is_null() {
            ptr::copy_nonoverlapping(buf.as_ptr(), dest, ret);
            dest = dest.add(ret);
        }

        if **src == '\0' as wchar_t {
            *src = ptr::null();
            return written;
        }

        *src = (*src).add(1);
        read += 1;
        written += ret;
    }
    written
}

#[no_mangle]
pub unsafe extern "C" fn wcspbrk(mut wcs: *const wchar_t, set: *const wchar_t) -> *mut wchar_t {
    wcs = wcs.add(wcscspn(wcs, set));
    if *wcs == 0 {
        ptr::null_mut()
    } else {
        // Once again, C wants us to transmute a const pointer to a
        // mutable one...
        wcs as *mut _
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcsrchr(ws1: *const wchar_t, wc: wchar_t) -> *mut wchar_t {
    let mut last_matching_wc = 0 as *const wchar_t;
    let mut i = 0;

    while *ws1.add(i) != 0 {
        if *ws1.add(i) == wc {
            last_matching_wc = ws1.add(i);
        }
        i += 1;
    }

    last_matching_wc as *mut wchar_t
}

#[no_mangle]
pub unsafe extern "C" fn wcsspn(wcs: *const wchar_t, set: *const wchar_t) -> size_t {
    inner_wcsspn(wcs, set, false)
}

#[no_mangle]
pub unsafe extern "C" fn wcsstr(ws1: *const wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    // Get length of ws2, not including null terminator
    let ws2_len = wcslen(ws2);

    // The standard says that we must return ws1 if ws2 has length 0
    if ws2_len == 0 {
        ws1 as *mut wchar_t
    } else {
        let ws1_len = wcslen(ws1);

        // Construct slices without null terminator
        let ws1_slice = slice::from_raw_parts(ws1, ws1_len);
        let ws2_slice = slice::from_raw_parts(ws2, ws2_len);

        /* Sliding ws2-sized window iterator on ws1. The iterator
         * returns None if ws2 is longer than ws1. */
        let mut ws1_windows = ws1_slice.windows(ws2_len);

        /* Find the first offset into ws1 where the window is equal to
         * the ws2 contents. Return null pointer if no match is found. */
        match ws1_windows.position(|ws1_window| ws1_window == ws2_slice) {
            Some(pos) => ws1.add(pos) as *mut wchar_t,
            None => ptr::null_mut(),
        }
    }
}

macro_rules! skipws {
    ($ptr:expr) => {
        while isspace(*$ptr) != 0 {
            $ptr = $ptr.add(1);
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn wcstod(mut ptr: *const wchar_t, end: *mut *mut wchar_t) -> c_double {
    const RADIX: u32 = 10;

    skipws!(ptr);
    let negative = *ptr == '-' as wchar_t;
    if negative {
        ptr = ptr.add(1);
    }

    let mut result: c_double = 0.0;
    while let Some(digit) = char::from_u32(*ptr as _).and_then(|c| c.to_digit(RADIX)) {
        result *= 10.0;
        if negative {
            result -= digit as c_double;
        } else {
            result += digit as c_double;
        }
        ptr = ptr.add(1);
    }
    if *ptr == '.' as wchar_t {
        ptr = ptr.add(1);

        let mut scale = 1.0;
        while let Some(digit) = char::from_u32(*ptr as _).and_then(|c| c.to_digit(RADIX)) {
            scale /= 10.0;
            if negative {
                result -= digit as c_double * scale;
            } else {
                result += digit as c_double * scale;
            }
            ptr = ptr.add(1);
        }
    }
    if !end.is_null() {
        *end = ptr as *mut _;
    }
    result
}

#[no_mangle]
pub unsafe extern "C" fn wcstok(
    mut wcs: *mut wchar_t,
    delim: *const wchar_t,
    state: *mut *mut wchar_t,
) -> *mut wchar_t {
    // Choose starting position
    if wcs.is_null() {
        if (*state).is_null() {
            // There was no next token
            return ptr::null_mut();
        }
        wcs = *state;
    }

    // Advance past any delimiters
    wcs = wcs.add(wcsspn(wcs, delim));

    // Check end
    if *wcs == 0 {
        *state = ptr::null_mut();
        return ptr::null_mut();
    }

    // Advance *to* any delimiters
    let end = wcspbrk(wcs, delim);
    if end.is_null() {
        *state = ptr::null_mut();
    } else {
        *end = 0;
        *state = end.add(1);
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
            && *$ptr == '0' as wchar_t
            && (*$ptr.add(1) == 'x' as wchar_t || *$ptr.add(1) == 'X' as wchar_t)
        {
            $ptr = $ptr.add(2);
            base = 16;
        }

        if base == 0 {
            base = if *$ptr == '0' as wchar_t { 8 } else { 10 };
        };

        let mut result: $type = 0;
        while let Some(digit) = char::from_u32(*$ptr as u32).and_then(|c| c.to_digit(base as u32)) {
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

            $ptr = $ptr.add(1);
        }
        result
    }};
}
macro_rules! strto_impl {
    ($type:ident, $ptr:expr, $base:expr) => {{
        let negative = *$ptr == '-' as wchar_t;
        if negative {
            $ptr = $ptr.add(1);
        }
        strtou_impl!($type, $ptr, $base, negative)
    }};
}

#[no_mangle]
pub unsafe extern "C" fn wcstol(
    mut ptr: *const wchar_t,
    end: *mut *mut wchar_t,
    base: c_int,
) -> c_long {
    skipws!(ptr);
    let result = strto_impl!(c_long, ptr, base);
    if !end.is_null() {
        *end = ptr as *mut _;
    }
    result
}

#[no_mangle]
pub unsafe extern "C" fn wcstoll(
    mut ptr: *const wchar_t,
    end: *mut *mut wchar_t,
    base: c_int,
) -> c_longlong {
    skipws!(ptr);
    let result = strto_impl!(c_longlong, ptr, base);
    if !end.is_null() {
        *end = ptr as *mut _;
    }
    result
}

#[no_mangle]
pub unsafe extern "C" fn wcstoimax(
    mut ptr: *const wchar_t,
    end: *mut *mut wchar_t,
    base: c_int,
) -> intmax_t {
    skipws!(ptr);
    let result = strto_impl!(intmax_t, ptr, base);
    if !end.is_null() {
        *end = ptr as *mut _;
    }
    result
}

#[no_mangle]
pub unsafe extern "C" fn wcstoul(
    mut ptr: *const wchar_t,
    end: *mut *mut wchar_t,
    base: c_int,
) -> c_ulong {
    skipws!(ptr);
    let result = strtou_impl!(c_ulong, ptr, base);
    if !end.is_null() {
        *end = ptr as *mut _;
    }
    result
}

#[no_mangle]
pub unsafe extern "C" fn wcstoull(
    mut ptr: *const wchar_t,
    end: *mut *mut wchar_t,
    base: c_int,
) -> c_ulonglong {
    skipws!(ptr);
    let result = strtou_impl!(c_ulonglong, ptr, base);
    if !end.is_null() {
        *end = ptr as *mut _;
    }
    result
}

#[no_mangle]
pub unsafe extern "C" fn wcstoumax(
    mut ptr: *const wchar_t,
    end: *mut *mut wchar_t,
    base: c_int,
) -> uintmax_t {
    skipws!(ptr);
    let result = strtou_impl!(uintmax_t, ptr, base);
    if !end.is_null() {
        *end = ptr as *mut _;
    }
    result
}

#[no_mangle]
pub unsafe extern "C" fn wcswcs(ws1: *const wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    wcsstr(ws1, ws2)
}

#[no_mangle]
pub unsafe extern "C" fn wcswidth(pwcs: *const wchar_t, n: size_t) -> c_int {
    let mut total_width = 0;
    for i in 0..n {
        let wc_width = wcwidth(*pwcs.add(i));
        if wc_width < 0 {
            return -1;
        }
        total_width += wc_width;
    }
    total_width
}

// #[no_mangle]
pub extern "C" fn wcsxfrm(ws1: *mut wchar_t, ws2: *const wchar_t, n: size_t) -> size_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wctob(c: wint_t) -> c_int {
    if c <= 0x7F {
        c as c_int
    } else {
        EOF
    }
}

#[no_mangle]
pub extern "C" fn wcwidth(wc: wchar_t) -> c_int {
    match char::from_u32(wc as u32) {
        Some(c) => match unicode_width::UnicodeWidthChar::width(c) {
            Some(width) => width as c_int,
            None => -1,
        },
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn wmemchr(ws: *const wchar_t, wc: wchar_t, n: size_t) -> *mut wchar_t {
    for i in 0..n {
        if *ws.add(i) == wc {
            return ws.add(i) as *mut wchar_t;
        }
    }
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn wmemcmp(ws1: *const wchar_t, ws2: *const wchar_t, n: size_t) -> c_int {
    for i in 0..n {
        let wc1 = *ws1.add(i);
        let wc2 = *ws2.add(i);
        if wc1 != wc2 {
            return wc1 - wc2;
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn wmemcpy(
    ws1: *mut wchar_t,
    ws2: *const wchar_t,
    n: size_t,
) -> *mut wchar_t {
    string::memcpy(
        ws1 as *mut c_void,
        ws2 as *const c_void,
        n * mem::size_of::<wchar_t>(),
    ) as *mut wchar_t
}

#[no_mangle]
pub unsafe extern "C" fn wmemmove(
    ws1: *mut wchar_t,
    ws2: *const wchar_t,
    n: size_t,
) -> *mut wchar_t {
    string::memmove(
        ws1 as *mut c_void,
        ws2 as *const c_void,
        n * mem::size_of::<wchar_t>(),
    ) as *mut wchar_t
}

#[no_mangle]
pub unsafe extern "C" fn wmemset(ws: *mut wchar_t, wc: wchar_t, n: size_t) -> *mut wchar_t {
    for i in 0..n {
        *ws.add(i) = wc;
    }
    ws
}

// #[no_mangle]
pub extern "C" fn wscanf(format: *const wchar_t, ap: va_list) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcscasecmp(mut s1: *const wchar_t, mut s2: *const wchar_t) -> c_int {
    unsafe {
        while *s1 != 0 && *s2 != 0 {
            if towlower(*s1 as wint_t) != towlower(*s2 as wint_t) {
                break;
            }
            s1 = s1.add(1);
            s2 = s2.add(1);
        }
        let result = towlower(*s1 as wint_t).wrapping_sub(towlower(*s2 as wint_t));
        return result as c_int;
    }
}

#[no_mangle]
pub extern "C" fn wcsncasecmp(mut s1: *const wchar_t, mut s2: *const wchar_t, n: size_t) -> c_int {
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
        return 0;
    }
}
