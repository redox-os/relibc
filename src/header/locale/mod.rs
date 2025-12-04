//! `locale.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/locale.h.html>.

use core::ptr;

use crate::platform::types::*;

const EMPTY_PTR: *const c_char = "\0" as *const _ as *const c_char;
// Can't use &str because of the mutability
static mut C_LOCALE: [c_char; 2] = [b'C' as c_char, 0];

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/locale.h.html>.
#[repr(C)]
pub struct lconv {
    currency_symbol: *const c_char,
    decimal_point: *const c_char,
    frac_digits: c_char,
    grouping: *const c_char,
    int_curr_symbol: *const c_char,
    int_frac_digits: c_char,
    mon_decimal_point: *const c_char,
    mon_grouping: *const c_char,
    mon_thousands_sep: *const c_char,
    negative_sign: *const c_char,
    n_cs_precedes: c_char,
    n_sep_by_space: c_char,
    n_sign_posn: c_char,
    positive_sign: *const c_char,
    p_cs_precedes: c_char,
    p_sep_by_space: c_char,
    p_sign_posn: c_char,
    thousands_sep: *const c_char,
}
unsafe impl Sync for lconv {}

// Mutable because POSIX demands a mutable pointer, even though it warns
// against mutating it
static mut CURRENT_LOCALE: lconv = lconv {
    currency_symbol: EMPTY_PTR,
    decimal_point: ".\0" as *const _ as *const c_char,
    frac_digits: c_char::max_value(),
    grouping: EMPTY_PTR,
    int_curr_symbol: EMPTY_PTR,
    int_frac_digits: c_char::max_value(),
    mon_decimal_point: EMPTY_PTR,
    mon_grouping: EMPTY_PTR,
    mon_thousands_sep: EMPTY_PTR,
    negative_sign: EMPTY_PTR,
    n_cs_precedes: c_char::max_value(),
    n_sep_by_space: c_char::max_value(),
    n_sign_posn: c_char::max_value(),
    positive_sign: EMPTY_PTR,
    p_cs_precedes: c_char::max_value(),
    p_sep_by_space: c_char::max_value(),
    p_sign_posn: c_char::max_value(),
    thousands_sep: EMPTY_PTR,
};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/localeconv.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn localeconv() -> *mut lconv {
    &raw mut CURRENT_LOCALE as *mut _
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setlocale.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setlocale(_option: c_int, _val: *const c_char) -> *mut c_char {
    // TODO actually implement
    &raw mut C_LOCALE as *mut c_char
}
