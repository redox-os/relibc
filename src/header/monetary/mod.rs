
/// monetary.h implementation for Redox, following the POSIX standard.
/// Following https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/monetary.h.html
///
/// We should provide a strfmon() implementation that formats a monetary value.
/// according to the current locale (TODO).
use alloc::string::{String, ToString};
use core::{ffi::CStr, ptr, slice, str};

extern crate alloc;

mod strfmon;
#[deny(unsafe_op_in_unsafe_fn)]
#[repr(C)]
struct LocaleMonetaryInfo {
    int_curr_symbol: &'static str,
    currency_symbol: &'static str,
    mon_decimal_point: &'static str,
    mon_thousands_sep: &'static str,
    mon_grouping: &'static [u8],
    positive_sign: &'static str,
    negative_sign: &'static str,
    int_frac_digits: u8,
    frac_digits: u8,
    p_cs_precedes: bool,
    p_sep_by_space: bool,
    p_sign_posn: u8,
    n_cs_precedes: bool,
    n_sep_by_space: bool,
    n_sign_posn: u8,
}

const DEFAULT_MONETARY: LocaleMonetaryInfo = LocaleMonetaryInfo {
    int_curr_symbol: "USD ",
    currency_symbol: "$",
    mon_decimal_point: ".",
    mon_thousands_sep: ",",
    mon_grouping: &[3, 3],
    positive_sign: "",
    negative_sign: "-",
    int_frac_digits: 2,
    frac_digits: 2,
    p_cs_precedes: true,
    p_sep_by_space: false,
    p_sign_posn: 1,
    n_cs_precedes: true,
    n_sep_by_space: false,
    n_sign_posn: 1,
};

#[derive(Default)]
struct FormatFlags {
    left_justify: bool,
    force_sign: bool,
    space_sign: bool,
    use_parens: bool,
    suppress_symbol: bool,
    field_width: Option<usize>,
    left_precision: Option<usize>,
    right_precision: Option<usize>,
    international: bool,
}

/// Use our own floating point implementation
/// TODO : use num-traits crate
#[inline]
fn my_fabs(x: f64) -> f64 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

#[inline]
fn my_floor(x: f64) -> f64 {
    let i = x as i64;
    if x < 0.0 && x != i as f64 {
        (i - 1) as f64
    } else {
        i as f64
    }
}

#[inline]
fn my_trunc(x: f64) -> f64 {
    if x < 0.0 {
        -my_floor(-x)
    } else {
        my_floor(x)
    }
}

#[inline]
fn my_round(x: f64) -> f64 {
    my_floor(x + 0.5)
}

#[inline]
fn my_pow10(n: usize) -> f64 {
    let mut result = 1.0;
    for _ in 0..n {
        result *= 10.0;
    }
    result
}

/// TODO : improve grouping implementation
fn apply_grouping(int_str: &str, monetary: &LocaleMonetaryInfo) -> String {
    let mut grouped = String::with_capacity(int_str.len() * 2);
    let mut count = 0;
    let mut group_idx = 0;

    // The grouping array can have up to 4 elements, but the last element is always 0
    for c in int_str.chars().rev() {
        if count > 0 && count % monetary.mon_grouping[group_idx] == 0 {
            grouped.push_str(monetary.mon_thousands_sep);
            // Move to next grouping size if available
            if group_idx + 1 < monetary.mon_grouping.len() {
                group_idx += 1;
            }
        }
        grouped.push(c);
        count += 1;
    }
    // Reverse the string to get the correct order
    grouped.chars().rev().collect()
}

/// Safe handling of large monetary values. Returns None if the value is too large to format
fn format_value_parts(value: f64, frac_digits: usize) -> Option<(String, i64)> {
    let abs_value = my_fabs(value);
    if abs_value > (i64::MAX as f64) {
        return None;
    }

    let int_part = my_trunc(abs_value) as i64;
    let scale = my_pow10(frac_digits);
    let frac_part = my_round((abs_value - int_part as f64) * scale) as i64;

    Some((int_part.to_string(), frac_part))
}
