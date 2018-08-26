// #[no_mangle]
pub extern "C" fn iswalnum(wc: wint_t, locale: locale_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswalpha(wc: wint_t, locale: locale_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswcntrl(wc: wint_t, locale: locale_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswdigit(wc: wint_t, locale: locale_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswgraph(wc: wint_t, locale: locale_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswlower(wc: wint_t, locale: locale_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswprint(wc: wint_t, locale: locale_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswpunct(wc: wint_t, locale: locale_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswspace(wc: wint_t, locale: locale_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswupper(wc: wint_t, locale: locale_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswxdigit(wc: wint_t, locale: locale_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswblank(wc: wint_t, locale: locale_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wctype(property: *const libc::c_char, locale: locale_t) -> wctype_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn iswctype(wc: wint_t, desc: wctype_t, locale: locale_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn towlower(wc: wint_t, locale: locale_t) -> wint_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn towupper(wc: wint_t, locale: locale_t) -> wint_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn wctrans(property: *const libc::c_char, locale: locale_t) -> wctrans_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn towctrans(wc: wint_t, desc: wctrans_t, locale: locale_t) -> wint_t {
    unimplemented!();
}
