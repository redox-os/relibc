use super::{apply_grouping, FormatFlags, LocaleMonetaryInfo, DEFAULT_MONETARY};
use alloc::string::{String, ToString};
use core::{ffi::CStr, ptr, result, slice, str};
use rust_libm::{fabs, floor, pow, round, trunc};

/// The `strfmon()` function formats a monetary value according to the format string `format`
/// and writes the result to the character array `s` of size `maxsize`.
/// The format string can contain plain characters and format specifiers.
///
/// Returns:
/// - The number of characters written (excluding the null terminator), or -1 if
/// an error occurs (e.g., invalid input, buffer overflow)
pub unsafe extern "C" fn strfmon(
    s: *mut i8,        // Output buffer
    maxsize: usize,    // Maximum size of the buffer
    format: *const i8, // Format string
    mut args: ...      // Variadic arguments for monetary values
) -> isize {
    // Validate input pointers and buffer size
    if s.is_null() || format.is_null() || maxsize == 0 {
        return -1; // Invalid input
    }

    // Convert the format string from C to string
    let format_str = match unsafe { CStr::from_ptr(format) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1, // Invalid format string
    };

    // Create a mutable slice for the output buffer
    let buffer = unsafe { slice::from_raw_parts_mut(s as *mut u8, maxsize) };
    let mut pos = 0;
    let mut format_chars = format_str.chars().peekable();

    // Parse the format string
    while let Some(c) = format_chars.next() {
        // Handle plain characters (non-`%`)
        if c != '%' {
            if pos >= buffer.len() {
                return -1; // Buffer overflow
            }
            buffer[pos] = c as u8; // Write the character to the buffer
            pos += 1;
            continue;
        }

        // Handle `%%` escape sequence
        if format_chars.peek() == Some(&'%') {
            if pos >= buffer.len() {
                return -1; // Buffer overflow
            }
            buffer[pos] = b'%'; // Write a literal `%`
            pos += 1;
            format_chars.next(); // Skip the second `%`
            continue;
        }

        // Parse format specifiers
        let mut flags = FormatFlags::default();

        // Parse flags (`+`, `(`, ...)
        while let Some(&next_char) = format_chars.peek() {
            match next_char {
                '=' => flags.left_justify = true,
                '+' => flags.force_sign = true,
                ' ' => flags.space_sign = true,
                '(' => flags.use_parens = true,
                '!' => flags.suppress_symbol = true,
                _ => break, // Stop when no more flags are found
            }
            format_chars.next();
        }

        // Parse field width
        let mut num_str = String::new();
        while let Some(&c) = format_chars.peek() {
            if !c.is_ascii_digit() {
                break;
            }
            num_str.push(c);
            format_chars.next();
        }
        if !num_str.is_empty() {
            flags.field_width = num_str.parse().ok(); // Parse as integer
        }

        // Parse left precision (`#`)
        if format_chars.peek() == Some(&'#') {
            format_chars.next(); // Skip `#`
            num_str.clear(); // Clear the string for precision
            while let Some(&c) = format_chars.peek() {
                if !c.is_ascii_digit() {
                    break;
                }
                num_str.push(c);
                format_chars.next();
            }
            flags.left_precision = num_str.parse().ok(); // Parse as integer
        }

        // Parse right precision indicated by `.`
        if format_chars.peek() == Some(&'.') {
            format_chars.next(); // Skip `.`
            num_str.clear(); // Clear the string for precision
            while let Some(&c) = format_chars.peek() {
                if !c.is_ascii_digit() {
                    break;
                }
                num_str.push(c);
                format_chars.next();
            }
            flags.right_precision = num_str.parse().ok(); // Parse as integer
        }

        // Handle conversion specifiers (`i` or `n`)
        match format_chars.next() {
            Some('i') => {
                // International formatting
                flags.international = true;
                let value = unsafe { args.arg::<f64>() }; // Get the argument as f64
                if let Some(written) =
                    format_monetary(&mut buffer[pos..], value, &DEFAULT_MONETARY, &flags)
                {
                    pos += written; // Update the position
                } else {
                    return -1; // Formatting failed
                }
            }
            Some('n') => {
                // Locale-specific formatting
                let value = unsafe { args.arg::<f64>() }; // Get the argument as f64
                if let Some(written) =
                    format_monetary(&mut buffer[pos..], value, &DEFAULT_MONETARY, &flags)
                {
                    pos += written; // Update the position
                } else {
                    return -1; // Failed to format the value
                }
            }
            _ => return -1,
        }
    }

    // Ensure there is space for the null terminator
    if pos >= buffer.len() {
        return -1; // Buffer overflow
    }
    buffer[pos] = 0; // Null-terminate the buffer
    pos as isize // Return the number of characters written
}

/// Formats a monetary value into the given `buffer` using locale-specific rules
/// from `monetary` and formatting options from `flags`.
/// Returns `Some(len)` if successful, where `len` is the number of bytes written,
/// or `None` on error.
///
/// # Parameters
/// - `buffer`: The output slice in which to write the formatted string
/// - `value`: The numeric value to format as monetary
/// - `monetary`: Locale-specific formatting rules, including symbols, separators,
///   and grouping sizes
/// - `flags`: Additional formatting options
///
fn format_monetary(
    buffer: &mut [u8],
    value: f64,
    monetary: &LocaleMonetaryInfo,
    flags: &FormatFlags,
) -> Option<usize> {
    // 1) determine sign and absolute value
    let is_negative = value < 0.0;
    let abs_value = fabs(value);

    // 2) figure out how many fractionals digits to use
    let frac_digits = if flags.international {
        flags
            .right_precision
            .unwrap_or(monetary.int_frac_digits as usize)
    } else {
        flags
            .right_precision
            .unwrap_or(monetary.frac_digits as usize)
    };

    // 3) split the value into integer and fractional parts
    let scale = pow(10.0, frac_digits as f64);

    // Check for overflow
    let max_safe_int = (i64::MAX as f64) / scale;
    if abs_value >= max_safe_int {
        return None;
    }

    let mut int_part = trunc(abs_value) as i64;
    let mut frac_part = round((abs_value - int_part as f64) * scale) as i64;

    // 4) handle carry-over if frac_part equals or exceeds scale after rounding
    if frac_part >= scale as i64 {
        int_part += 1;
        frac_part = 0;
    }

    // 5) convert the integer part to string
    let mut int_str = int_part.to_string();

    // 6) apply left precision if specified (padding with '0')
    //    So if left_precision is 5 and int_str is "42", it becomes "00042".
    if let Some(left_prec) = flags.left_precision {
        if int_str.len() > left_prec {
            // The integer part is too large to fit the precision
            return None;
        }
        // Right-align the number in a field of `left_prec` width, padded with '0'
        int_str = format!("{:0>width$}", int_str, width = left_prec);
    }

    // 7) build the final formatted output in a temporary String
    let mut result = String::with_capacity(int_str.len() * 2 + 20);

    // 7a) determine currency symbol placement and sign rules
    let (cs_precedes, sep_by_space, sign_posn) = if is_negative {
        (
            monetary.n_cs_precedes,
            monetary.n_sep_by_space,
            monetary.n_sign_posn,
        )
    } else {
        (
            monetary.p_cs_precedes,
            monetary.p_sep_by_space,
            monetary.p_sign_posn,
        )
    };

    // 7b) determine which sign to display
    //     - negative sign if value is negative
    //     - positive_sign if user forced sign
    //     - space if space_sign is set and value is positive
    //     - empty otherwise
    let sign = match (is_negative, flags.force_sign, flags.space_sign) {
        (true, _, _) => monetary.negative_sign,
        (false, true, _) => monetary.positive_sign,
        (false, false, true) => " ",
        _ => "",
    };

    // 7c) choose which currency symbol to display
    //     - maybe empty if suppressed
    //     - int_curr_symbol for international format
    //     - currency_symbol for local format
    let symbol = if flags.suppress_symbol {
        ""
    } else if flags.international {
        monetary.int_curr_symbol
    } else {
        monetary.currency_symbol
    };

    // 8) add opening parenthesis if sign position is 0
    if sign_posn == 0 {
        result.push('(');
    } else if sign_posn == 1 {
        result.push_str(sign);
    }

    // 9) add currency symbol if it precedes the amount
    if cs_precedes {
        result.push_str(symbol);
        if sep_by_space {
            result.push(' ');
        }
    }

    // 10) group the integer string and append it
    let grouped = apply_grouping(&int_str, monetary);
    result.push_str(&grouped);

    // 11) append the fractional part, if any
    if frac_digits > 0 {
        result.push_str(monetary.mon_decimal_point);
        // Zero-pad fractional part to the specified width
        result.push_str(&format!("{:0>width$}", frac_part, width = frac_digits));
    }

    // 12) if the currency symbol follows the amount, add it now
    if !cs_precedes {
        if sep_by_space {
            result.push(' ');
        }
        result.push_str(symbol);
    }

    // 13) if sign_posn == 0, close the parenthesis
    if sign_posn == 0 {
        result.push(')');
    }

    // 14) checks if the user specified a total field width
    //     - if the final result is shorter, we padd it
    //     - if `left_justify` is true, padd on the right otherwise padd on the left
    if let Some(width) = flags.field_width {
        if result.len() < width {
            let padding = " ".repeat(width - result.len());
            result = if flags.left_justify {
                result + &padding
            } else {
                padding + &result
            };
        }
    }

    // 15) write the final string to the buffer
    if result.len() > buffer.len() {
        // Not enough space in the output buffer
        return None;
    }
    buffer[..result.len()].copy_from_slice(result.as_bytes());

    // 16) return how many bytes we wrote
    Some(result.len())
}
