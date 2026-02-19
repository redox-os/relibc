//! `inttypes.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/inttypes.h.html>.

use crate::{
    header::{ctype, errno::*, stdlib::*},
    platform::{
        self,
        types::{c_char, c_int, c_long, intmax_t, uintmax_t},
    },
};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/imaxabs.html>.
#[unsafe(no_mangle)]
pub extern "C" fn imaxabs(i: intmax_t) -> intmax_t {
    i.abs()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/inttypes.h.html>.
#[repr(C)]
pub struct imaxdiv_t {
    quot: intmax_t,
    rem: intmax_t,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/imaxdiv.html>.
#[unsafe(no_mangle)]
pub extern "C" fn imaxdiv(i: intmax_t, j: intmax_t) -> imaxdiv_t {
    imaxdiv_t {
        quot: i / j,
        rem: i % j,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strtoimax.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strtoimax(
    s: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
) -> intmax_t {
    strto_impl!(
        intmax_t,
        false,
        intmax_t::MAX,
        intmax_t::MIN,
        s,
        endptr,
        base
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strtoimax.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strtoumax(
    s: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
) -> uintmax_t {
    strto_impl!(
        uintmax_t,
        false,
        uintmax_t::MAX,
        uintmax_t::MIN,
        s,
        endptr,
        base
    )
}

// wcstoimax(), wcstoumax() currently defined in header::wchar?
