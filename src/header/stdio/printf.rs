use core::fmt::Write as CoreWrite;
use core::{ptr, slice, str};

use c_str::CStr;
use platform::types::*;
use platform::{self, WriteByte};
use va_list::VaList;

pub unsafe fn printf<W: WriteByte>(w: W, format: *const c_char, mut ap: VaList) -> c_int {
    let mut w = platform::CountingWriter::new(w);

    let format = slice::from_raw_parts(format as *const u8, usize::max_value());

    let mut found_percent = false;
    for &b in format.iter() {
        // check for NUL
        if b == 0 {
            break;
        }

        if found_percent {
            if match b as char {
                '%' => {
                    found_percent = false;
                    w.write_char('%')
                }
                'c' => {
                    let a = ap.get::<u32>();

                    found_percent = false;

                    w.write_u8(a as u8)
                }
                'd' | 'i' => {
                    let a = ap.get::<c_int>();

                    found_percent = false;

                    w.write_fmt(format_args!("{}", a))
                }
                'f' | 'F' => {
                    let a = ap.get::<f64>();

                    found_percent = false;

                    w.write_fmt(format_args!("{}", a))
                }
                'n' => {
                    let _a = ap.get::<c_int>();

                    found_percent = false;
                    Ok(())
                }
                'p' => {
                    let a = ap.get::<usize>();

                    found_percent = false;

                    w.write_fmt(format_args!("0x{:x}", a))
                }
                's' => {
                    let a = ap.get::<*const c_char>();

                    found_percent = false;
                    if a != ptr::null() {
                        let a_cstr = CStr::from_ptr(a);
                        w.write_str(str::from_utf8_unchecked(a_cstr.to_bytes()))
                    } else {
                        w.write_str("NULL")
                    }
                }
                'u' => {
                    let a = ap.get::<c_uint>();

                    found_percent = false;

                    w.write_fmt(format_args!("{}", a))
                }
                'x' => {
                    let a = ap.get::<c_uint>();

                    found_percent = false;

                    w.write_fmt(format_args!("{:x}", a))
                }
                'X' => {
                    let a = ap.get::<c_uint>();

                    found_percent = false;

                    w.write_fmt(format_args!("{:X}", a))
                }
                'o' => {
                    let a = ap.get::<c_uint>();

                    found_percent = false;

                    w.write_fmt(format_args!("{:o}", a))
                }
                '-' => Ok(()),
                '+' => Ok(()),
                ' ' => Ok(()),
                '#' => Ok(()),
                '0'...'9' => Ok(()),
                _ => Ok(()),
            }.is_err()
            {
                return -1;
            }
        } else if b == b'%' {
            found_percent = true;
        } else {
            if w.write_u8(b).is_err() {
                return -1;
            }
        }
    }

    w.written as c_int
}
