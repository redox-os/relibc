//! wchar implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/wchar.h.html

use core::ptr;
use va_list::VaList as va_list;

use header::stdio::*;
use header::stdlib::MB_CUR_MAX;
use header::time::*;
use platform;
use platform::types::*;

mod utf8;

const WEOF: wint_t = 0xFFFFFFFFu32;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct mbstate_t;

#[no_mangle]
pub unsafe extern "C" fn btowc(c: c_int) -> wint_t {
    //Check for EOF
    if c as wint_t == WEOF {
        return WEOF;
    }

    let uc = c as u8;
    let c = uc as c_char;
    let mut ps: mbstate_t = mbstate_t;
    let mut wc: wchar_t = 0;
    let saved_errno = platform::errno;
    let status = mbrtowc(&mut wc, &c as (*const c_char), 1, &mut ps);
    if status == usize::max_value() || status == usize::max_value() - 1 {
        platform::errno = saved_errno;
        return WEOF;
    }
    return wc as wint_t;
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

// #[no_mangle]
pub extern "C" fn fputws(ws: *const wchar_t, stream: *mut FILE) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn fwide(stream: *mut FILE, mode: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn getwc(stream: *mut FILE) -> wint_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn getwchar() -> wint_t {
    unimplemented!();
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
        return utf8::mbrtowc(pwc, &xs[0] as *const c_char, 1, ps);
    } else {
        return utf8::mbrtowc(pwc, s, n, ps);
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
        let amount = mbrtowc(
            &mut wc,
            src.offset(src_offset as isize),
            src_len - src_offset,
            ps,
        );

        // Stop in the event a decoding error occured.
        if amount == -1isize as usize {
            *src_ptr = src.offset(src_offset as isize);
            return 1isize as usize;
        }

        // Stop decoding early in the event we encountered a partial character.
        if amount == -2isize as usize {
            *ps = ps_copy;
            break;
        }

        // Store the decoded wide character in the destination buffer.
        if !dst_ptr.is_null() {
            *dst_ptr.offset(dst_offset as isize) = wc;
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

    *src_ptr = src.offset(src_offset as isize);
    return dst_offset;
}

//Convert a multibyte string to a wide string
#[no_mangle]
pub extern "C" fn mbsrtowcs(
    dst: *mut wchar_t,
    src: *mut *const c_char,
    len: size_t,
    ps: *mut mbstate_t,
) -> size_t {
    unsafe { mbsnrtowcs(dst, src, size_t::max_value(), len, ps) }
}

#[no_mangle]
pub unsafe extern "C" fn putwc(wc: wchar_t, stream: *mut FILE) -> wint_t {
    fputwc(wc, &mut *stream)
}

#[no_mangle]
pub unsafe extern "C" fn putwchar(wc: wchar_t) -> wint_t {
    fputwc(wc, &mut *stdout)
}

// #[no_mangle]
pub extern "C" fn swprintf(
    s: *mut wchar_t,
    n: size_t,
    format: *const wchar_t,
    ap: va_list,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn swscanf(s: *const wchar_t, format: *const wchar_t, ap: va_list) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn towlower(wc: wint_t) -> wint_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn towupper(wc: wint_t) -> wint_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn ungetwc(wc: wint_t, stream: *mut FILE) -> wint_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn vfwprintf(stream: *mut FILE, format: *const wchar_t, arg: va_list) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn vwprintf(format: *const wchar_t, arg: va_list) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn vswprintf(
    s: *mut wchar_t,
    n: size_t,
    format: *const wchar_t,
    arg: va_list,
) -> c_int {
    unimplemented!();
}

//widechar to multibyte
#[no_mangle]
pub extern "C" fn wcrtomb(s: *mut c_char, wc: wchar_t, ps: *mut mbstate_t) -> size_t {
    let mut buffer: [c_char; MB_CUR_MAX as usize] = [0; MB_CUR_MAX as usize];
    let mut wc_cpy = wc;
    let mut s_cpy = s;

    if s.is_null() {
        wc_cpy = 0;
        s_cpy = buffer.as_mut_ptr();
    }

    unsafe { utf8::wcrtomb(s_cpy, wc_cpy, ps) }
}

// #[no_mangle]
pub extern "C" fn wcscat(ws1: *mut wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcschr(ws1: *const wchar_t, ws2: wchar_t) -> *mut c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcscmp(ws1: *const wchar_t, ws2: *const wchar_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcscoll(ws1: *const wchar_t, ws2: *const wchar_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcscpy(ws1: *mut wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcscspn(ws1: *const wchar_t, ws2: *const wchar_t) -> size_t {
    unimplemented!();
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

// #[no_mangle]
pub extern "C" fn wcslen(ws: *const wchar_t) -> c_ulong {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcsncat(ws1: *mut wchar_t, ws2: *const wchar_t, n: size_t) -> *mut wchar_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcsncmp(ws1: *const wchar_t, ws2: *const wchar_t, n: size_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcsncpy(ws1: *mut wchar_t, ws2: *const wchar_t, n: size_t) -> *mut wchar_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcspbrk(ws1: *const wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcsrchr(ws1: *const wchar_t, ws2: wchar_t) -> *mut wchar_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcsrtombs(
    dst: *mut c_char,
    src: *mut *const wchar_t,
    len: size_t,
    ps: *mut mbstate_t,
) -> size_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcsspn(ws1: *const wchar_t, ws2: *const wchar_t) -> size_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcsstr(ws1: *const wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcstod(nptr: *const wchar_t, endptr: *mut *mut wchar_t) -> f64 {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcstok(
    ws1: *mut wchar_t,
    ws2: *const wchar_t,
    ptr: *mut *mut wchar_t,
) -> *mut wchar_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcstol(nptr: *const wchar_t, endptr: *mut *mut wchar_t, base: c_int) -> c_long {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcstoul(nptr: *const wchar_t, endptr: *mut *mut wchar_t, base: c_int) -> c_ulong {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcswcs(ws1: *const wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcswidth(pwcs: *const wchar_t, n: size_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcsxfrm(ws1: *mut wchar_t, ws2: *const wchar_t, n: size_t) -> size_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wctob(c: wint_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wcwidth(wc: wchar_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wmemchr(ws: *const wchar_t, wc: wchar_t, n: size_t) -> *mut c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wmemcmp(ws1: *const wchar_t, ws2: *const wchar_t, n: size_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wmemcpy(ws1: *mut wchar_t, ws2: *const wchar_t, n: size_t) -> *mut wchar_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wmemmove(ws1: *mut wchar_t, ws2: *const wchar_t, n: size_t) -> *mut wchar_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wmemset(ws1: *mut wchar_t, ws2: wchar_t, n: size_t) -> *mut wchar_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wprintf(format: *const wchar_t, ap: va_list) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wscanf(format: *const wchar_t, ap: va_list) -> c_int {
    unimplemented!();
}
