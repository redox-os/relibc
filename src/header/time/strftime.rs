// `strftime` implementation.
//
// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strftime.html>.

use alloc::string::String;

use super::{get_offset, tm};
use crate::{
    c_str::CStr,
    platform::{
        self, WriteByte,
        types::{c_char, c_int, size_t},
    },
};

// We use the langinfo constants
use crate::header::langinfo::{
    ABDAY_1,
    ABMON_1,
    AM_STR,
    DAY_1,
    MON_1,
    PM_STR,
    // TODO : other constants if needed
    nl_item,
    nl_langinfo,
};

/// A helper that calls `nl_langinfo(item)` and converts the returned pointer
/// into a `&str`. If it fails or is null, returns an empty string "".
unsafe fn langinfo_to_str(item: nl_item) -> &'static str {
    use core::ffi::CStr;

    let ptr = unsafe { nl_langinfo(item) };
    if ptr.is_null() {
        return "";
    }
    match unsafe { CStr::from_ptr(ptr) }.to_str() {
        Ok(s) => s,
        Err(_) => "",
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strftime.html>.
///
/// Formats time data according to the given `format` string.
///
/// Use `langinfo` for locale-based day/month names,
/// but still hard-codes other aspects of the "C" locale (like numeric/date
/// formats) and ignores `%E` / `%O` variations.
pub unsafe fn strftime<W: WriteByte>(w: &mut W, format: *const c_char, t: *const tm) -> size_t {
    /// Helper that actually parses the format string and writes output.
    pub unsafe fn inner_strftime<W: WriteByte>(
        w: &mut W,
        mut format: *const c_char,
        t: *const tm,
    ) -> bool {
        macro_rules! w {
            (byte $b:expr) => {{
                if w.write_u8($b).is_err() {
                    return false;
                }
            }};
            (char $chr:expr) => {{
                if w.write_char($chr).is_err() {
                    return false;
                }
            }};
            (recurse $fmt:expr) => {{
                let mut tmp = String::with_capacity($fmt.len() + 1);
                tmp.push_str($fmt);
                tmp.push('\0');

                if unsafe { !inner_strftime(w, tmp.as_ptr() as *mut c_char, t) } {
                    return false;
                }
            }};
            ($str:expr) => {{
                if w.write_str($str).is_err() {
                    return false;
                }
            }};
            ($fmt:expr, $($args:expr),+) => {{
                // Could use write!() if we didn't need the exact count
                if write!(w, $fmt, $($args),+).is_err() {
                    return false;
                }
            }};
        }

        while unsafe { *format } != 0 {
            // If the character isn't '%', just copy it out.
            if unsafe { *format } as u8 != b'%' {
                w!(byte unsafe { *format } as u8);
                format = unsafe { format.offset(1) };
                continue;
            }

            // Skip '%'
            format = unsafe { format.offset(1) };

            // POSIX says '%E' and '%O' can modify numeric formats for locales,
            // but we ignore them in this minimal "C" locale approach.
            if unsafe { *format } as u8 == b'E' || unsafe { *format } as u8 == b'O' {
                format = unsafe { format.offset(1) };
            }

            match unsafe { *format } as u8 {
                // Literal '%'
                b'%' => w!(byte b'%'),

                // Newline and tab expansions
                b'n' => w!(byte b'\n'),
                b't' => w!(byte b'\t'),

                // Abbreviated weekday name: %a
                b'a' => {
                    // `ABDAY_1 + tm_wday` is the correct langinfo ID for abbreviated weekdays
                    let s = unsafe { langinfo_to_str(ABDAY_1 + (*t).tm_wday as i32) };
                    w!(s);
                }

                // Full weekday name: %A
                b'A' => {
                    // `DAY_1 + tm_wday` is the correct langinfo ID for full weekdays
                    let s = unsafe { langinfo_to_str(DAY_1 + (*t).tm_wday as i32) };
                    w!(s);
                }

                // Abbreviated month name: %b or %h
                b'b' | b'h' => {
                    let s = unsafe { langinfo_to_str(ABMON_1 + (*t).tm_mon as i32) };
                    w!(s);
                }

                // Full month name: %B
                b'B' => {
                    let s = unsafe { langinfo_to_str(MON_1 + (*t).tm_mon as i32) };
                    w!(s);
                }

                // Century: %C
                b'C' => {
                    let mut year = unsafe { (*t).tm_year } / 100;
                    if unsafe { (*t).tm_year } % 100 != 0 {
                        year += 1;
                    }
                    w!("{:02}", year + 19);
                }

                // Day of month: %d
                b'd' => w!("{:02}", unsafe { (*t).tm_mday }),

                // %D => same as %m/%d/%y
                b'D' => w!(recurse "%m/%d/%y"),

                // Day of month, space-padded: %e
                b'e' => w!("{:2}", unsafe { (*t).tm_mday }),

                // ISO 8601 date: %F => %Y-%m-%d
                b'F' => w!(recurse "%Y-%m-%d"),

                // Hour (00-23): %H
                b'H' => w!("{:02}", unsafe { (*t).tm_hour }),

                // Hour (01-12): %I
                b'I' => w!("{:02}", (unsafe { (*t).tm_hour } + 12 - 1) % 12 + 1),

                // Day of year: %j
                b'j' => w!("{:03}", unsafe { (*t).tm_yday } + 1),

                // etc.
                b'k' => w!("{:2}", unsafe { (*t).tm_hour }),
                b'l' => w!("{:2}", (unsafe { (*t).tm_hour } + 12 - 1) % 12 + 1),
                b'm' => w!("{:02}", unsafe { (*t).tm_mon } + 1),
                b'M' => w!("{:02}", unsafe { (*t).tm_min }),

                // AM/PM (uppercase): %p
                b'p' => {
                    // Get "AM" / "PM" from langinfo
                    if unsafe { (*t).tm_hour } < 12 {
                        w!(unsafe { langinfo_to_str(AM_STR) });
                    } else {
                        w!(unsafe { langinfo_to_str(PM_STR) });
                    }
                }

                // am/pm (lowercase): %P
                b'P' => {
                    // Convert the AM_STR / PM_STR to lowercase
                    if unsafe { (*t).tm_hour } < 12 {
                        let am = unsafe { langinfo_to_str(AM_STR) }.to_ascii_lowercase();
                        w!(&am);
                    } else {
                        let pm = unsafe { langinfo_to_str(PM_STR) }.to_ascii_lowercase();
                        w!(&pm);
                    }
                }

                // 12-hour clock with seconds + AM/PM: %r => %I:%M:%S %p
                b'r' => w!(recurse "%I:%M:%S %p"),

                // 24-hour clock without seconds: %R => %H:%M
                b'R' => w!(recurse "%H:%M"),

                // Seconds since the Epoch: %s => calls mktime() to convert tm to time_t
                b's' => w!("{}", unsafe { super::mktime(t as *mut tm) }),

                // Seconds (00-60): %S (unchanged)
                b'S' => w!("{:02}", unsafe { (*t).tm_sec }),

                // 24-hour clock with seconds: %T => %H:%M:%S
                b'T' => w!(recurse "%H:%M:%S"),

                // Weekday (1-7, Monday=1): %u
                b'u' => w!("{}", (unsafe { (*t).tm_wday } + 7 - 1) % 7 + 1),

                // Sunday-based week of year: %U
                b'U' => w!(
                    "{}",
                    (unsafe { (*t).tm_yday } + 7 - unsafe { (*t).tm_wday }) / 7
                ),

                // ISO-8601 week of year
                b'V' => w!("{}", week_of_year(unsafe { &*t })),

                // Weekday (0-6, Sunday=0): %w
                b'w' => w!("{}", unsafe { (*t).tm_wday }),

                // Monday-based week of year: %W
                b'W' => w!(
                    "{}",
                    (unsafe { (*t).tm_yday } + 7 - (unsafe { (*t).tm_wday } + 6) % 7) / 7
                ),

                // Last two digits of year: %y
                b'y' => w!("{:02}", unsafe { (*t).tm_year } % 100),

                // Full year: %Y
                b'Y' => w!("{}", unsafe { (*t).tm_year } + 1900),

                // Timezone offset: %z
                b'z' => {
                    let offset = unsafe { (*t).tm_gmtoff };
                    let (sign, offset) = if offset < 0 {
                        ('-', -offset)
                    } else {
                        ('+', offset)
                    };
                    let mins = offset.div_euclid(60);
                    let min = mins.rem_euclid(60);
                    let hour = mins.div_euclid(60);
                    w!("{}{:02}{:02}", sign, hour, min)
                }

                // Timezone name: %Z
                b'Z' => w!(
                    "{}",
                    unsafe { CStr::from_ptr((*t).tm_zone) }.to_str().unwrap()
                ),

                // Date+time+TZ: %+
                b'+' => w!(recurse "%a %b %d %T %Z %Y"),

                // Unrecognized format specifier => fail
                _ => return false,
            }

            // Move past the format specifier
            format = unsafe { format.offset(1) };
        }
        true
    }

    // Wrap the writer in a CountingWriter to return how many bytes were written.
    let mut cw = platform::CountingWriter::new(w);
    if unsafe { !inner_strftime(&mut cw, format, t) } {
        return 0;
    }
    cw.written
}

/// Calculate number of weeks in a year as defined by ISO 8601
///
/// ## Source
/// https://en.wikipedia.org/wiki/ISO_week_date
fn weeks_per_year(year: c_int) -> c_int {
    let year = year as f64;
    let p_y = (year + (year / 4.) - (year / 100.) + (year / 400.)) as c_int % 7;
    if p_y == 4 { 53 } else { 52 }
}

/// Calculate the week of the year accounting for leap weeks (ISO 8601)
///
/// ## Source
/// https://en.wikipedia.org/wiki/ISO_week_date
fn week_of_year(time: &tm) -> c_int {
    let week = (10 + time.tm_yday - time.tm_wday) / 7;

    if week <= 1 {
        weeks_per_year(time.tm_year - 1)
    } else if week > weeks_per_year(time.tm_year) {
        1
    } else {
        week
    }
}
