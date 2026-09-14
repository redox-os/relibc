//! `inttypes.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/inttypes.h.html>.

use crate::{
    header::{
        errno::{EINVAL, ERANGE},
        stdlib::{convert_hex, convert_integer, convert_octal, detect_base, parse_sign},
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
        unsafe { CStr::from_ptr(nptr) },
        unsafe { endptr.as_mut() },
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
        unsafe { CStr::from_ptr(nptr) },
        unsafe { endptr.as_mut() },
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcstoimax(
    nptr: *const wchar_t,
    endptr: *mut *mut wchar_t,
    base: c_int,
) -> intmax_t {
    wcsto_impl!(
        intmax_t,
        unsafe { WStr::from_ptr(nptr) },
        unsafe { endptr.as_mut() },
        base
    )
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
    nptr: *const wchar_t,
    endptr: *mut *mut wchar_t,
    base: c_int,
) -> uintmax_t {
    wcsto_impl!(
        uintmax_t,
        unsafe { WStr::from_ptr(nptr) },
        unsafe { endptr.as_mut() },
        base
    )
}
