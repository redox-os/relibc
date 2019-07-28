use crate::{
    header::{ctype, errno::*, stdlib::*},
    platform::{self, types::*},
};

#[no_mangle]
pub extern "C" fn imaxabs(i: intmax_t) -> intmax_t {
    i.abs()
}

#[no_mangle]
#[repr(C)]
pub struct imaxdiv_t {
    quot: intmax_t,
    rem: intmax_t,
}

#[no_mangle]
pub extern "C" fn imaxdiv(i: intmax_t, j: intmax_t) -> imaxdiv_t {
    imaxdiv_t {
        quot: i / j,
        rem: i % j,
    }
}

#[no_mangle]
pub unsafe extern "C" fn strtoimax(
    s: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
) -> intmax_t {
    strto_impl!(
        intmax_t,
        false,
        intmax_t::max_value(),
        intmax_t::min_value(),
        s,
        endptr,
        base
    )
}

#[no_mangle]
pub unsafe extern "C" fn strtoumax(
    s: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
) -> uintmax_t {
    strto_impl!(
        uintmax_t,
        false,
        uintmax_t::max_value(),
        uintmax_t::min_value(),
        s,
        endptr,
        base
    )
}

#[allow(unused)]
// #[no_mangle]
pub extern "C" fn wcstoimax(
    nptr: *const wchar_t,
    endptr: *mut *mut wchar_t,
    base: c_int,
) -> intmax_t {
    unimplemented!();
}

#[allow(unused)]
// #[no_mangle]
pub extern "C" fn wcstoumax(
    nptr: *const wchar_t,
    endptr: *mut *mut wchar_t,
    base: c_int,
) -> uintmax_t {
    unimplemented!();
}
