// TODO: reuse more code with the thin printf impl
use crate::{
    c_str::{self, WStr},
    header::stdio::printf::{FmtKind, IntKind, Number, PrintfFmt, PrintfIter, VaArg, VaListCache},
    io::{self, Write},
};
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};
use core::{char, cmp, f64, ffi::VaList, fmt, num::FpCategory, ops::Range, slice};

use crate::{
    header::errno::EILSEQ,
    platform::{self, types::*},
};

//  ___                 _                           _        _   _
// |_ _|_ __ ___  _ __ | | ___ _ __ ___   ___ _ __ | |_ __ _| |_(_) ___  _ __  _
//  | || '_ ` _ \| '_ \| |/ _ \ '_ ` _ \ / _ \ '_ \| __/ _` | __| |/ _ \| '_ \(_)
//  | || | | | | | |_) | |  __/ | | | | |  __/ | | | || (_| | |_| | (_) | | | |_
// |___|_| |_| |_| .__/|_|\___|_| |_| |_|\___|_| |_|\__\__,_|\__|_|\___/|_| |_(_)
//               |_|

enum FmtCase {
    Lower,
    Upper,
}

// The spelled-out "infinity"/"INFINITY" is also permitted by the standard
static INF_STR_LOWER: &str = "inf";
static INF_STR_UPPER: &str = "INF";

static NAN_STR_LOWER: &str = "nan";
static NAN_STR_UPPER: &str = "NAN";

fn fmt_int<I>(fmt: u32, i: I) -> String
where
    I: fmt::Display + fmt::Octal + fmt::LowerHex + fmt::UpperHex,
{
    match char::from_u32(fmt).unwrap_or('\0') {
        'o' => format!("{:o}", i),
        'u' => i.to_string(),
        'x' => format!("{:x}", i),
        'X' => format!("{:X}", i),
        _ => panic!(
            "fmt_int should never be called with the fmt {:?}",
            char::from_u32(fmt)
        ),
    }
}

fn pad<W: Write>(
    w: &mut W,
    current_side: bool,
    pad_char: u32,
    range: Range<usize>,
) -> io::Result<()> {
    if current_side {
        for _ in range {
            if let Some(c) = char::from_u32(pad_char) {
                write!(w, "{}", c)?;
            }
        }
    }
    Ok(())
}

fn abs(float: c_double) -> c_double {
    // Don't ask me whe float.abs() seems absent...
    if float.is_sign_negative() {
        -float
    } else {
        float
    }
}

fn float_string(float: c_double, precision: usize, trim: bool) -> String {
    let mut string = format!("{:.p$}", float, p = precision);
    if trim && string.contains('.') {
        let truncate = {
            let slice = string.trim_end_matches('0');
            let mut truncate = slice.len();
            if slice.ends_with('.') {
                truncate -= 1;
            }
            truncate
        };
        string.truncate(truncate);
    }
    string
}

fn float_exp(mut float: c_double) -> (c_double, isize) {
    let mut exp: isize = 0;
    while abs(float) >= 10.0 {
        float /= 10.0;
        exp += 1;
    }
    while f64::EPSILON < abs(float) && abs(float) < 1.0 {
        float *= 10.0;
        exp -= 1;
    }
    (float, exp)
}

fn fmt_float_exp<W: Write>(
    w: &mut W,
    exp_fmt: u32,
    trim: bool,
    precision: usize,
    float: c_double,
    exp: isize,
    left: bool,
    pad_space: usize,
    pad_zero: usize,
) -> io::Result<()> {
    let mut exp2 = exp;
    let mut exp_len = 1;
    while exp2 >= 10 {
        exp2 /= 10;
        exp_len += 1;
    }

    let string = float_string(float, precision, trim);
    let len = string.len() + 2 + 2.max(exp_len);

    pad(w, !left, ' ' as u32, len..pad_space)?;
    let bytes = if string.starts_with('-') {
        w.write_all(&[b'-'])?;
        &string.as_bytes()[1..]
    } else {
        string.as_bytes()
    };
    pad(w, !left, '0' as u32, len..pad_zero)?;
    w.write_all(bytes)?;
    if let Some(c) = char::from_u32(exp_fmt) {
        write!(w, "{}{:+03}", c, exp)?;
    }
    pad(w, left, ' ' as u32, len..pad_space)?;

    Ok(())
}

fn fmt_float_normal<W: Write>(
    w: &mut W,
    trim: bool,
    precision: usize,
    float: c_double,
    left: bool,
    pad_space: usize,
    pad_zero: usize,
) -> io::Result<usize> {
    let string = float_string(float, precision, trim);

    pad(w, !left, ' ' as u32, string.len()..pad_space)?;
    let bytes = if string.starts_with('-') {
        w.write_all(&[b'-'])?;
        &string.as_bytes()[1..]
    } else {
        string.as_bytes()
    };
    pad(w, true, '0' as u32, string.len()..pad_zero)?;
    w.write_all(bytes)?;
    pad(w, left, ' ' as u32, string.len()..pad_space)?;

    Ok(string.len())
}

/// Write ±infinity or ±NaN representation for any floating-point style
fn fmt_float_nonfinite<W: Write>(w: &mut W, float: c_double, case: FmtCase) -> io::Result<()> {
    if float.is_sign_negative() {
        w.write_all(&[b'-'])?;
    }

    let nonfinite_str = match float.classify() {
        FpCategory::Infinite => match case {
            FmtCase::Lower => INF_STR_LOWER,
            FmtCase::Upper => INF_STR_UPPER,
        },
        FpCategory::Nan => match case {
            FmtCase::Lower => NAN_STR_LOWER,
            FmtCase::Upper => NAN_STR_UPPER,
        },
        _ => {
            // This function should only be called with infinite or NaN value.
            panic!("this should not be possible")
        }
    };

    w.write_all(nonfinite_str.as_bytes())?;

    Ok(())
}

unsafe fn inner_wprintf<W: Write>(w: W, format: WStr, mut ap: VaList) -> io::Result<c_int> {
    let w = &mut platform::CountingWriter::new(w);

    let iterator = PrintfIter::<c_str::Wide> { format };

    // Pre-fetch vararg types
    let mut varargs = VaListCache::default();
    let mut positional = BTreeMap::new();
    // ^ NOTE: This depends on the sorted order, do not change to HashMap or whatever

    for section in iterator {
        let arg = match section {
            Ok(PrintfFmt::Plain(text)) => continue,
            Ok(PrintfFmt::Arg(arg)) => arg,
            Err(()) => return Ok(-1),
        };
        if arg.fmtkind == FmtKind::Percent {
            continue;
        }
        for num in &[arg.min_width, arg.precision.unwrap_or(Number::Static(0))] {
            match num {
                Number::Next => varargs.args.push(VaArg::c_int(ap.arg::<c_int>())),
                Number::Index(i) => {
                    positional.insert(i - 1, (FmtKind::Signed, IntKind::Int));
                }
                Number::Static(_) => (),
            }
        }
        match arg.index {
            Some(i) => {
                positional.insert(i - 1, (arg.fmtkind, arg.intkind));
            }
            None => varargs
                .args
                .push(VaArg::arg_from(arg.fmtkind, arg.intkind, &mut ap)),
        }
    }

    // Make sure, in order, the positional arguments exist with the specified type
    for (i, arg) in positional {
        varargs.get(i, &mut ap, Some(arg));
    }

    // Main loop
    for section in iterator {
        let arg = match section {
            Ok(PrintfFmt::Plain(text)) => {
                for &wc in text.iter() {
                    if let Some(c) = char::from_u32(wc) {
                        write!(w, "{}", c)?;
                    }
                }
                continue;
            }
            Ok(PrintfFmt::Arg(arg)) => arg,
            Err(()) => return Ok(-1),
        };
        let alternate = arg.alternate;
        let zero = arg.zero;
        let mut left = arg.left;
        let sign_reserve = arg.sign_reserve;
        let sign_always = arg.sign_always;
        let min_width = arg.min_width.resolve(&mut varargs, &mut ap);
        let precision = arg.precision.map(|n| n.resolve(&mut varargs, &mut ap));
        let pad_zero = if zero { min_width } else { 0 };
        let signed_space = match pad_zero {
            0 => min_width as isize,
            _ => 0,
        };
        let pad_space = if signed_space < 0 {
            left = true;
            -signed_space as usize
        } else {
            signed_space as usize
        };
        let intkind = arg.intkind;
        let fmt_char = arg.fmt;
        let fmt = fmt_char as u32;
        let fmtkind = arg.fmtkind;
        let fmtcase = match fmt_char {
            'x' | 'f' | 'e' | 'g' => Some(FmtCase::Lower),
            'X' | 'F' | 'E' | 'G' => Some(FmtCase::Upper),
            _ => None,
        };

        let index = arg.index.map(|i| i - 1).unwrap_or_else(|| {
            if fmtkind == FmtKind::Percent {
                0
            } else {
                let i = varargs.i;
                varargs.i += 1;
                i
            }
        });

        match fmtkind {
            FmtKind::Percent => w.write_all(&[b'%'])?,
            FmtKind::Signed => {
                let string = match varargs.get(index, &mut ap, Some((arg.fmtkind, arg.intkind))) {
                    VaArg::c_char(i) => i.to_string(),
                    VaArg::c_double(i) => panic!("this should not be possible"),
                    VaArg::c_int(i) => i.to_string(),
                    VaArg::c_long(i) => i.to_string(),
                    VaArg::c_longlong(i) => i.to_string(),
                    VaArg::c_short(i) => i.to_string(),
                    VaArg::intmax_t(i) => i.to_string(),
                    VaArg::pointer(i) => (i as usize).to_string(),
                    VaArg::ptrdiff_t(i) => i.to_string(),
                    VaArg::ssize_t(i) => i.to_string(),
                    VaArg::wint_t(_) => unreachable!("this should not be possible"),
                };
                let positive = !string.starts_with('-');
                let zero = precision == Some(0) && string == "0";

                let mut len = string.len();
                let mut final_len = string.len().max(precision.unwrap_or(0));
                if positive && (sign_reserve || sign_always) {
                    final_len += 1;
                }
                if zero {
                    len = 0;
                    final_len = 0;
                }

                pad(w, !left, ' ' as u32, final_len..pad_space)?;

                let bytes = if positive {
                    if sign_reserve {
                        w.write_all(&[b' '])?;
                    } else if sign_always {
                        w.write_all(&[b'+'])?;
                    }
                    string.as_bytes()
                } else {
                    w.write_all(&[b'-'])?;
                    &string.as_bytes()[1..]
                };
                pad(w, true, '0' as u32, len..precision.unwrap_or(pad_zero))?;

                if !zero {
                    w.write_all(bytes)?;
                }

                pad(w, left, ' ' as u32, final_len..pad_space)?;
            }
            FmtKind::Unsigned => {
                let string = match varargs.get(index, &mut ap, Some((arg.fmtkind, arg.intkind))) {
                    VaArg::c_char(i) => fmt_int(fmt, i as c_uchar),
                    VaArg::c_double(i) => panic!("this should not be possible"),
                    VaArg::c_int(i) => fmt_int(fmt, i as c_uint),
                    VaArg::c_long(i) => fmt_int(fmt, i as c_ulong),
                    VaArg::c_longlong(i) => fmt_int(fmt, i as c_ulonglong),
                    VaArg::c_short(i) => fmt_int(fmt, i as c_ushort),
                    VaArg::intmax_t(i) => fmt_int(fmt, i as uintmax_t),
                    VaArg::pointer(i) => fmt_int(fmt, i as usize),
                    VaArg::ptrdiff_t(i) => fmt_int(fmt, i as size_t),
                    VaArg::ssize_t(i) => fmt_int(fmt, i as size_t),
                    VaArg::wint_t(_) => unreachable!("this should not be possible"),
                };
                let zero = precision == Some(0) && string == "0";

                // If this int is padded out to be larger than it is, don't
                // add an extra zero if octal.
                let no_precision = precision.map(|pad| pad < string.len()).unwrap_or(true);

                let len;
                let final_len = if zero {
                    len = 0;
                    0
                } else {
                    len = string.len();
                    len.max(precision.unwrap_or(0))
                        + if alternate && string != "0" {
                            match char::from_u32(fmt).unwrap_or('\0') {
                                'o' if no_precision => 1,
                                'x' | 'X' => 2,
                                _ => 0,
                            }
                        } else {
                            0
                        }
                };

                pad(w, !left, ' ' as u32, final_len..pad_space)?;

                if alternate && string != "0" {
                    match char::from_u32(fmt).unwrap_or('\0') {
                        'o' if no_precision => w.write_all(&[b'0'])?,
                        'x' => w.write_all(&[b'0', b'x'])?,
                        'X' => w.write_all(&[b'0', b'X'])?,
                        _ => (),
                    }
                }
                pad(w, true, '0' as u32, len..precision.unwrap_or(pad_zero))?;

                if !zero {
                    w.write_all(string.as_bytes())?;
                }

                pad(w, left, ' ' as u32, final_len..pad_space)?;
            }
            FmtKind::Scientific => {
                let float = match varargs.get(index, &mut ap, Some((arg.fmtkind, arg.intkind))) {
                    VaArg::c_double(i) => i,
                    _ => panic!("this should not be possible"),
                };
                if float.is_finite() {
                    let (float, exp) = float_exp(float);
                    let precision = precision.unwrap_or(6);

                    fmt_float_exp(
                        w, fmt, false, precision, float, exp, left, pad_space, pad_zero,
                    )?;
                } else {
                    fmt_float_nonfinite(w, float, fmtcase.unwrap())?;
                }
            }
            FmtKind::Decimal => {
                let float = match varargs.get(index, &mut ap, Some((arg.fmtkind, arg.intkind))) {
                    VaArg::c_double(i) => i,
                    _ => panic!("this should not be possible"),
                };
                if float.is_finite() {
                    let precision = precision.unwrap_or(6);

                    fmt_float_normal(w, false, precision, float, left, pad_space, pad_zero)?;
                } else {
                    fmt_float_nonfinite(w, float, fmtcase.unwrap())?;
                }
            }
            FmtKind::AnyNotation => {
                let float = match varargs.get(index, &mut ap, Some((arg.fmtkind, arg.intkind))) {
                    VaArg::c_double(i) => i,
                    _ => panic!("this should not be possible"),
                };
                if float.is_finite() {
                    let (log, exp) = float_exp(float);
                    let exp_fmt = ('E' as u32) | (fmt & 32);
                    let precision = precision.unwrap_or(6);
                    let use_exp_format = exp < -4 || exp >= precision as isize;

                    if use_exp_format {
                        // Length of integral part will always be 1 here,
                        // because that's how x/floor(log10(x)) works
                        let precision = precision.saturating_sub(1);
                        fmt_float_exp(
                            w, exp_fmt, true, precision, log, exp, left, pad_space, pad_zero,
                        )?;
                    } else {
                        // Length of integral part will be the exponent of
                        // the unused logarithm, unless the exponent is
                        // negative which in case the integral part must
                        // of course be 0, 1 in length
                        let len = 1 + cmp::max(0, exp) as usize;
                        let precision = precision.saturating_sub(len);
                        fmt_float_normal(w, true, precision, float, left, pad_space, pad_zero)?;
                    }
                } else {
                    fmt_float_nonfinite(w, float, fmtcase.unwrap())?;
                }
            }
            FmtKind::String => {
                let ptr = match varargs.get(index, &mut ap, Some((arg.fmtkind, arg.intkind))) {
                    VaArg::pointer(p) => p,
                    _ => panic!("this should not be possible"),
                } as *const c_char;

                if ptr.is_null() {
                    w.write_all(b"(null)")?;
                } else {
                    let max = precision.unwrap_or(::core::usize::MAX);

                    if intkind == IntKind::Long || intkind == IntKind::LongLong {
                        // Handle wchar_t
                        let mut ptr = ptr as *const wchar_t;
                        let mut string = String::new();

                        while *ptr != 0 {
                            let c = match char::from_u32(*ptr as _) {
                                Some(c) => c,
                                None => {
                                    platform::ERRNO.set(EILSEQ);
                                    return Err(io::last_os_error());
                                }
                            };
                            if string.len() + c.len_utf8() >= max {
                                break;
                            }
                            string.push(c);
                            ptr = ptr.add(1);
                        }

                        pad(w, !left, ' ' as u32, string.len()..pad_space)?;
                        w.write_all(string.as_bytes())?;
                        pad(w, left, ' ' as u32, string.len()..pad_space)?;
                    } else {
                        let mut len = 0;
                        while *ptr.add(len) != 0 && len < max {
                            len += 1;
                        }

                        pad(w, !left, ' ' as u32, len..pad_space)?;
                        w.write_all(slice::from_raw_parts(ptr as *const u8, len))?;
                        pad(w, left, ' ' as u32, len..pad_space)?;
                    }
                }
            }
            FmtKind::Char => match varargs.get(index, &mut ap, Some((arg.fmtkind, arg.intkind))) {
                VaArg::c_char(c) => {
                    pad(w, !left, ' ' as u32, 1..pad_space)?;
                    w.write_all(&[c as u8])?;
                    pad(w, left, ' ' as u32, 1..pad_space)?;
                }
                VaArg::wint_t(c) => {
                    let c = match char::from_u32(c as _) {
                        Some(c) => c,
                        None => {
                            platform::ERRNO.set(EILSEQ);
                            return Err(io::last_os_error());
                        }
                    };
                    let mut buf = [0; 4];

                    pad(w, !left, ' ' as u32, 1..pad_space)?;
                    w.write_all(c.encode_utf8(&mut buf).as_bytes())?;
                    pad(w, left, ' ' as u32, 1..pad_space)?;
                }
                _ => unreachable!("this should not be possible"),
            },
            FmtKind::Pointer => {
                let ptr = match varargs.get(index, &mut ap, Some((arg.fmtkind, arg.intkind))) {
                    VaArg::pointer(p) => p,
                    _ => panic!("this should not be possible"),
                };

                let mut len = 1;
                if ptr.is_null() {
                    len = "(nil)".len();
                } else {
                    let mut ptr = ptr as usize;
                    while ptr >= 10 {
                        ptr /= 10;
                        len += 1;
                    }
                }

                pad(w, !left, ' ' as u32, len..pad_space)?;
                if ptr.is_null() {
                    write!(w, "(nil)")?;
                } else {
                    write!(w, "0x{:x}", ptr as usize)?;
                }
                pad(w, left, ' ' as u32, len..pad_space)?;
            }
            FmtKind::GetWritten => {
                let ptr = match varargs.get(index, &mut ap, Some((arg.fmtkind, arg.intkind))) {
                    VaArg::pointer(p) => p,
                    _ => panic!("this should not be possible"),
                };

                match intkind {
                    IntKind::Byte => *(ptr as *mut c_char) = w.written as c_char,
                    IntKind::Short => *(ptr as *mut c_short) = w.written as c_short,
                    IntKind::Int => *(ptr as *mut c_int) = w.written as c_int,
                    IntKind::Long => *(ptr as *mut c_long) = w.written as c_long,
                    IntKind::LongLong => *(ptr as *mut c_longlong) = w.written as c_longlong,
                    IntKind::IntMax => *(ptr as *mut intmax_t) = w.written as intmax_t,
                    IntKind::PtrDiff => *(ptr as *mut ptrdiff_t) = w.written as ptrdiff_t,
                    IntKind::Size => *(ptr as *mut size_t) = w.written as size_t,
                }
            }
        }
    }
    Ok(w.written as c_int)
}

pub unsafe fn wprintf<W: Write>(w: W, format: WStr, ap: VaList) -> c_int {
    inner_wprintf(w, format, ap).unwrap_or(-1)
}
