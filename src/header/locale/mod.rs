//! locale implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/locale.h.html

use core::ptr;

use crate::platform::types::*;

const EMPTY_PTR: *const c_char = "\0" as *const _ as *const c_char;
// Can't use &str because of the mutability
static mut C_LOCALE: [c_char; 2] = [b'C' as c_char, 0];

#[repr(C)]
#[no_mangle]
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

static CURRENT_LOCALE: lconv = lconv {
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

#[no_mangle]
pub extern "C" fn localeconv() -> *const lconv {
    &CURRENT_LOCALE as *const _
}

#[no_mangle]
pub unsafe extern "C" fn setlocale(_option: c_int, val: *const c_char) -> *mut c_char {
    if val.is_null() {
        return C_LOCALE.as_mut_ptr() as *mut c_char;
    }
    // TODO actually implement
    ptr::null_mut()
}
