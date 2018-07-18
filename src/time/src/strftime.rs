use alloc::string::String;
use platform::types::*;
use platform::Write;
use tm;

pub unsafe fn strftime<W: Write>(
    toplevel: bool,
    mut w: &mut W,
    maxsize: usize,
    mut format: *const c_char,
    t: *const tm,
) -> size_t {
    let mut written = 0;
    if toplevel {
        // Reserve nul byte
        written += 1;
    }
    macro_rules! w {
        (reserve $amount:expr) => {{
            if written + $amount > maxsize {
                return 0;
            }
            written += $amount;
        }};
        (byte $b:expr) => {{
            w!(reserve 1);
            w.write_u8($b);
        }};
        (char $chr:expr) => {{
            w!(reserve $chr.len_utf8());
            w.write_char($chr);
        }};
        (recurse $fmt:expr) => {{
            let mut fmt = String::with_capacity($fmt.len() + 1);
            fmt.push_str($fmt);
            fmt.push('\0');

            let count = strftime(false, w, maxsize - written, fmt.as_ptr() as *mut c_char, t);
            if count == 0 {
                return 0;
            }
            written += count;
            assert!(written <= maxsize);
        }};
        ($str:expr) => {{
            w!(reserve $str.len());
            w.write_str($str);
        }};
        ($fmt:expr, $($args:expr),+) => {{
            // Would use write!() if I could get the length written
            w!(&format!($fmt, $($args),+))
        }};
    }
    const WDAYS: [&'static str; 7] = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    const MONTHS: [&'static str; 12] = [
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
            w!(byte(*format as u8));
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
            b's' => w!("{}", ::mktime(t)),
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
            _ => return 0,
        }

        format = format.offset(1);
    }
    if toplevel {
        // nul byte is already counted in written
        w.write_u8(0);
    }
    written
}
