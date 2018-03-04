pub type wchar_t = libc::c_int;
pub type wint_t = libc::c_uint;

#[no_mangle]
pub extern "C" fn btowc(c: libc::c_int) -> wint_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fwprintf(stream: *mut FILE, format: *const wchar_t, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fwscanf(stream: *mut FILE, format: *const wchar_t, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswalnum(wc: wint_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswalpha(wc: wint_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswcntrl(wc: wint_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswdigit(wc: wint_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswgraph(wc: wint_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswlower(wc: wint_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswprint(wc: wint_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswpunct(wc: wint_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswspace(wc: wint_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswupper(wc: wint_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswxdigit(wc: wint_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fgetwc(stream: *mut FILE) -> wint_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fgetws(ws: *mut wchar_t, n: libc::c_int,
                  stream: *mut FILE) -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fputwc(wc: wchar_t, stream: *mut FILE) -> wint_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fputws(ws: *const wchar_t, stream: *mut FILE)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fwide(stream: *mut FILE, mode: libc::c_int)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getwc(stream: *mut FILE) -> wint_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getwchar() -> wint_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mbsinit(ps: *const mbstate_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mbrlen(s: *const libc::c_char, n: usize,
                  ps: *mut mbstate_t) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mbrtowc(pwc: *mut wchar_t, s: *const libc::c_char,
                   n: usize, ps: *mut mbstate_t) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mbsrtowcs(dst: *mut wchar_t,
                     src: *mut *const libc::c_char, len: usize,
                     ps: *mut mbstate_t) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn putwc(wc: wchar_t, stream: *mut FILE) -> wint_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn putwchar(wc: wchar_t) -> wint_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn swprintf(s: *mut wchar_t, n: usize,
                    format: *const wchar_t, ...) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn swscanf(s: *const wchar_t, format: *const wchar_t, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn towlower(wc: wint_t) -> wint_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn towupper(wc: wint_t) -> wint_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ungetwc(wc: wint_t, stream: *mut FILE) -> wint_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn vfwprintf(stream: *mut FILE, format: *const wchar_t,
                   arg: va_list) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn vwprintf(format: *const wchar_t, arg: va_list)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn vswprintf(s: *mut wchar_t, n: usize, format: *const wchar_t,
                     arg: va_list) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcrtomb(s: *mut libc::c_char, wc: wchar_t,
                   ps: *mut mbstate_t) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcscat(ws1: *mut wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcschr(ws1: *const wchar_t, ws2: wchar_t)
     -> *mut libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcscmp(ws1: *const wchar_t, ws2: *const wchar_t)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcscoll(ws1: *const wchar_t, ws2: *const wchar_t)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcscpy(ws1: *mut wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcscspn(ws1: *const wchar_t, ws2: *const wchar_t) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcsftime(wcs: *mut wchar_t, maxsize: usize, format: *const wchar_t,
                    timptr: *mut tm) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcslen(ws: *const wchar_t) -> libc::c_ulong {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcsncat(ws1: *mut wchar_t, ws2: *const wchar_t, n: usize)
     -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcsncmp(ws1: *const wchar_t, ws2: *const wchar_t, n: usize)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcsncpy(ws1: *mut wchar_t, ws2: *const wchar_t, n: usize)
     -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcspbrk(ws1: *const wchar_t, ws2: *const wchar_t)
     -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcsrchr(ws1: *const wchar_t, ws2: wchar_t) -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcsrtombs(dst: *mut libc::c_char,
                     src: *mut *const wchar_t, len: usize,
                     ps: *mut mbstate_t) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcsspn(ws1: *const wchar_t, ws2: *const wchar_t) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcsstr(ws1: *const wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcstod(nptr: *const wchar_t, endptr: *mut *mut wchar_t) -> f64 {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcstok(ws1: *mut wchar_t, ws2: *const wchar_t,
                  ptr: *mut *mut wchar_t) -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcstol(nptr: *const wchar_t, endptr: *mut *mut wchar_t,
                  base: libc::c_int) -> libc::c_long {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcstoul(nptr: *const wchar_t, endptr: *mut *mut wchar_t,
                   base: libc::c_int) -> libc::c_ulong {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcswcs(ws1: *const wchar_t, ws2: *const wchar_t) -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcswidth(pwcs: *const wchar_t, n: usize)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcsxfrm(ws1: *mut wchar_t, ws2: *const wchar_t, n: usize)
     -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wctob(c: wint_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wcwidth(wc: wchar_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wmemchr(ws: *const wchar_t, wc: wchar_t, n: usize)
     -> *mut libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wmemcmp(ws1: *const wchar_t, ws2: *const wchar_t, n: usize)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wmemcpy(ws1: *mut wchar_t, ws2: *const wchar_t, n: usize)
     -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wmemmove(ws1: *mut wchar_t, ws2: *const wchar_t, n: usize)
     -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wmemset(ws1: *mut wchar_t, ws2: wchar_t, n: usize)
     -> *mut wchar_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wprintf(format: *const wchar_t, ...) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wscanf(format: *const wchar_t, ...) -> libc::c_int {
    unimplemented!();
}
