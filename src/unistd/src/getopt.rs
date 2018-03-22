//! getopt implementation for Redox, following http://pubs.opengroup.org/onlinepubs/009695399/functions/getopt.html

use super::platform::types::*;
use super::platform;
use super::stdio;
use super::string;
use core::ptr;

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut optarg: *mut c_char = ptr::null_mut();

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut optind: c_int = 1;

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut opterr: c_int = 1;

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut optopt: c_int = -1;

static mut CURRENT_OPT: *mut c_char = ptr::null_mut();

#[no_mangle]
pub unsafe extern "C" fn getopt(
    argc: c_int,
    argv: *const *mut c_char,
    optstring: *const c_char,
) -> c_int {
    if CURRENT_OPT.is_null() || *CURRENT_OPT == 0 {
        if optind >= argc {
            -1
        } else {
            let current_arg = *argv.offset(optind as isize);
            if current_arg.is_null() || *current_arg != b'-' as c_char
                || string::strcmp(current_arg, b"-\0".as_ptr() as _) == 0
            {
                -1
            } else if string::strcmp(current_arg, b"--\0".as_ptr() as _) == 0 {
                optind += 1;
                -1
            } else {
                // remove the '-'
                let current_arg = current_arg.offset(1);

                parse_arg(argc, argv, current_arg, optstring)
            }
        }
    } else {
        parse_arg(argc, argv, CURRENT_OPT, optstring)
    }
}

unsafe fn parse_arg(
    argc: c_int,
    argv: *const *mut c_char,
    current_arg: *mut c_char,
    optstring: *const c_char,
) -> c_int {
    let update_current_opt = || {
        CURRENT_OPT = current_arg.offset(1);
        if *CURRENT_OPT == 0 {
            optind += 1;
        }
    };

    let print_error = |desc: &[u8]| {
        // NOTE: we don't use fprintf to get around the usage of va_list
        stdio::fputs(*argv as _, &mut *stdio::stderr);
        stdio::fputs(desc.as_ptr() as _, &mut *stdio::stderr);
        stdio::fputc(*current_arg as _, &mut *stdio::stderr);
        stdio::fputc(b'\n' as _, &mut *stdio::stderr);
    };

    match find_option(*current_arg, optstring) {
        Some(GetoptOption::Flag) => {
            update_current_opt();

            *current_arg as c_int
        }
        Some(GetoptOption::OptArg) => {
            CURRENT_OPT = b"\0".as_ptr() as _;
            if *current_arg.offset(1) == 0 {
                optind += 2;
                if optind > argc {
                    CURRENT_OPT = ptr::null_mut();

                    optopt = *current_arg as c_int;
                    let errch = if *optstring == b':' as c_char {
                        b':'
                    } else {
                        if opterr != 0 {
                            print_error(b": option requries an argument -- \0");
                        }

                        b'?'
                    };
                    errch as c_int
                } else {
                    optarg = *argv.offset(optind as isize - 1);

                    *current_arg as c_int
                }
            } else {
                optarg = current_arg.offset(1);
                optind += 1;

                *current_arg as c_int
            }
        }
        None => {
            // couldn't find the given option in optstring
            if opterr != 0 {
                print_error(b": illegal option -- \0");
            }

            update_current_opt();

            optopt = *current_arg as _;
            b'?' as c_int
        }
    }
}

enum GetoptOption {
    Flag,
    OptArg,
}

unsafe fn find_option(ch: c_char, optstring: *const c_char) -> Option<GetoptOption> {
    let mut i = 0;

    while *optstring.offset(i) != 0 {
        if *optstring.offset(i) == ch {
            let result = if *optstring.offset(i + 1) == b':' as c_char {
                GetoptOption::OptArg
            } else {
                GetoptOption::Flag
            };
            return Some(result);
        }
        i += 1;
    }

    None
}
