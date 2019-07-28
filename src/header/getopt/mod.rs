//! getopt implementation for relibc

use crate::{
    header::{
        stdio, string,
        unistd::{optarg, opterr, optind, optopt},
    },
    platform::types::*,
};
use core::ptr;

static mut CURRENT_OPT: *mut c_char = ptr::null_mut();

pub const no_argument: c_int = 0;
pub const required_argument: c_int = 1;
pub const optional_argument: c_int = 2;

#[repr(C)]
pub struct option {
    name: *const c_char,
    has_arg: c_int,
    flag: *mut c_int,
    val: c_int,
}

#[no_mangle]
#[linkage = "weak"] // often redefined in GNU programs
pub unsafe extern "C" fn getopt_long(
    argc: c_int,
    argv: *const *mut c_char,
    optstring: *const c_char,
    longopts: *const option,
    longindex: *mut c_int,
) -> c_int {
    // if optarg is not set, we still don't want the previous value leaking
    optarg = ptr::null_mut();

    // handle reinitialization request
    if optind == 0 {
        optind = 1;
        CURRENT_OPT = ptr::null_mut();
    }

    if CURRENT_OPT.is_null() || *CURRENT_OPT == 0 {
        if optind >= argc {
            -1
        } else {
            let current_arg = *argv.offset(optind as isize);
            if current_arg.is_null()
                || *current_arg != b'-' as c_char
                || *current_arg.offset(1) == 0
            {
                -1
            } else if string::strcmp(current_arg, c_str!("--").as_ptr()) == 0 {
                optind += 1;
                -1
            } else {
                // remove the '-'
                let current_arg = current_arg.offset(1);

                if *current_arg == b'-' as c_char && !longopts.is_null() {
                    let current_arg = current_arg.offset(1);
                    // is a long option
                    for i in 0.. {
                        let opt = &*longopts.offset(i);
                        if opt.name.is_null() {
                            break;
                        }

                        let mut end = 0;
                        while {
                            let c = *current_arg.offset(end);
                            c != 0 && c != b'=' as c_char
                        } {
                            end += 1;
                        }

                        if string::strncmp(current_arg, opt.name, end as size_t) == 0 {
                            optind += 1;
                            *longindex = i as c_int;

                            if opt.has_arg == optional_argument {
                                if *current_arg.offset(end) == b'=' as c_char {
                                    optarg = current_arg.offset(end + 1);
                                }
                            } else if opt.has_arg == required_argument {
                                if *current_arg.offset(end) == b'=' as c_char {
                                    optarg = current_arg.offset(end + 1);
                                } else if optind < argc {
                                    optarg = *argv.offset(optind as isize);
                                    optind += 1;
                                } else if *optstring == b':' as c_char {
                                    return b':' as c_int;
                                } else {
                                    stdio::fputs(*argv as _, &mut *stdio::stderr);
                                    stdio::fputs(
                                        ": option '--\0".as_ptr() as _,
                                        &mut *stdio::stderr,
                                    );
                                    stdio::fputs(current_arg, &mut *stdio::stderr);
                                    stdio::fputs(
                                        "' requires an argument\n\0".as_ptr() as _,
                                        &mut *stdio::stderr,
                                    );
                                    return b'?' as c_int;
                                }
                            }

                            if opt.flag.is_null() {
                                return opt.val;
                            } else {
                                *opt.flag = opt.val;
                                return 0;
                            }
                        }
                    }
                }

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

            optopt = *current_arg as c_int;
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
