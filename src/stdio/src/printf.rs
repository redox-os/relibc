use core::{fmt, slice, str};

use platform::{self, Write};
use platform::types::*;
use vl::VaList;

pub unsafe fn printf<W: Write>(mut w: W, format: *const c_char, mut ap: VaList) -> c_int {
    let format = unsafe { slice::from_raw_parts(format as *const u8, usize::max_value()) };

    let mut found_percent = false;
    for &b in format.iter() {
        // check for NUL
        if b == 0 {
            break;
        }

        if found_percent {
            match b as char {
                '%' => {
                    w.write_char('%');
                    found_percent = false;
                }
                'c' => {
                    let a = ap.get::<u32>();

                    w.write_u8(a as u8);

                    found_percent = false;
                }
                'd' | 'i' => {
                    let a = ap.get::<c_int>();

                    w.write_fmt(format_args!("{}", a));

                    found_percent = false;
                }
                'f' | 'F' => {
                    let a = ap.get::<f64>();

                    w.write_fmt(format_args!("{}", a));

                    found_percent = false;
                }
                'n' => {
                    let _a = ap.get::<c_int>();

                    found_percent = false;
                }
                'p' => {
                    let a = ap.get::<usize>();

                    w.write_fmt(format_args!("0x{:x}", a));

                    found_percent = false;
                }
                's' => {
                    let a = ap.get::<usize>();

                    w.write_str(str::from_utf8_unchecked(platform::c_str(
                        a as *const c_char,
                    )));

                    found_percent = false;
                }
                'u' => {
                    let a = ap.get::<c_uint>();

                    w.write_fmt(format_args!("{}", a));

                    found_percent = false;
                }
                'x' => {
                    let a = ap.get::<c_uint>();

                    w.write_fmt(format_args!("{:x}", a));

                    found_percent = false;
                }
                'X' => {
                    let a = ap.get::<c_uint>();

                    w.write_fmt(format_args!("{:X}", a));

                    found_percent = false;
                }
                'o' => {
                    let a = ap.get::<c_uint>();

                    w.write_fmt(format_args!("{:o}", a));

                    found_percent = false;
                }
                '-' => {}
                '+' => {}
                ' ' => {}
                '#' => {}
                '0'...'9' => {}
                _ => {}
            }
        } else if b == b'%' {
            found_percent = true;
        } else {
            w.write_u8(b);
        }
    }

    0
}
