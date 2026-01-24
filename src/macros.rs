/// Print to stdout
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = $crate::platform::FileWriter::new(1).write_fmt(format_args!($($arg)*));
    }};
}

/// Print with new line to stdout.
/// Deprecated, consider using log::info instead
#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*))
    };
}

/// Print to stderr
#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = $crate::platform::FileWriter::new(2).write_fmt(format_args!($($arg)*));
    }};
}

/// Print with new line to stderr.
/// Deprecated, consider using log::info instead
#[macro_export]
macro_rules! eprintln {
    () => {
        $crate::eprint!("\n")
    };
    ($($arg:tt)*) => {
        $crate::eprint!("{}\n", format_args!($($arg)*))
    };
}

pub const ISSUE_URL: &str = "https://gitlab.redox-os.org/redox-os/relibc/-/issues/";

// Skippable todo!(issue, fmt)
#[macro_export]
macro_rules! todo_skip {
    ($issue:expr, $($arg:tt)*) => {
        if $issue != 0 {
            log::info!("TODO ({}{}): {}", crate::macros::ISSUE_URL, $issue, format_args!($($arg)*))
        } else {
            log::info!("TODO: {}", format_args!($($arg)*))
        }
    };
}

// Recoverable error todo!(issue, fmt, err)
#[macro_export]
macro_rules! todo_error {
    ($issue:expr, $err:expr, $($arg:tt)*) => {
        if $issue != 0 {
            log::error!("TODO ({}{}): {}: {}", crate::macros::ISSUE_URL, $issue, format_args!($($arg)*), $err)
        } else {
            log::error!("TODO: {}: {:?}", format_args!($($arg)*), $err)
        }
    };
}

// Unrecoverable error todo!(issue, fmt)
#[macro_export]
macro_rules! todo_panic {
    ($issue:expr, $($arg:tt)*) => {
        if $issue != 0 {
            todo!("{} ({}{})", format_args!($($arg)*), crate::macros::ISSUE_URL, $issue)
        } else {
            todo!("{}", format_args!($($arg)*))
        }
    };
}

#[macro_export]
#[cfg(feature = "no_trace")]
macro_rules! trace_expr {
    ($expr:expr, $($arg:tt)*) => {
        $expr
    };
}

#[macro_export]
#[cfg(not(feature = "no_trace"))]
macro_rules! trace_expr {
    ($expr:expr, $($arg:tt)*) => ({
        use $crate::header::errno::STR_ERROR;
        use $crate::platform;

        log::trace!("{}", format_args!($($arg)*));

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

        log::trace!("{} = {} ({}, {})", format_args!($($arg)*), ret, trace_errno, trace_strerror);

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
                unsafe {
                    *$endptr = $s.offset(idx) as *mut _;
                }
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
        while ctype::isspace(unsafe { *$s.offset(idx) } as c_int) != 0 {
            idx += 1;
        }

        // check for +/-
        let positive = match is_positive(unsafe { *$s.offset(idx) }) {
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
        let num_str = unsafe { $s.offset(idx) };
        let res = match $base {
            0 => unsafe { detect_base(num_str) }.and_then(|($base, i)| {
                idx += i;
                unsafe { convert_integer(num_str.offset(i), $base) }
            }),
            8 => unsafe { convert_octal(num_str) },
            16 => unsafe { convert_hex(num_str) },
            _ => unsafe { convert_integer(num_str, $base) },
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
                if positive { MAX_VAL } else { MIN_VAL }
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

        while ctype::isspace(unsafe{*s} as c_int) != 0 {
            s = unsafe{ s.offset(1)};
        }

        let mut result: $type = 0.0;
        let mut exponent: Option<$type> = None;
        let mut radix = 10;

        let result_sign = match unsafe{*s} as u8 {
            b'-' => {
                s = unsafe{s.offset(1)};
                -1.0
            }
            b'+' => {
                s = unsafe{s.offset(1)};
                1.0
            }
            _ => 1.0,
        };

        let rust_s = unsafe{CStr::from_ptr(s)}.to_string_lossy();

        // detect NaN, Inf
        if rust_s.to_lowercase().starts_with("inf") {
            result = $type::INFINITY;
            s = unsafe{s.offset(3)};
        } else if rust_s.to_lowercase().starts_with("nan") {
            // we cannot signal negative NaN in LLVM backed languages
            // https://github.com/rust-lang/rust/issues/73328 , https://github.com/rust-lang/rust/issues/81261
            result = $type::NAN;
            s = unsafe{s.offset(3)};
        } else {
            if unsafe{*s} as u8 == b'0' && unsafe{*s.offset(1)} as u8 == b'x' {
                s = unsafe{s.offset(2)};
                radix = 16;
            }

            while let Some(digit) = (unsafe{*s} as u8 as char).to_digit(radix) {
                result *= radix as $type;
                result += digit as $type;
                s = unsafe{s.offset(1)};
            }

            if unsafe{*s} as u8 == b'.' {
                s = unsafe{s.offset(1)};

                let mut i = 1.0;
                while let Some(digit) = (unsafe{*s} as u8 as char).to_digit(radix) {
                    i *= radix as $type;
                    result += digit as $type / i;
                    s = unsafe{s.offset(1)};
                }
            }

            let s_before_exponent = s;

            exponent = match (unsafe{*s} as u8, radix) {
                (b'e' | b'E', 10) | (b'p' | b'P', 16) => {
                    s = unsafe{s.offset(1)};

                    let is_exponent_positive = match unsafe{*s} as u8 {
                        b'-' => {
                            s = unsafe{s.offset(1)};
                            false
                        }
                        b'+' => {
                            s = unsafe{s.offset(1)};
                            true
                        }
                        _ => true,
                    };

                    // Exponent digits are always in base 10.
                    if (unsafe{*s} as u8 as char).is_digit(10) {
                        let mut exponent_value = 0;

                        while let Some(digit) = (unsafe{*s} as u8 as char).to_digit(10) {
                            exponent_value *= 10;
                            exponent_value += digit;
                            s = unsafe{s.offset(1)};
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
            unsafe{*endptr = s as *mut _};
        }

        if let Some(exponent) = exponent {
            result_sign * result * exponent
        } else {
            result_sign * result
        }
    }};
}

/// Project an `Out<struct X { field: Type }>` to `struct X { field: Out<Type> }`.
///
/// It is allowed to include only a subset of the struct's fields. The struct must implement
/// `OutProject`.
#[macro_export]
macro_rules! out_project {
    {
        let $struct:ty { $($field:ident : $fieldty:ty),*$(,)? } = $src:ident;
    } => {
        // Verify $src actually has type Out<$struct>. Also verify it implements `OutProject`. This
        // excludes
        //
        // - the case where $src is Out<&Struct>, where it would be very UB to just construct a
        // writable reference to $src.$field, or a smart pointer
        // - the case where there are unaligned fields where it would be UB to call ptr::write to
        // them (requiring packed structs)
        {
            fn ensure_type<U: $crate::out::OutProject>(_t: &$crate::out::Out<U>) {}
            ensure_type::<$struct>(&$src);
        }
        // Verify there are no duplicate struct fields. This is not strictly necessary as Out lacks
        // the noalias requirement, but forbidding the same field to occur multiple times would
        // allow both cases. The compiler will reject any struct that reuses the same identifier.
        const _: () = {
            $(
                if ::core::mem::offset_of!($struct, $field) % ::core::mem::align_of::<$fieldty>() != 0 {
                    panic!(concat!("unaligned field ", stringify!($field), " of struct ", stringify!($struct), "."));
                }
            )*
            struct S {
                $(
                    $field: $fieldty
                ),*
            }
        };

        // Finally, create an Out<$fieldty> for each field.
        $(
            // getting the pointer to $field is safe
            let $field = unsafe { &raw mut (*$crate::out::Out::<_>::as_mut_ptr(&mut $src)).$field };
        )*
        $(
            let mut $field: $crate::out::Out<$fieldty> = unsafe {
                // SAFETY: the only guarantee is that the pointer is valid and writable for the
                // duration of 'b where $src: Out<'b, T>. But if so, and T is a struct, that
                // must also be true for all the struct fields.
                $crate::out::Out::with_lifetime_of(
                    $crate::out::Out::nonnull($field),
                    &$src,
                )
            };
        )*
    }
}
#[macro_export]
macro_rules! OutProject {
    derive() { $(#[$($attrs:meta),*])* $v:vis struct $name:ident {
        $(
            $(#[$($fa:meta),*])* $fv:vis $field:ident : $type:ty
        ),*$(,)?
    } } => {
        // SAFETY: As simple as it is, OutProject is valid for any struct, and the pattern we have
        // matched above ensures $name is one.
        unsafe impl $crate::out::OutProject for $name {}
    }
}
#[macro_export]
#[cfg(not(feature = "check_against_libc_crate"))]
macro_rules! CheckVsLibcCrate {
    derive() { $(#[$($attrs:meta),*])* $v:vis struct $name:ident {
        $(
            $(#[$($fa:meta),*])* $fv:vis $field:ident : $type:ty
        ),*$(,)?
    } } => {
    }
}

// TODO: probably exists nice nightly features that allow conflicting impls. Then we wouldn't need
// much of this redundant code just to say A == B -> B == A and say A == B -> *mut A == *mut B.
pub trait LibcTypeEquals<A, B> {}
//impl<A, B> LibcTypeEquals<A, B> for () {}
impl<A, B> LibcTypeEquals<*mut A, *mut B> for () where (): LibcTypeEquals<A, B> {}
impl<A, B> LibcTypeEquals<*const A, *const B> for () where (): LibcTypeEquals<A, B> {}
impl<A, B, const N: usize> LibcTypeEquals<[A; N], [B; N]> for () where (): LibcTypeEquals<A, B> {}
macro_rules! for_primitive_int(
    ($i:ident) => {
        impl LibcTypeEquals<$i, $i> for () {}
    }
);
for_primitive_int!(u8);
for_primitive_int!(u16);
for_primitive_int!(u32);
for_primitive_int!(u64);
for_primitive_int!(u128);
for_primitive_int!(usize);
for_primitive_int!(i8);
for_primitive_int!(i16);
for_primitive_int!(i32);
for_primitive_int!(i64);
for_primitive_int!(i128);
for_primitive_int!(isize);
impl LibcTypeEquals<crate::platform::types::c_void, crate::platform::types::c_void> for () {}
impl LibcTypeEquals<__libc_only_for_layout_checks::c_void, crate::platform::types::c_void> for () {}
impl LibcTypeEquals<crate::platform::types::c_void, __libc_only_for_layout_checks::c_void> for () {}

//impl LibcTypeEquals<__libc_only_for_layout_checks::c_void>

/// Derive macro which checks that structs here are defined the same as in the libc crate. Perhaps
/// not sufficiently rigorous to soundly cast between the types, but should catch most mistakes.
#[macro_export]
#[cfg(feature = "check_against_libc_crate")]
macro_rules! CheckVsLibcCrate {
    // XXX: not sure we can have the name be different from libc::$name without parameters to the
    // derive macro
    derive() { $(#[$($attrs:meta),*])* $v:vis struct $name:ident {
        $(
            $(#[$($fa:meta),*])* $fv:vis $field:ident : $type:ty
        ),*$(,)?
    } } => {
        // TODO: check repr(C)? probably possible to match on $attrs
        #[allow(dead_code)]
        const _: () = {
            if ::core::mem::size_of::<$name>() != ::core::mem::size_of::<::__libc_only_for_layout_checks::$name>() {
                panic!("struct size mismatch");
            }
            if ::core::mem::align_of::<$name>() != ::core::mem::align_of::<::__libc_only_for_layout_checks::$name>() {
                panic!("struct alignment mismatch");
            }
            $(
                if ::core::mem::offset_of!($name, $field) != ::core::mem::offset_of!(__libc_only_for_layout_checks::$name, $field) {
                    panic!("struct field offset mismatch");
                }
            )*
        };
        $(
            // check all field types are equivalent
            #[allow(dead_code)]
            const _: () = {
                fn ensure_ty<A, B>(a: A, b: B) where (): $crate::macros::LibcTypeEquals::<A, B> {}
                fn for_libc(a: $name, b: __libc_only_for_layout_checks::$name) {
                    let a: $type = panic!("never called");
                    ensure_ty(a, b.$field);
                }
            };
        )*
        impl $crate::macros::LibcTypeEquals<$name, __libc_only_for_layout_checks::$name> for () {}
        impl $crate::macros::LibcTypeEquals<__libc_only_for_layout_checks::$name, $name> for () {}
    }
}
