// https://pubs.opengroup.org/onlinepubs/7908799/xsh/strptime.html

use crate::{
    header::{string::strlen, time::tm},
    platform::types::size_t,
};
use alloc::{string::String, vec::Vec};
use core::{
    ffi::{CStr, c_char, c_int, c_void},
    mem::MaybeUninit,
    ptr,
    ptr::NonNull,
    slice, str,
};

/// For convenience, we define some helper constants for the C-locale.
const SHORT_DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const LONG_DAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
const SHORT_MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const LONG_MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strptime.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strptime(
    buf: *const c_char,
    format: *const c_char,
    tm: *mut tm,
) -> *mut c_char {
    // Validate inputs
    let buf_ptr = if let Some(ptr) = NonNull::new(buf as *const c_void as *mut c_void) {
        ptr
    } else {
        return ptr::null_mut();
    };
    //
    let fmt_ptr = if let Some(ptr) = NonNull::new(format as *const c_void as *mut c_void) {
        ptr
    } else {
        return ptr::null_mut();
    };

    let tm_ptr = if let Some(ptr) = NonNull::new(tm) {
        ptr
    } else {
        return ptr::null_mut();
    };

    // Convert raw pointers into slices/strings.
    let input_str = unsafe {
        if buf.is_null() {
            return ptr::null_mut();
        }
        match CStr::from_ptr(buf).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(), // Not a valid UTF-8
        }
    };

    let fmt_str = unsafe {
        if format.is_null() {
            return ptr::null_mut();
        }
        match CStr::from_ptr(format).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(), // Not a valid UTF-8
        }
    };

    // We parse the format specifiers in a loop
    let mut fmt_chars = fmt_str.chars().peekable();
    let mut index_in_input = 0;

    while let Some(fc) = fmt_chars.next() {
        if fc != '%' {
            // If it's a normal character, we expect it to match exactly in input
            if input_str.len() <= index_in_input {
                return ptr::null_mut(); // input ended too soon
            }
            let in_char = input_str.as_bytes()[index_in_input] as char;
            if in_char != fc {
                // mismatch
                return ptr::null_mut();
            }
            index_in_input += 1;
            continue;
        }

        // If we see '%', read the next character
        let Some(spec) = fmt_chars.next() else {
            // format string ended abruptly after '%'
            return ptr::null_mut();
        };

        // POSIX says `%E` or `%O` are modified specifiers for locale.
        // We will skip them if they appear (like strftime does) and read the next char.
        let final_spec = if spec == 'E' || spec == 'O' {
            match fmt_chars.next() {
                Some(ch) => ch,
                None => return ptr::null_mut(),
            }
        } else {
            spec
        };

        // Handle known specifiers
        match final_spec {
            ///////////////////////////
            // Whitespace: %n or %t  //
            ///////////////////////////
            'n' | 't' => {
                // Skip over any whitespace in the input
                while index_in_input < input_str.len()
                    && input_str.as_bytes()[index_in_input].is_ascii_whitespace()
                {
                    index_in_input += 1;
                }
            }

            ///////////////////////////
            // Literal % => "%%"     //
            ///////////////////////////
            '%' => {
                if index_in_input >= input_str.len()
                    || input_str.as_bytes()[index_in_input] as char != '%'
                {
                    return ptr::null_mut();
                }
                index_in_input += 1;
            }

            ///////////////////////////
            // Day of Month: %d / %e //
            ///////////////////////////
            'd' | 'e' => {
                // parse a 2-digit day (with or without leading zero)
                let (val, len) = match parse_int(&input_str[index_in_input..], 2, false) {
                    Some(v) => unsafe { v },
                    None => return ptr::null_mut(),
                };
                unsafe {
                    (*tm).tm_mday = val as c_int;
                    // Day of month is limited to [1,31] according to the standard
                    if (*tm).tm_mday < 1 || (*tm).tm_mday > 31 {
                        return ptr::null_mut();
                    }
                }
                index_in_input += len;
            }

            ///////////////////////////
            // Month: %m             //
            ///////////////////////////
            'm' => {
                // parse a 2-digit month
                let (val, len) = match parse_int(&input_str[index_in_input..], 2, false) {
                    Some(v) => v,
                    None => return ptr::null_mut(),
                };
                // tm_mon is 0-based (0 = Jan, 1 = Feb,...)
                unsafe {
                    (*tm).tm_mon = (val as c_int) - 1;
                    if (*tm).tm_mon < 0 || (*tm).tm_mon > 11 {
                        return ptr::null_mut();
                    }
                }
                index_in_input += len;
            }

            //////////////////////////////
            // Year without century: %y //
            //////////////////////////////
            'y' => {
                // parse a 2-digit year
                let (val, len) = match parse_int(&input_str[index_in_input..], 2, false) {
                    Some(v) => v,
                    None => return ptr::null_mut(),
                };
                // According to POSIX, %y in strptime is [00,99], and the "year" is 1900..1999 for [00..99],
                // but the standard says: "values in [69..99] refer to 1969..1999, [00..68] => 2000..2068"
                let fullyear = if val >= 69 { val + 1900 } else { val + 2000 };
                unsafe {
                    (*tm).tm_year = (fullyear - 1900) as c_int;
                }
                index_in_input += len;
            }

            ///////////////////////////
            // Year with century: %Y //
            ///////////////////////////
            'Y' => {
                // parse up to 4-digit (or more) year
                // We allow more than 4 digits if needed
                let (val, len) = match parse_int(&input_str[index_in_input..], 4, true) {
                    Some(v) => v,
                    None => return ptr::null_mut(),
                };
                unsafe {
                    (*tm).tm_year = (val as c_int) - 1900;
                }
                index_in_input += len;
            }

            ///////////////////////////
            // Hour (00..23): %H     //
            ///////////////////////////
            'H' => {
                let (val, len) = match parse_int(&input_str[index_in_input..], 2, false) {
                    Some(v) => v,
                    None => return ptr::null_mut(),
                };
                if val > 23 {
                    return ptr::null_mut();
                }
                unsafe {
                    (*tm).tm_hour = val as c_int;
                }
                index_in_input += len;
            }

            ///////////////////////////
            // Hour (01..12): %I     //
            ///////////////////////////
            'I' => {
                let (val, len) = match parse_int(&input_str[index_in_input..], 2, false) {
                    Some(v) => v,
                    None => return ptr::null_mut(),
                };
                if val < 1 || val > 12 {
                    return ptr::null_mut();
                }
                unsafe {
                    (*tm).tm_hour = val as c_int;
                }
                // We’ll interpret AM/PM with %p if it appears later
                index_in_input += len;
            }

            ///////////////////////////
            // Minute (00..59): %M   //
            ///////////////////////////
            'M' => {
                let (val, len) = match parse_int(&input_str[index_in_input..], 2, false) {
                    Some(v) => v,
                    None => return ptr::null_mut(),
                };
                if val > 59 {
                    return ptr::null_mut();
                }
                unsafe {
                    (*tm).tm_min = val as c_int;
                }
                index_in_input += len;
            }

            ///////////////////////////
            // Seconds (00..60): %S  //
            ///////////////////////////
            'S' => {
                let (val, len) = match parse_int(&input_str[index_in_input..], 2, false) {
                    Some(v) => v,
                    None => return ptr::null_mut(),
                };
                if val > 60 {
                    return ptr::null_mut();
                }
                unsafe {
                    (*tm).tm_sec = val as c_int;
                }
                index_in_input += len;
            }

            ///////////////////////////
            // AM/PM: %p             //
            ///////////////////////////
            'p' => {
                // Parse either "AM" or "PM" (no case-sensitive)
                // We'll read up to 2 or 3 letters from input ("AM", "PM")
                let leftover = &input_str[index_in_input..];
                let parsed_len = match parse_am_pm(leftover) {
                    Some((is_pm, used)) => {
                        if unsafe { (*tm).tm_hour } == 12 {
                            // 12 AM => 00:xx, 12 PM => 12:xx
                            unsafe {
                                (*tm).tm_hour = if is_pm { 12 } else { 0 };
                            }
                        } else {
                            // 1..11 AM => 1..11, 1..11 PM => 13..23
                            if is_pm {
                                unsafe {
                                    (*tm).tm_hour += 12;
                                }
                            }
                        }
                        used
                    }
                    None => return ptr::null_mut(),
                };
                index_in_input += parsed_len;
            }

            ///////////////////////////
            // Weekday Name: %a/%A   //
            ///////////////////////////
            'a' => {
                // Abbreviated day name (Sun..Sat)
                let leftover = &input_str[index_in_input..];
                let parsed_len = match parse_weekday(leftover, true) {
                    Some((wday, used)) => {
                        unsafe {
                            (*tm).tm_wday = wday as c_int;
                        }
                        used
                    }
                    None => return ptr::null_mut(),
                };
                index_in_input += parsed_len;
            }
            'A' => {
                // Full day name (Sunday..Saturday)
                let leftover = &input_str[index_in_input..];
                let parsed_len = match parse_weekday(leftover, false) {
                    Some((wday, used)) => {
                        unsafe {
                            (*tm).tm_wday = wday as c_int;
                        }
                        used
                    }
                    None => return ptr::null_mut(),
                };
                index_in_input += parsed_len;
            }

            ///////////////////////////
            // Month Name: %b/%B/%h  //
            ///////////////////////////
            'b' | 'h' => {
                // Abbreviated month name
                let leftover = &input_str[index_in_input..];
                let parsed_len = match parse_month(leftover, true) {
                    Some((mon, used)) => {
                        unsafe {
                            (*tm).tm_mon = mon as c_int;
                        }
                        used
                    }
                    None => return ptr::null_mut(),
                };
                index_in_input += parsed_len;
            }
            'B' => {
                // Full month name
                let leftover = &input_str[index_in_input..];
                let parsed_len = match parse_month(leftover, false) {
                    Some((mon, used)) => {
                        unsafe {
                            (*tm).tm_mon = mon as c_int;
                        }
                        used
                    }
                    None => return ptr::null_mut(),
                };
                index_in_input += parsed_len;
            }

            ///////////////////////////
            // Day of year: %j       //
            ///////////////////////////
            'j' => {
                // parse 3-digit day of year [001..366]
                let (val, len) = match parse_int(&input_str[index_in_input..], 3, false) {
                    Some(v) => v,
                    None => return ptr::null_mut(),
                };
                if val < 1 || val > 366 {
                    return ptr::null_mut();
                }
                // store in tm_yday
                unsafe {
                    (*tm).tm_yday = (val - 1) as c_int;
                }
                index_in_input += len;
            }

            //////////////////////////////////
            // Date shortcuts: %D, %F, etc. //
            //////////////////////////////////
            'D' => {
                // Equivalent to "%m/%d/%y"
                // We can do a mini strptime recursion or manually parse
                // For simplicity, we'll do it inline here
                let subfmt = "%m/%d/%y";
                let used =
                    match unsafe { apply_subformat(&input_str[index_in_input..], subfmt, tm) } {
                        Some(v) => v,
                        None => return ptr::null_mut(),
                    };
                index_in_input += used;
            }
            'F' => {
                // Equivalent to "%Y-%m-%d"
                let subfmt = "%Y-%m-%d";
                let used =
                    match unsafe { apply_subformat(&input_str[index_in_input..], subfmt, tm) } {
                        Some(v) => v,
                        None => return ptr::null_mut(),
                    };
                index_in_input += used;
            }
            'T' => {
                // Equivalent to %H:%M:%S
                let subfmt = "%H:%M:%S";
                let used =
                    match unsafe { apply_subformat(&input_str[index_in_input..], subfmt, tm) } {
                        Some(v) => v,
                        None => return ptr::null_mut(),
                    };
                index_in_input += used;
            }

            //////////////////////////////////////////////////////////
            // TODO : not implemented: %x, %X, %c, %r, %R, etc. //
            //////////////////////////////////////////////////////////
            // Hint : if you want to implement these, do similarly to %D / %F (or parse manually)
            'x' | 'X' | 'c' | 'r' | 'R' => {
                // Return NULL if we don’t want to accept them :
                return ptr::null_mut();
            }

            ///////////////////////////
            // Timezone: %Z or %z    //
            ///////////////////////////
            'Z' | 'z' => {
                // Full/abbrev time zone name or numeric offset
                // Implementation omitted. Real support is quite complicated.
                return ptr::null_mut();
            }

            //////////
            // else //
            //////////
            _ => {
                // We do not recognize this specifier
                return ptr::null_mut();
            }
        }
    }

    // If we got here, parsing was successful. Return pointer to the
    // next unparsed character in `buf`.
    let ret_ptr = unsafe { buf.add(index_in_input) };
    ret_ptr as *mut c_char
}

// -----------------------
// Helper / Parsing Logic
// -----------------------

/// Parse an integer from the beginning of `input_str`.
///
/// - `width` is the maximum number of digits to parse
/// - `allow_variable_width` indicates if we can parse fewer digits
///   (e.g., `%Y` can have more than 4 digits, but also might parse "2023" or "12345").
fn parse_int(input: &str, width: usize, allow_variable: bool) -> Option<(i32, usize)> {
    let mut val = 0i32;
    let mut chars = input.chars();
    let mut count = 0;

    while let Some(c) = chars.next() {
        if !c.is_ascii_digit() {
            break;
        }

        // Check for integer overflow
        val = val.checked_mul(10)?.checked_add((c as u8 - b'0') as i32)?;

        count += 1;
        if count == width && !allow_variable {
            break;
        }
    }

    if count == 0 { None } else { Some((val, count)) }
}

/// Handle AM/PM. Returns (is_pm, length_consumed).
/// Accepts "AM", "am", "PM", "pm" case-insensitively.
fn parse_am_pm(s: &str) -> Option<(bool, usize)> {
    let trimmed = s.trim_start();
    // Amount of whitespace skipped; can be 0
    let diff = s.len() - trimmed.len();
    let s = trimmed.get(0..2)?;

    if s.eq_ignore_ascii_case("AM") {
        return Some((false, diff + 2));
    }
    if s.eq_ignore_ascii_case("PM") {
        return Some((true, diff + 2));
    }
    None
}

/// Parse a weekday name from `s`.
/// - if `abbrev == true`, match short forms: "Mont".."Sun"
/// - otherwise, match "Monday".."Sunday"
/// Return (weekday_index, length_consumed).
fn parse_weekday(s: &str, abbrev: bool) -> Option<(usize, usize)> {
    let list = if abbrev { &SHORT_DAYS } else { &LONG_DAYS };
    for (i, name) in list.iter().enumerate() {
        if s.len() >= name.len() && s[0..name.len()].eq_ignore_ascii_case(name) {
            return Some((i, name.len()));
        }
    }
    None
}

/// Parse a month name from `s`.
/// - If `abbrev == true`, match short forms: "Jan".."Dec"
/// - Otherwise, match "January".."December"
/// Return (month_index, length_consumed).
fn parse_month(s: &str, abbrev: bool) -> Option<(usize, usize)> {
    let list = if abbrev { &SHORT_MONTHS } else { &LONG_MONTHS };
    for (i, name) in list.iter().enumerate() {
        if s.len() >= name.len() && s[0..name.len()].eq_ignore_ascii_case(name) {
            return Some((i, name.len()));
        }
    }
    None
}

/// Apply a small subformat (like "%m/%d/%y" or "%Y-%m-%d") to `input`.
/// Return how many characters of `input` were consumed or None on error.
unsafe fn apply_subformat(input: &str, subfmt: &str, tm: *mut tm) -> Option<usize> {
    // We'll do a temporary strptime call on a substring.
    // Then we see how many chars it consumed. If that call fails, we return None.
    // Otherwise, we return the count.

    // Convert `input` to a null-terminated buffer temporarily
    let mut tmpbuf = String::with_capacity(input.len() + 1);
    tmpbuf.push_str(input);
    tmpbuf.push('\0');

    let mut tmpfmt = String::with_capacity(subfmt.len() + 1);
    tmpfmt.push_str(subfmt);
    tmpfmt.push('\0');

    // We need a copy of the tm, so if partial parse fails, we don't override.
    let old_tm = unsafe { ptr::read(tm) }; // backup

    let consumed_ptr = unsafe {
        strptime(
            tmpbuf.as_ptr() as *const c_char,
            tmpfmt.as_ptr() as *const c_char,
            tm,
        )
    };

    if consumed_ptr.is_null() {
        // revert
        unsafe {
            *tm = old_tm;
        }
        return None;
    }

    // consumed_ptr - tmpbuf.as_ptr() => # of bytes consumed
    let diff = (consumed_ptr as usize) - (tmpbuf.as_ptr() as usize);
    Some(diff)
}

#[cfg(test)]
mod tests {
    use super::parse_am_pm;

    #[test]
    fn am_pm_parser_works() {
        let am = "am";
        let am_expected = Some((false, 2));
        assert_eq!(am_expected, parse_am_pm(am));

        let pm = "pm";
        let pm_expected = Some((true, 2));
        assert_eq!(pm_expected, parse_am_pm(pm));

        let am_caps = "AM";
        assert_eq!(am_expected, parse_am_pm(am_caps));

        let pm_caps = "PM";
        assert_eq!(pm_expected, parse_am_pm(pm_caps));

        let am_weird = "aM";
        assert_eq!(am_expected, parse_am_pm(am_weird));

        let am_prefix = "        \tam";
        let am_prefix_expected = Some((false, 11));
        assert_eq!(am_prefix_expected, parse_am_pm(am_prefix));

        let pm_spaces = "        pm        ";
        let pm_spaces_expected = Some((true, 10));
        assert_eq!(pm_spaces_expected, parse_am_pm(pm_spaces));
    }
}
