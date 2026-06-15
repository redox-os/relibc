//! `inttypes.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/inttypes.h.html>.

use crate::{
    header::{
        ctype::{self, isspace},
        errno::{EINVAL, ERANGE},
        stdlib::{convert_hex, convert_integer, convert_octal, detect_base, is_positive},
    },
    platform::{
        self,
        types::{c_char, c_int, c_long, intmax_t, uintmax_t, wchar_t},
    },
};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/imaxabs.html>.
///
/// Computes the absolute value of the integer `j`.
#[unsafe(no_mangle)]
pub extern "C" fn imaxabs(j: intmax_t) -> intmax_t {
    j.abs()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/inttypes.h.html>.
///
/// Structure type that is the type of the value returned by the `imaxdiv()`
/// function.
#[repr(C)]
pub struct imaxdiv_t {
    /// The quotient.
    quot: intmax_t,
    /// The remainder.
    rem: intmax_t,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/imaxdiv.html>.
///
/// Computes `numer` / `denom` and `numer` % `denom` in a single operation.
///
/// Returns the struct `imaxdiv_t`, comprising both the quotient and remainder.
#[unsafe(no_mangle)]
pub extern "C" fn imaxdiv(numer: intmax_t, denom: intmax_t) -> imaxdiv_t {
    imaxdiv_t {
        quot: numer / denom,
        rem: numer % denom,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strtoimax.html>.
///
/// Equivalent to `strtol()` and `strtoll()`, except that the initial portion
/// of the string shall be converted to `intmax_t`.
///
/// Upon success, returns the converted value. If no conversion could be
/// performed or the value of `base` is not supported, returns `0`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strtoimax(
    nptr: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
) -> intmax_t {
    strto_impl!(
        intmax_t,
        false,
        intmax_t::MAX,
        intmax_t::MIN,
        nptr,
        endptr,
        base
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strtoimax.html>.
///
/// Equivalent to `strtoul()` and `strtoull()`, except that the initial portion
/// of the string shall be converted to `uintmax_t`.
///
/// Upon success, returns the converted value. If no conversion could be
/// performed or the value of `base` is not supported, returns `0`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strtoumax(
    nptr: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
) -> uintmax_t {
    strto_impl!(
        uintmax_t,
        false,
        uintmax_t::MAX,
        uintmax_t::MIN,
        nptr,
        endptr,
        base
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcstoimax.html>.
///
/// Equivalent to `wctol()` and `wctoll()`, except that the initial portion
/// of the wide string shall be converted to `intmax_t`.
///
/// Upon success, returns the converted value. If no conversion could be
/// performed, returns `0`.
#[expect(clippy::cast_lossless)] // not all users of `wcsto_impl!` are lossless
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcstoimax(
    mut nptr: *const wchar_t,
    endptr: *mut *mut wchar_t,
    base: c_int,
) -> intmax_t {
    skipws!(nptr);
    let result = wcsto_impl!(intmax_t, nptr, base);
    if !endptr.is_null() {
        unsafe { *endptr = nptr.cast_mut() };
    }
    result
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcstoimax.html>.
///
/// Equivalent to `wctoul()` and `wctoull()`, except that the initial portion
/// of the wide string shall be converted to `uintmax_t`.
///
/// Upon success, returns the converted value. If no conversion could be
/// performed, returns `0`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcstoumax(
    mut nptr: *const wchar_t,
    endptr: *mut *mut wchar_t,
    base: c_int,
) -> uintmax_t {
    skipws!(nptr);
    let result = wcsto_impl!(uintmax_t, nptr, base);
    if !endptr.is_null() {
        unsafe { *endptr = nptr.cast_mut() };
    }
    result
}
