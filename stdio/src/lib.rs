//! stdio implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/stdio.h.html

#![no_std]

extern crate platform;
extern crate va_list as vl;

use platform::types::*;
use vl::VaList as va_list;

pub const BUFSIZ: c_int = 4096;

pub const FILENAME_MAX: c_int = 4096;

pub struct FILE;

pub static mut stdout: *mut FILE = 1 as *mut FILE;
pub static mut stderr: *mut FILE = 2 as *mut FILE;

#[no_mangle]
pub unsafe extern "C" fn vfprintf(file: *mut FILE, format: *const c_char, mut ap: va_list) -> c_int {
    use core::fmt::Write;
    use core::slice;
    use core::str;

    extern "C" {
        fn strlen(s: *const c_char) -> size_t;
    }

    let mut w = platform::FileWriter(file as c_int);

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

#[no_mangle]
pub unsafe extern "C" fn vprintf(format: *const c_char, ap: va_list) -> c_int {
    vfprintf(stdout, format, ap)
}

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
