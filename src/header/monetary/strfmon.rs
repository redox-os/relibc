use super::{FormatFlags, LocaleMonetaryInfo, DEFAULT_MONETARY};
use alloc::string::{String, ToString};
use core::{ffi::CStr, ptr, result, slice, str};
use libm::{fabs, floor, pow, round, trunc};

/// The `strfmon()` function formats a monetary value according to the format string `format`
/// and writes the result to the character array `s` of size `maxsize`.
/// The format string can contain plain characters and format specifiers.
///
/// Returns:
/// - The number of characters written (excluding the null terminator), or
/// - -1 if an error occurs (e.g., invalid input, buffer overflow).
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

fn format_monetary(
    buffer: &mut [u8],
    value: f64,
    monetary: &LocaleMonetaryInfo,
    flags: &FormatFlags,
) -> Option<usize> {
    let mut pos = 0;
    let is_negative = value < 0.0;
    let abs_value = fabs(value);

    let frac_digits = if flags.international {
        flags
            .right_precision
            .unwrap_or(monetary.int_frac_digits as usize)
    } else {
        flags
            .right_precision
            .unwrap_or(monetary.frac_digits as usize)
    };

    let scale = pow(10.0, frac_digits as f64);
    let mut int_part = trunc(abs_value) as i64;
    let mut frac_part = round((abs_value - int_part as f64) * scale) as i64;

    // Ensure that the fractional part (frac_part) doesn’t overflow due to rounding
    // when abs_value is very close to the next integer value.
    if frac_part >= scale as i64 {
        // Handle carry-over to the integer part
        int_part += 1;
        frac_part = 0;
    }

    let mut int_str = int_part.to_string();

    if let Some(left_prec) = flags.left_precision {
        if int_str.len() > left_prec {
            return None;
        }
        while int_str.len() < left_prec {
            int_str.insert(0, '0');
        }
    }

    // Apply grouping
    let mut grouped = String::with_capacity(int_str.len() * 2);
    let mut group_idx = 0;
    let mut count = 0;

    // The grouping array can have up to 4 elements, but the last element is always 0
    for c in int_str.chars().rev() {
        if count > 0 {
            let current_group = monetary.mon_grouping.get(group_idx).copied().unwrap_or(0);
            if current_group > 0 && count % current_group == 0 {
                grouped.push_str(monetary.mon_thousands_sep);
                group_idx += 1; // Move to the next grouping size
            }
        }
        grouped.push(c);
        count += 1;
    }
    // Reverse the string to get the correct order
    let mut result = String::with_capacity(grouped.len() + 20);

    // Add the sign position
    let sign_posn = if is_negative {
        monetary.n_sign_posn
    } else {
        monetary.p_sign_posn
    };

    // Add the sign
    let sign = if is_negative {
        monetary.negative_sign
    } else if flags.force_sign {
        monetary.positive_sign
    } else if flags.space_sign {
        " "
    } else {
        ""
    };

    // Add the sign at the beginning
    if sign_posn == 0 {
        result.push('(');
    } else if sign_posn == 1 {
        result.push_str(sign);
    }

    // Add the currency symbol
    let cs_precedes = if is_negative {
        monetary.n_cs_precedes
    } else {
        monetary.p_cs_precedes
    };

    // Add a space between the currency symbol and the value
    let sep_by_space = if is_negative {
        let scale = pow(10.0, frac_digits as f64);
        monetary.n_sep_by_space
    } else {
        monetary.p_sep_by_space
    };

    // Add the currency symbol
    let symbol = if flags.suppress_symbol {
        ""
    } else if flags.international {
        monetary.int_curr_symbol
    } else {
        monetary.currency_symbol
    };

    // Add the currency symbol
    if cs_precedes {
        result.push_str(symbol);
        if sep_by_space {
            result.push(' ');
        }
    }

    // Add the value
    result.push_str(&grouped.chars().rev().collect::<String>());

    if frac_digits > 0 {
        result.push_str(monetary.mon_decimal_point);
        result.push_str(&format!("{:0width$}", frac_part, width = frac_digits));
    }

    if !cs_precedes {
        // Add the currency symbol after the value
        if sep_by_space {
            result.push(' ');
        }
        result.push_str(symbol);
    }

    if sign_posn == 0 {
        result.push(')');
    }

    if let Some(width) = flags.field_width {
        // Check if the field width is specified
        if result.len() < width {
            let padding = " ".repeat(width - result.len());
            if flags.left_justify {
                result.push_str(&padding);
            } else {
                result = padding + &result;
            }
        }
    }

    // Write the formatted string to the buffer
    for (i, &b) in result.as_bytes().iter().enumerate() {
        if pos + i >= buffer.len() {
            return None;
        }
        buffer[pos + i] = b;
    }

    Some(pos + result.len())
}
