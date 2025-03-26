/// monetary.h implementation for Redox, following the POSIX standard.
/// Following https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/monetary.h.html
///
/// We should provide a strfmon() implementation that formats a monetary value,
/// according to the current locale (TODO).
use alloc::string::{String, ToString};
use core::{ffi::CStr, ptr, slice, str};

use rust_libm::{fabs, floor, pow, round, trunc};

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

/// Formats a monetary value according to the current locale.
fn apply_grouping(int_str: &str, monetary: &LocaleMonetaryInfo) -> String {
    let mut grouped = String::with_capacity(int_str.len() * 2);
    let mut count = 0;
    let mut group_idx = 0;
    let current_grouping = &monetary.mon_grouping;
    let separator = monetary.mon_thousands_sep;

    for c in int_str.chars() {
        if count > 0 {
            let current_group = current_grouping[group_idx.min(current_grouping.len() - 1)];
            if current_group > 0 && count % current_group == 0 {
                grouped.push_str(separator);
                if group_idx + 1 < current_grouping.len() {
                    group_idx += 1;
                }
            }
        }
        grouped.push(c);
        count += 1;
    }

    grouped
}

/// Safe handling of large monetary values. Returns None if the value is too large to format
fn format_value_parts(value: f64, frac_digits: usize) -> Option<(String, i64)> {
    let abs_value = fabs(value);
    if abs_value > (i64::MAX as f64) {
        // Check if the value is too large to format
        return None;
    }

    let int_part = trunc(abs_value) as i64;
    let scale = pow(10.0, frac_digits as f64);
    let frac_part = round((abs_value - int_part as f64) * scale) as i64;

    Some((int_part.to_string(), frac_part))
}
