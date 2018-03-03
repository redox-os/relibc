#[no_mangle]
pub extern "C" fn iswalnum(__wc: wint_t, __locale: locale_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswalpha(__wc: wint_t, __locale: locale_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswcntrl(__wc: wint_t, __locale: locale_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswdigit(__wc: wint_t, __locale: locale_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswgraph(__wc: wint_t, __locale: locale_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswlower(__wc: wint_t, __locale: locale_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswprint(__wc: wint_t, __locale: locale_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswpunct(__wc: wint_t, __locale: locale_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswspace(__wc: wint_t, __locale: locale_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswupper(__wc: wint_t, __locale: locale_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswxdigit(__wc: wint_t, __locale: locale_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswblank(__wc: wint_t, __locale: locale_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wctype(__property: *const libc::c_char, __locale: locale_t) -> wctype_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn iswctype(__wc: wint_t, __desc: wctype_t, _locale: locale_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn towlower(__wc: wint_t, __locale: locale_t) -> wint_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn towupper(__wc: wint_t, __locale: locale_t) -> wint_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wctrans(__property: *const libc::c_char, _locale: locale_t) -> wctrans_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn towctrans(__wc: wint_t, __desc: wctrans_t, _locale: locale_t) -> wint_t {
    unimplemented!();
}
