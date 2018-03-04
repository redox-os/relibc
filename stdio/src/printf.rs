use core::fmt;

use platform::types::*;
use vl::VaList;

pub unsafe fn printf<W: fmt::Write>(mut w: W, format: *const c_char, mut ap: VaList) -> c_int {
    use core::fmt::Write;
    use core::slice;
    use core::str;

    extern "C" {
        fn strlen(s: *const c_char) -> size_t;
    }

    let format = slice::from_raw_parts(format as *const u8, strlen(format));

    let mut i = 0;
    let mut found_percent = false;
    while i < format.len() {
        let b = format[i];

        if found_percent {
            match b as char {
                '%' => {
                    w.write_char('%');
                    found_percent = false;
                },
                'c' => {
                    let a = ap.get::<u32>();

                    w.write_char(a as u8 as char);

                    found_percent = false;
                },
                'd' | 'i' => {
                    let a = ap.get::<c_int>();

                    w.write_fmt(format_args!("{}", a));

                    found_percent = false;
                },
                'n' => {
                    let _a = ap.get::<c_int>();

                    found_percent = false;
                },
                'p' => {
                    let a = ap.get::<usize>();

                    w.write_fmt(format_args!("0x{:x}", a));

                    found_percent = false;
                },
                's' => {
                    let a = ap.get::<usize>();

                    w.write_str(str::from_utf8_unchecked(
                        slice::from_raw_parts(a as *const u8, strlen(a as *const c_char))
                    ));

                    found_percent = false;
                },
                'u' => {
                    let a = ap.get::<c_uint>();

                    w.write_fmt(format_args!("{}", a));

                    found_percent = false;
                },
                'x' => {
                    let a = ap.get::<c_uint>();

                    w.write_fmt(format_args!("{:x}", a));

                    found_percent = false;
                },
                'X' => {
                    let a = ap.get::<c_uint>();

                    w.write_fmt(format_args!("{:X}", a));

                    found_percent = false;
                },
                '-' => {},
                '+' => {},
                ' ' => {},
                '#' => {},
                '0' ... '9' => {},
                _ => {}
            }
        } else if b == b'%' {
            found_percent = true;
        } else {
            w.write_char(b as char);
        }

        i += 1;
    }

    0
}
