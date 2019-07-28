use alloc::string::String;

use crate::platform::{self, types::*, WriteByte};

use super::tm;

pub unsafe fn strftime<W: WriteByte>(w: &mut W, format: *const c_char, t: *const tm) -> size_t {
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
                let mut fmt = String::with_capacity($fmt.len() + 1);
                fmt.push_str($fmt);
                fmt.push('\0');

                if !inner_strftime(w, fmt.as_ptr() as *mut c_char, t) {
                    return false;
                }
            }};
            ($str:expr) => {{
                if w.write_str($str).is_err() {
                    return false;
                }
            }};
            ($fmt:expr, $($args:expr),+) => {{
                // Would use write!() if I could get the length written
                if write!(w, $fmt, $($args),+).is_err() {
                    return false;
                }
            }};
        }
        const WDAYS: [&str; 7] = [
            "Sunday",
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
        ];
        const MONTHS: [&str; 12] = [
            "January",
            "Febuary",
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

        while *format != 0 {
            if *format as u8 != b'%' {
                w!(byte * format as u8);
                format = format.offset(1);
                continue;
            }

            format = format.offset(1);

            if *format as u8 == b'E' || *format as u8 == b'O' {
                // Ignore because these do nothing without locale
                format = format.offset(1);
            }

            match *format as u8 {
                b'%' => w!(byte b'%'),
                b'n' => w!(byte b'\n'),
                b't' => w!(byte b'\t'),
                b'a' => w!(&WDAYS[(*t).tm_wday as usize][..3]),
                b'A' => w!(WDAYS[(*t).tm_wday as usize]),
                b'b' | b'h' => w!(&MONTHS[(*t).tm_mon as usize][..3]),
                b'B' => w!(MONTHS[(*t).tm_mon as usize]),
                b'C' => {
                    let mut year = (*t).tm_year / 100;
                    // Round up
                    if (*t).tm_year % 100 != 0 {
                        year += 1;
                    }
                    w!("{:02}", year + 19);
                }
                b'd' => w!("{:02}", (*t).tm_mday),
                b'D' => w!(recurse "%m/%d/%y"),
                b'e' => w!("{:2}", (*t).tm_mday),
                b'F' => w!(recurse "%Y-%m-%d"),
                b'H' => w!("{:02}", (*t).tm_hour),
                b'I' => w!("{:02}", ((*t).tm_hour + 12 - 1) % 12 + 1),
                b'j' => w!("{:03}", (*t).tm_yday),
                b'k' => w!("{:2}", (*t).tm_hour),
                b'l' => w!("{:2}", ((*t).tm_hour + 12 - 1) % 12 + 1),
                b'm' => w!("{:02}", (*t).tm_mon + 1),
                b'M' => w!("{:02}", (*t).tm_min),
                b'p' => w!(if (*t).tm_hour < 12 { "AM" } else { "PM" }),
                b'P' => w!(if (*t).tm_hour < 12 { "am" } else { "pm" }),
                b'r' => w!(recurse "%I:%M:%S %p"),
                b'R' => w!(recurse "%H:%M"),
                // Nothing is modified in mktime, but the C standard of course requires a mutable pointer ._.
                b's' => w!("{}", super::mktime(t as *mut tm)),
                b'S' => w!("{:02}", (*t).tm_sec),
                b'T' => w!(recurse "%H:%M:%S"),
                b'u' => w!("{}", ((*t).tm_wday + 7 - 1) % 7 + 1),
                b'U' => w!("{}", ((*t).tm_yday + 7 - (*t).tm_wday) / 7),
                b'w' => w!("{}", (*t).tm_wday),
                b'W' => w!("{}", ((*t).tm_yday + 7 - ((*t).tm_wday + 6) % 7) / 7),
                b'y' => w!("{:02}", (*t).tm_year % 100),
                b'Y' => w!("{}", (*t).tm_year + 1900),
                b'z' => w!("+0000"), // TODO
                b'Z' => w!("UTC"),   // TODO
                b'+' => w!(recurse "%a %b %d %T %Z %Y"),
                _ => return false,
            }

            format = format.offset(1);
        }
        true
    }

    let mut w = platform::CountingWriter::new(w);
    if !inner_strftime(&mut w, format, t) {
        return 0;
    }

    w.written
}
