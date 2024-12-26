use super::{FormatFlags, LocaleMonetaryInfo, DEFAULT_MONETARY};
use alloc::string::{String, ToString};
use core::{ffi::CStr, ptr, slice, str};
use libm::{fabs, floor, pow, round, trunc};

/*
   The strfmon() function formats a monetary value according to the format string format and writes the result to the character
   array s of size maxsize.
   The format string format is a string that can contain plain characters and format specifiers.
   The format specifiers start with the character '%' and are followed by one or more flags, an optional field width,
   an optional left precision, an optional right precision, and a conversion specifier.
   The conversion specifier can be either 'i' or 'n'. The 'i' specifier formats the monetary value according to the
   locale-specific rules, while the 'n' specifier formats the monetary value according to the international rules.

   The function returns the number of characters written to the buffer, excluding the null terminator, or -1 if an error occurred.

   TODO : better handling error : always return -1 if an error occurred
*/
pub unsafe extern "C" fn strfmon(
    s: *mut i8,        // char *
    maxsize: usize,    // size_t
    format: *const i8, // const char *
    mut args: ...
) -> isize {
    if s.is_null() || format.is_null() || maxsize == 0 {
        // Check for null pointers and zero size
        return -1;
    }

    let format_str = match unsafe { CStr::from_ptr(format) }.to_str() {
        Ok(s) => s,          // Convert the format string to a string
        Err(_) => return -1, // Return -1 if the format string is not valid
    };
    // Buffer to write the formatted string
    let buffer = unsafe { slice::from_raw_parts_mut(s as *mut u8, maxsize) };
    let mut pos = 0;
    let mut format_chars = format_str.chars().peekable();

    while let Some(c) = format_chars.next() {
        if c != '%' {
            if pos >= buffer.len() {
                return -1;
            }
            // Write the character to the buffer
            buffer[pos] = c as u8; // Write the character to the buffer
            pos += 1; // Move to the next position
            continue; // Move to the next character
        }
        // Handle '%%' as an escape sequence
        if format_chars.peek() == Some(&'%') {
            if pos >= buffer.len() {
                return -1; // Buffer overflow
            }
            buffer[pos] = b'%'; // Write '%' to the buffer
            pos += 1; // Move to the next position
            format_chars.next(); // Skip the second '%'
            continue;
        }

        let mut flags = FormatFlags::default();

        // Parse flags
        while let Some(&next_char) = format_chars.peek() {
            match next_char {
                '=' => flags.left_justify = true,
                '+' => flags.force_sign = true,
                ' ' => flags.space_sign = true,
                '(' => flags.use_parens = true,
                '!' => flags.suppress_symbol = true,
                _ => break,
            }
            format_chars.next();
        }

        // Parse field width
        let mut num_str = String::new();
        while let Some(&c) = format_chars.peek() {
            if !c.is_ascii_digit() {
                break;
            }
            num_str.push(c); // Add the character to the number string
            format_chars.next(); // Move to the next character
        }
        if !num_str.is_empty() {
            flags.field_width = num_str.parse().ok(); // Parse the number string
        }

        // Parse left precision
        if format_chars.peek() == Some(&'#') {
            format_chars.next(); // Skip the '#'
            num_str.clear(); // Clear the number string
            while let Some(&c) = format_chars.peek() {
                if !c.is_ascii_digit() {
                    // Check if the character is a digit
                    break;
                }
                num_str.push(c);
                format_chars.next();
            }
            flags.left_precision = num_str.parse().ok();
        }

        // Parse right precision
        if format_chars.peek() == Some(&'.') {
            format_chars.next();
            num_str.clear();
            while let Some(&c) = format_chars.peek() {
                if !c.is_ascii_digit() {
                    break;
                }
                num_str.push(c);
                format_chars.next();
            }
            flags.right_precision = num_str.parse().ok();
        }

        match format_chars.next() {
            Some('i') => {
                flags.international = true;
                let value = unsafe { args.arg::<f64>() };
                if let Some(written) =
                    format_monetary(&mut buffer[pos..], value, &DEFAULT_MONETARY, &flags)
                {
                    pos += written;
                } else {
                    return -1;
                }
            }
            Some('n') => {
                let value = unsafe { args.arg::<f64>() }; // Get the next argument as a f64
                if let Some(written) =
                    format_monetary(&mut buffer[pos..], value, &DEFAULT_MONETARY, &flags)
                {
                    pos += written; // Move the position by the number of characters written
                } else {
                    return -1;
                }
            }
            _ => return -1,
        }
    }

    if pos >= buffer.len() {
        return -1;
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
    let mut pos = 0; // Initialize the position to 0
    let is_negative = value < 0.0; // Check if the value is negative
    let abs_value = fabs(value); // Get the absolute value of the number

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
        if count > 0
            && count % monetary.mon_grouping[group_idx.min(monetary.mon_grouping.len() - 1)] == 0
        {
            grouped.push_str(monetary.mon_thousands_sep);
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
        //let scale = my_pow10(frac_digits);
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
