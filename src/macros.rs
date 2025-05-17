/// Print to stdout
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = write!($crate::platform::FileWriter::new(1), $($arg)*);
    });
}

/// Print with new line to stdout
#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

/// Print to stderr
#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = write!($crate::platform::FileWriter::new(2), $($arg)*);
    });
}

/// Print with new line to stderr
#[macro_export]
macro_rules! eprintln {
    () => (eprint!("\n"));
    ($fmt:expr) => (eprint!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (eprint!(concat!($fmt, "\n"), $($arg)*));
}

/// Lifted from libstd
#[macro_export]
macro_rules! dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        eprintln!("[{}:{}:{}]", file!(), line!(), column!());
    };
    ($val:expr) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                eprintln!(
                    "[{}:{}:{}] {} = {:#?}",
                    file!(),
                    line!(),
                    column!(),
                    stringify!($val),
                    &tmp
                );
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($(dbg!($val)),+,)
    };
}

#[macro_export]
#[cfg(not(feature = "trace"))]
macro_rules! trace {
    ($($arg:tt)*) => {};
}

#[macro_export]
#[cfg(feature = "trace")]
macro_rules! trace {
    ($($arg:tt)*) => ({
        use $crate::{Pal, Sys};
        eprintln!($($arg)*);
    });
}

#[macro_export]
#[cfg(not(feature = "trace"))]
macro_rules! trace_expr {
    ($expr:expr, $($arg:tt)*) => {
        $expr
    };
}

#[macro_export]
#[cfg(feature = "trace")]
macro_rules! trace_expr {
    ($expr:expr, $($arg:tt)*) => ({
        use $crate::header::errno::STR_ERROR;
        use $crate::platform;

        trace!("{}", format_args!($($arg)*));

        let trace_old_errno = platform::ERRNO.get();
        platform::ERRNO.set(0);

        let ret = $expr;

        let trace_errno = platform::ERRNO.get() as isize;
        if trace_errno == 0 {
            platform::ERRNO.set(trace_old_errno);
        }

        let trace_strerror = if trace_errno >= 0 && trace_errno < STR_ERROR.len() as isize {
            STR_ERROR[trace_errno as usize]
        } else {
            "Unknown error"
        };

        trace!("{} = {} ({}, {})", format_args!($($arg)*), ret, trace_errno, trace_strerror);

        ret
    });
}

#[macro_export]
macro_rules! strto_impl {
    (
        $rettype:ty, $signed:expr, $maxval:expr, $minval:expr, $s:ident, $endptr:ident, $base:ident
    ) => {{
        // ensure these are constants
        const CHECK_SIGN: bool = $signed;
        const MAX_VAL: $rettype = $maxval;
        const MIN_VAL: $rettype = $minval;

        let set_endptr = |idx: isize| {
            if !$endptr.is_null() {
                // This is stupid, but apparently strto* functions want
                // const input but mut output, yet the man page says
                // "stores the address of the first invalid character in *endptr"
                // so obviously it doesn't want us to clone it.
                *$endptr = $s.offset(idx) as *mut _;
            }
        };

        let invalid_input = || {
            platform::ERRNO.set(EINVAL);
            set_endptr(0);
        };

        // only valid bases are 2 through 36
        if $base != 0 && ($base < 2 || $base > 36) {
            invalid_input();
            return 0;
        }

        let mut idx = 0;

        // skip any whitespace at the beginning of the string
        while ctype::isspace(*$s.offset(idx) as c_int) != 0 {
            idx += 1;
        }

        // check for +/-
        let positive = match is_positive(*$s.offset(idx)) {
            Some((pos, i)) => {
                idx += i;
                pos
            }
            None => {
                invalid_input();
                return 0;
            }
        };

        // convert the string to a number
        let num_str = $s.offset(idx);
        let res = match $base {
            0 => detect_base(num_str).and_then(|($base, i)| {
                idx += i;
                convert_integer(num_str.offset(i), $base)
            }),
            8 => convert_octal(num_str),
            16 => convert_hex(num_str),
            _ => convert_integer(num_str, $base),
        };

        // check for error parsing octal/hex prefix
        // also check to ensure a number was indeed parsed
        let (num, i, overflow) = match res {
            Some(res) => res,
            None => {
                invalid_input();
                return 0;
            }
        };
        idx += i;

        let overflow = if CHECK_SIGN {
            overflow || (num as c_long).is_negative()
        } else {
            overflow
        };
        // account for the sign
        let num = num as $rettype;
        let num = if overflow {
            platform::ERRNO.set(ERANGE);
            if CHECK_SIGN {
                if positive {
                    MAX_VAL
                } else {
                    MIN_VAL
                }
            } else {
                MAX_VAL
            }
        } else {
            if positive {
                num
            } else {
                // not using -num to keep the compiler happy
                num.overflowing_neg().0
            }
        };

        set_endptr(idx);

        num
    }};
}

#[macro_export]
macro_rules! strto_float_impl {
    ($type:ident, $s:expr, $endptr:expr) => {{
        let mut s = $s;
        let endptr = $endptr;

        while ctype::isspace(*s as c_int) != 0 {
            s = s.offset(1);
        }

        let mut result: $type = 0.0;
        let mut exponent: Option<$type> = None;
        let mut radix = 10;

        let result_sign = match *s as u8 {
            b'-' => {
                s = s.offset(1);
                -1.0
            }
            b'+' => {
                s = s.offset(1);
                1.0
            }
            _ => 1.0,
        };

        let rust_s = CStr::from_ptr(s).to_string_lossy();

        // detect NaN, Inf
        if rust_s.to_lowercase().starts_with("inf") {
            result = $type::INFINITY;
            s = s.offset(3);
        } else if rust_s.to_lowercase().starts_with("nan") {
            // we cannot signal negative NaN in LLVM backed languages
            // https://github.com/rust-lang/rust/issues/73328 , https://github.com/rust-lang/rust/issues/81261
            result = $type::NAN;
            s = s.offset(3);
        } else {
            if *s as u8 == b'0' && *s.offset(1) as u8 == b'x' {
                s = s.offset(2);
                radix = 16;
            }

            while let Some(digit) = (*s as u8 as char).to_digit(radix) {
                result *= radix as $type;
                result += digit as $type;
                s = s.offset(1);
            }

            if *s as u8 == b'.' {
                s = s.offset(1);

                let mut i = 1.0;
                while let Some(digit) = (*s as u8 as char).to_digit(radix) {
                    i *= radix as $type;
                    result += digit as $type / i;
                    s = s.offset(1);
                }
            }

            let s_before_exponent = s;

            exponent = match (*s as u8, radix) {
                (b'e' | b'E', 10) | (b'p' | b'P', 16) => {
                    s = s.offset(1);

                    let is_exponent_positive = match *s as u8 {
                        b'-' => {
                            s = s.offset(1);
                            false
                        }
                        b'+' => {
                            s = s.offset(1);
                            true
                        }
                        _ => true,
                    };

                    // Exponent digits are always in base 10.
                    if (*s as u8 as char).is_digit(10) {
                        let mut exponent_value = 0;

                        while let Some(digit) = (*s as u8 as char).to_digit(10) {
                            exponent_value *= 10;
                            exponent_value += digit;
                            s = s.offset(1);
                        }

                        let exponent_base = match radix {
                            10 => 10u128,
                            16 => 2u128,
                            _ => unreachable!(),
                        };

                        if is_exponent_positive {
                            Some(exponent_base.pow(exponent_value) as $type)
                        } else {
                            Some(1.0 / (exponent_base.pow(exponent_value) as $type))
                        }
                    } else {
                        // Exponent had no valid digits after 'e'/'p' and '+'/'-', rollback
                        s = s_before_exponent;
                        None
                    }
                }
                _ => None,
            };
        }

        if !endptr.is_null() {
            // This is stupid, but apparently strto* functions want
            // const input but mut output, yet the man page says
            // "stores the address of the first invalid character in *endptr"
            // so obviously it doesn't want us to clone it.
            *endptr = s as *mut _;
        }

        if let Some(exponent) = exponent {
            result_sign * result * exponent
        } else {
            result_sign * result
        }
    }};
}
