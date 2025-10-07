// TODO: reuse more code with the wide printf impl
use crate::{
    c_str::{self, CStr, NulStr},
    io::{self, Write},
};
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};
use core::{cmp, ffi::VaList, fmt, num::FpCategory, ops::Range, slice};

use crate::{
    header::errno::{self, EILSEQ},
    platform::{self, types::*},
};

//  ____        _ _                 _       _
// | __ )  ___ (_) | ___ _ __ _ __ | | __ _| |_ ___ _
// |  _ \ / _ \| | |/ _ \ '__| '_ \| |/ _` | __/ _ (_)
// | |_) | (_) | | |  __/ |  | |_) | | (_| | ||  __/_
// |____/ \___/|_|_|\___|_|  | .__/|_|\__,_|\__\___(_)
//                           |_|

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum IntKind {
    Byte,
    Short,
    Int,
    Long,
    LongLong,
    IntMax,
    PtrDiff,
    Size,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum FmtKind {
    Percent,

    Signed,
    Unsigned,

    Scientific,
    Decimal,
    AnyNotation,

    String,
    Char,
    Pointer,
    GetWritten,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Number {
    Static(usize),
    Index(usize),
    Next,
}
impl Number {
    pub(crate) unsafe fn resolve(self, varargs: &mut VaListCache, ap: &mut VaList) -> usize {
        let arg = match self {
            Number::Static(num) => return num,
            Number::Index(i) => varargs.get(i - 1, ap, None),
            Number::Next => {
                let i = varargs.i;
                varargs.i += 1;
                varargs.get(i, ap, None)
            }
        };
        match arg {
            VaArg::c_char(i) => i as usize,
            VaArg::c_double(i) => i as usize,
            VaArg::c_int(i) => i as usize,
            VaArg::c_long(i) => i as usize,
            VaArg::c_longlong(i) => i as usize,
            VaArg::c_short(i) => i as usize,
            VaArg::intmax_t(i) => i as usize,
            VaArg::pointer(i) => i as usize,
            VaArg::ptrdiff_t(i) => i as usize,
            VaArg::ssize_t(i) => i as usize,
            VaArg::wint_t(i) => i as usize,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum VaArg {
    c_char(c_char),
    c_double(c_double),
    c_int(c_int),
    c_long(c_long),
    c_longlong(c_longlong),
    c_short(c_short),
    intmax_t(intmax_t),
    pointer(*const c_void),
    ptrdiff_t(ptrdiff_t),
    ssize_t(ssize_t),
    wint_t(wint_t),
}
impl VaArg {
    pub(crate) unsafe fn arg_from(fmtkind: FmtKind, intkind: IntKind, ap: &mut VaList) -> VaArg {
        // Per the C standard using va_arg with a type with a size
        // less than that of an int for integers and double for floats
        // is invalid. As a result any arguments smaller than an int or
        // double passed to a function will be promoted to the smallest
        // possible size. The VaList::arg function will handle this
        // automagically.

        match (fmtkind, intkind) {
            (FmtKind::Percent, _) => panic!("Can't call arg_from on %"),

            (FmtKind::Char, IntKind::Long) | (FmtKind::Char, IntKind::LongLong) => {
                VaArg::wint_t(ap.arg::<wint_t>())
            }

            (FmtKind::Char, _)
            | (FmtKind::Unsigned, IntKind::Byte)
            | (FmtKind::Signed, IntKind::Byte) => {
                // c_int is passed but truncated to c_char
                VaArg::c_char(ap.arg::<c_int>() as c_char)
            }
            (FmtKind::Unsigned, IntKind::Short) | (FmtKind::Signed, IntKind::Short) => {
                // c_int is passed but truncated to c_short
                VaArg::c_short(ap.arg::<c_int>() as c_short)
            }
            (FmtKind::Unsigned, IntKind::Int) | (FmtKind::Signed, IntKind::Int) => {
                VaArg::c_int(ap.arg::<c_int>())
            }
            (FmtKind::Unsigned, IntKind::Long) | (FmtKind::Signed, IntKind::Long) => {
                VaArg::c_long(ap.arg::<c_long>())
            }
            (FmtKind::Unsigned, IntKind::LongLong) | (FmtKind::Signed, IntKind::LongLong) => {
                VaArg::c_longlong(ap.arg::<c_longlong>())
            }
            (FmtKind::Unsigned, IntKind::IntMax) | (FmtKind::Signed, IntKind::IntMax) => {
                VaArg::intmax_t(ap.arg::<intmax_t>())
            }
            (FmtKind::Unsigned, IntKind::PtrDiff) | (FmtKind::Signed, IntKind::PtrDiff) => {
                VaArg::ptrdiff_t(ap.arg::<ptrdiff_t>())
            }
            (FmtKind::Unsigned, IntKind::Size) | (FmtKind::Signed, IntKind::Size) => {
                VaArg::ssize_t(ap.arg::<ssize_t>())
            }

            (FmtKind::AnyNotation, _) | (FmtKind::Decimal, _) | (FmtKind::Scientific, _) => {
                VaArg::c_double(ap.arg::<c_double>())
            }

            (FmtKind::GetWritten, _) | (FmtKind::Pointer, _) | (FmtKind::String, _) => {
                VaArg::pointer(ap.arg::<*const c_void>())
            }
        }
    }
    unsafe fn transmute(&self, fmtkind: FmtKind, intkind: IntKind) -> VaArg {
        // At this point, there are conflicting printf arguments. An
        // example of this is:
        // ```c
        // printf("%1$d %1$lf\n", 5, 0.1);
        // ```
        // We handle it just like glibc: We read it from the VaList
        // using the *last* argument type, but we transmute it when we
        // try to access the other ones.
        union Untyped {
            c_char: c_char,
            c_double: c_double,
            c_int: c_int,
            c_long: c_long,
            c_longlong: c_longlong,
            c_short: c_short,
            intmax_t: intmax_t,
            pointer: *const c_void,
            ptrdiff_t: ptrdiff_t,
            ssize_t: ssize_t,
            wint_t: wint_t,
        }
        let untyped = match *self {
            VaArg::c_char(i) => Untyped { c_char: i },
            VaArg::c_double(i) => Untyped { c_double: i },
            VaArg::c_int(i) => Untyped { c_int: i },
            VaArg::c_long(i) => Untyped { c_long: i },
            VaArg::c_longlong(i) => Untyped { c_longlong: i },
            VaArg::c_short(i) => Untyped { c_short: i },
            VaArg::intmax_t(i) => Untyped { intmax_t: i },
            VaArg::pointer(i) => Untyped { pointer: i },
            VaArg::ptrdiff_t(i) => Untyped { ptrdiff_t: i },
            VaArg::ssize_t(i) => Untyped { ssize_t: i },
            VaArg::wint_t(i) => Untyped { wint_t: i },
        };
        match (fmtkind, intkind) {
            (FmtKind::Percent, _) => panic!("Can't call transmute on %"),

            (FmtKind::Char, IntKind::Long) | (FmtKind::Char, IntKind::LongLong) => {
                VaArg::wint_t(untyped.wint_t)
            }

            (FmtKind::Char, _)
            | (FmtKind::Unsigned, IntKind::Byte)
            | (FmtKind::Signed, IntKind::Byte) => VaArg::c_char(untyped.c_char),
            (FmtKind::Unsigned, IntKind::Short) | (FmtKind::Signed, IntKind::Short) => {
                VaArg::c_short(untyped.c_short)
            }
            (FmtKind::Unsigned, IntKind::Int) | (FmtKind::Signed, IntKind::Int) => {
                VaArg::c_int(untyped.c_int)
            }
            (FmtKind::Unsigned, IntKind::Long) | (FmtKind::Signed, IntKind::Long) => {
                VaArg::c_long(untyped.c_long)
            }
            (FmtKind::Unsigned, IntKind::LongLong) | (FmtKind::Signed, IntKind::LongLong) => {
                VaArg::c_longlong(untyped.c_longlong)
            }
            (FmtKind::Unsigned, IntKind::IntMax) | (FmtKind::Signed, IntKind::IntMax) => {
                VaArg::intmax_t(untyped.intmax_t)
            }
            (FmtKind::Unsigned, IntKind::PtrDiff) | (FmtKind::Signed, IntKind::PtrDiff) => {
                VaArg::ptrdiff_t(untyped.ptrdiff_t)
            }
            (FmtKind::Unsigned, IntKind::Size) | (FmtKind::Signed, IntKind::Size) => {
                VaArg::ssize_t(untyped.ssize_t)
            }

            (FmtKind::AnyNotation, _) | (FmtKind::Decimal, _) | (FmtKind::Scientific, _) => {
                VaArg::c_double(untyped.c_double)
            }

            (FmtKind::GetWritten, _) | (FmtKind::Pointer, _) | (FmtKind::String, _) => {
                VaArg::pointer(untyped.pointer)
            }
        }
    }
}
#[derive(Default)]
pub(crate) struct VaListCache {
    pub(crate) args: Vec<VaArg>,
    pub(crate) i: usize,
}
impl VaListCache {
    pub(crate) unsafe fn get(
        &mut self,
        i: usize,
        ap: &mut VaList,
        default: Option<(FmtKind, IntKind)>,
    ) -> VaArg {
        if let Some(&arg) = self.args.get(i) {
            // This value is already cached
            let mut arg = arg;
            if let Some((fmtkind, intkind)) = default {
                // ...but as a different type
                arg = arg.transmute(fmtkind, intkind);
            }
            return arg;
        }

        // Get all values before this value
        while self.args.len() < i {
            // We can't POSSIBLY know the type if we reach this
            // point. Reaching here means there are unused gaps in the
            // arguments. Ultimately we'll have to settle down with
            // defaulting to c_int.
            self.args.push(VaArg::c_int(ap.arg::<c_int>()))
        }

        // Add the value to the cache
        self.args.push(match default {
            Some((fmtkind, intkind)) => VaArg::arg_from(fmtkind, intkind, ap),
            None => VaArg::c_int(ap.arg::<c_int>()),
        });

        // Return the value
        self.args[i]
    }
}

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

fn pop_int_raw<T: c_str::Kind>(format: &mut NulStr<T>) -> Option<usize> {
    let mut int = None;
    while let Some((digit, rest)) = format
        .split_first_char()
        .and_then(|(d, r)| Some((d.to_digit(10)?, r)))
    {
        *format = rest;
        if int.is_none() {
            int = Some(0);
        }
        *int.as_mut().unwrap() *= 10;
        *int.as_mut().unwrap() += digit as usize;
    }
    int
}
fn pop_index<T: c_str::Kind>(format: &mut NulStr<T>) -> Option<usize> {
    // Peek ahead for a positional argument:
    let mut format2 = *format;
    if let Some(i) = pop_int_raw(&mut format2) {
        if let Some(('$', format2)) = format2.split_first_char() {
            *format = format2;
            return Some(i);
        }
    }
    None
}
fn pop_int<T: c_str::Kind>(format: &mut NulStr<T>) -> Option<Number> {
    if let Some(('*', rest)) = format.split_first_char() {
        *format = rest;
        Some(pop_index(format).map(Number::Index).unwrap_or(Number::Next))
    } else {
        pop_int_raw(format).map(Number::Static)
    }
}

fn fmt_int<I, T: c_str::Kind>(fmt: char, i: I) -> String
where
    I: fmt::Display + fmt::Octal + fmt::LowerHex + fmt::UpperHex + fmt::Binary,
{
    match fmt {
        'o' => format!("{:o}", i),
        'u' => i.to_string(),
        'x' => format!("{:x}", i),
        'X' => format!("{:X}", i),
        'b' | 'B' if T::IS_THIN_NOT_WIDE => format!("{:b}", i),
        _ => panic!(
            "fmt_int should never be called with the fmt {:?}",
            fmt as char,
        ),
    }
}

fn pad<W: Write>(
    w: &mut W,
    current_side: bool,
    pad_char: u8,
    range: Range<usize>,
) -> io::Result<()> {
    if current_side {
        for _ in range {
            w.write_all(&[pad_char])?;
        }
    }
    Ok(())
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
    while float.abs() >= 10.0 {
        float /= 10.0;
        exp += 1;
    }
    while f64::EPSILON < float.abs() && float.abs() < 1.0 {
        float *= 10.0;
        exp -= 1;
    }
    (float, exp)
}

fn fmt_float_exp<W: Write>(
    w: &mut W,
    exp_fmt: char,
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

    pad(w, !left, b' ', len..pad_space)?;
    let bytes = if string.starts_with('-') {
        w.write_all(&[b'-'])?;
        &string.as_bytes()[1..]
    } else {
        string.as_bytes()
    };
    pad(w, !left, b'0', len..pad_zero)?;
    w.write_all(bytes)?;
    write!(w, "{}{:+03}", exp_fmt, exp)?;
    pad(w, left, b' ', len..pad_space)?;

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

    pad(w, !left, b' ', string.len()..pad_space)?;
    let bytes = if string.starts_with('-') {
        w.write_all(&[b'-'])?;
        &string.as_bytes()[1..]
    } else {
        string.as_bytes()
    };
    pad(w, true, b'0', string.len()..pad_zero)?;
    w.write_all(bytes)?;
    pad(w, left, b' ', string.len()..pad_space)?;

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

#[derive(Clone, Copy)]
pub(crate) struct PrintfIter<'a, T: c_str::Kind> {
    pub(crate) format: NulStr<'a, T>,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct PrintfArg {
    pub(crate) index: Option<usize>,
    pub(crate) alternate: bool,
    pub(crate) zero: bool,
    pub(crate) left: bool,
    pub(crate) sign_reserve: bool,
    pub(crate) sign_always: bool,
    pub(crate) min_width: Number,
    pub(crate) precision: Option<Number>,
    pub(crate) intkind: IntKind,
    pub(crate) fmt: char,
    pub(crate) fmtkind: FmtKind,
}
#[derive(Debug)]
pub(crate) enum PrintfFmt<'a, U> {
    Plain(&'a [U]),
    Arg(PrintfArg),
}
impl<'a, T: c_str::Kind> Iterator for PrintfIter<'a, T> {
    type Item = Result<PrintfFmt<'a, T::Char>, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        // Send PrintfFmt::Plain until the next %
        let first_percent = match self.format.find_get_subslice_or_all(b'%') {
            Err(([], _)) => return None,
            Ok((chunk @ [_, ..], rest)) | Err((chunk @ [_, ..], rest)) => {
                self.format = rest;
                return Some(Ok(PrintfFmt::Plain(chunk)));
            }
            Ok(([], rest)) => rest,
        };

        // at this point the next char must be %
        self.format = first_percent.split_first().expect("must be %").1;

        let mut peekahead = self.format;
        let index = pop_index(&mut peekahead).map(|i| {
            self.format = peekahead;
            i
        });

        // Flags:
        let mut alternate = false;
        let mut zero = false;
        let mut left = false;
        let mut sign_reserve = false;
        let mut sign_always = false;

        while let Some((c, rest)) = self.format.split_first_char() {
            match c {
                '#' => alternate = true,
                '0' => zero = true,
                '-' => left = true,
                ' ' => sign_reserve = true,
                '+' => sign_always = true,
                _ => break,
            }
            self.format = rest;
        }

        // Width and precision:
        let min_width = pop_int(&mut self.format).unwrap_or(Number::Static(0));
        let precision = if let Some(('.', rest)) = self.format.split_first_char() {
            self.format = rest;
            match pop_int(&mut self.format) {
                int @ Some(_) => int,
                None => return Some(Err(())),
            }
        } else {
            None
        };

        // Integer size:
        let mut intkind = IntKind::Int;
        while let Some((byte, rest)) = self.format.split_first_char() {
            intkind = match byte {
                'h' => {
                    if intkind == IntKind::Short || intkind == IntKind::Byte {
                        IntKind::Byte
                    } else {
                        IntKind::Short
                    }
                }
                'j' => IntKind::IntMax,
                'l' => {
                    if intkind == IntKind::Long || intkind == IntKind::LongLong {
                        IntKind::LongLong
                    } else {
                        IntKind::Long
                    }
                }
                'q' | 'L' => IntKind::LongLong,
                't' => IntKind::PtrDiff,
                'z' => IntKind::Size,
                _ => break,
            };

            self.format = rest;
        }
        let Some((fmt, rest)) = self.format.split_first_char() else {
            return Some(Err(()));
        };
        self.format = rest;
        let fmtkind = match fmt {
            '%' => FmtKind::Percent,
            'd' | 'i' => FmtKind::Signed,
            'o' | 'u' | 'x' | 'X' => FmtKind::Unsigned,
            'b' | 'B' if T::IS_THIN_NOT_WIDE => FmtKind::Unsigned,
            'e' | 'E' => FmtKind::Scientific,
            'f' | 'F' => FmtKind::Decimal,
            'g' | 'G' => FmtKind::AnyNotation,
            's' => FmtKind::String,
            'c' => FmtKind::Char,
            'p' => FmtKind::Pointer,
            'n' => FmtKind::GetWritten,
            'm' if T::IS_THIN_NOT_WIDE => {
                // %m is technically for syslog only, but musl and glibc implement it for
                // printf because it is difficult and error prone to implement a format
                // specifier for just *one* function.
                return Some(Ok(PrintfFmt::Plain(
                    T::chars_from_bytes(
                        errno::STR_ERROR
                            .get(platform::ERRNO.get() as usize)
                            .map(|e| e.as_bytes())
                            .unwrap_or(b"unknown error"),
                    )
                    .expect("string must be thin"),
                )));
            }
            _ => return Some(Err(())),
        };

        Some(Ok(PrintfFmt::Arg(PrintfArg {
            index,
            alternate,
            zero,
            left,
            sign_reserve,
            sign_always,
            min_width,
            precision,
            intkind,
            fmt,
            fmtkind,
        })))
    }
}

pub(crate) unsafe fn inner_printf<T: c_str::Kind>(
    w: impl Write,
    format: NulStr<T>,
    mut ap: VaList,
) -> io::Result<c_int> {
    let w = &mut platform::CountingWriter::new(w);

    let iterator = PrintfIter { format };

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
                if T::IS_THIN_NOT_WIDE {
                    let bytes = T::chars_to_bytes(text).expect("is thin");
                    w.write_all(bytes)?;
                } else {
                    // TODO: wcsrtombs wrapper
                    for c in text.iter().filter_map(|u| char::from_u32((*u).into())) {
                        write!(w, "{}", c);
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
        let fmt = arg.fmt;
        let fmtkind = arg.fmtkind;
        let fmtcase = match fmt {
            'b' if T::IS_THIN_NOT_WIDE => Some(FmtCase::Lower),
            'B' if T::IS_THIN_NOT_WIDE => Some(FmtCase::Upper),
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

                pad(w, !left, b' ', final_len..pad_space)?;

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
                pad(w, true, b'0', len..precision.unwrap_or(pad_zero))?;

                if !zero {
                    w.write_all(bytes)?;
                }

                pad(w, left, b' ', final_len..pad_space)?;
            }
            FmtKind::Unsigned => {
                let string = match varargs.get(index, &mut ap, Some((arg.fmtkind, arg.intkind))) {
                    VaArg::c_char(i) => fmt_int::<_, T>(fmt, i as c_uchar),
                    VaArg::c_double(i) => panic!("this should not be possible"),
                    VaArg::c_int(i) => fmt_int::<_, T>(fmt, i as c_uint),
                    VaArg::c_long(i) => fmt_int::<_, T>(fmt, i as c_ulong),
                    VaArg::c_longlong(i) => fmt_int::<_, T>(fmt, i as c_ulonglong),
                    VaArg::c_short(i) => fmt_int::<_, T>(fmt, i as c_ushort),
                    VaArg::intmax_t(i) => fmt_int::<_, T>(fmt, i as uintmax_t),
                    VaArg::pointer(i) => fmt_int::<_, T>(fmt, i as usize),
                    VaArg::ptrdiff_t(i) => fmt_int::<_, T>(fmt, i as size_t),
                    VaArg::ssize_t(i) => fmt_int::<_, T>(fmt, i as size_t),
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
                            match fmt {
                                'o' if no_precision => 1,
                                'x' | 'X' => 2,
                                'b' | 'B' if T::IS_THIN_NOT_WIDE => 2,
                                _ => 0,
                            }
                        } else {
                            0
                        }
                };

                pad(w, !left, b' ', final_len..pad_space)?;

                if alternate && string != "0" {
                    match fmt {
                        'o' if no_precision => w.write_all(b"0")?,
                        'x' => w.write_all(b"0x")?,
                        'X' => w.write_all(b"0X")?,
                        'b' if T::IS_THIN_NOT_WIDE => w.write_all(b"0b")?,
                        'B' if T::IS_THIN_NOT_WIDE => w.write_all(b"0B")?,
                        _ => (),
                    }
                }
                pad(w, true, b'0', len..precision.unwrap_or(pad_zero))?;

                if !zero {
                    w.write_all(string.as_bytes())?;
                }

                pad(w, left, b' ', final_len..pad_space)?;
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
                    // TODO: .is_uppercase()?
                    let exp_fmt = if fmt as u32 & 32 == 32 { 'e' } else { 'E' };
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

                        pad(w, !left, b' ', string.len()..pad_space)?;
                        w.write_all(string.as_bytes())?;
                        pad(w, left, b' ', string.len()..pad_space)?;
                    } else {
                        let mut len = 0;
                        while *ptr.add(len) != 0 && len < max {
                            len += 1;
                        }

                        pad(w, !left, b' ', len..pad_space)?;
                        w.write_all(slice::from_raw_parts(ptr as *const u8, len))?;
                        pad(w, left, b' ', len..pad_space)?;
                    }
                }
            }
            FmtKind::Char => match varargs.get(index, &mut ap, Some((arg.fmtkind, arg.intkind))) {
                VaArg::c_char(c) => {
                    pad(w, !left, b' ', 1..pad_space)?;
                    w.write_all(&[c as u8])?;
                    pad(w, left, b' ', 1..pad_space)?;
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

                    pad(w, !left, b' ', 1..pad_space)?;
                    w.write_all(c.encode_utf8(&mut buf).as_bytes())?;
                    pad(w, left, b' ', 1..pad_space)?;
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

                pad(w, !left, b' ', len..pad_space)?;
                if ptr.is_null() {
                    write!(w, "(nil)")?;
                } else {
                    write!(w, "0x{:x}", ptr as usize)?;
                }
                pad(w, left, b' ', len..pad_space)?;
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

/// Implementation of `printf` formatting function, generic over a `writer`
///
/// This implementation in currently compliant over C17 specification (lacking a few one from C23)
/// and contains extensions as well.
///
/// # The Format Specification
/// ```text
/// %[conversion-flags][field-width][precision][length-modifier]<conversion-format>
/// ```
///
/// <div class="warning">
/// ※ : This symbol means it is not implemented yet, but it is defined in the C standard
/// </div>
///
/// ## Conversion Flags
/// Conversion flags are flags that modify the behavior of the [conversion
/// format]. Each one can happen only once per format specifier. They are:
///
/// - `-`: The result of the conversion is left-justified within the field (by default it is
///   right-justified).
/// - `+`: The sign of signed conversions is always prepended to the result of the conversion (by
///   default the result is preceded by minus **only** when it is negative).
/// - ` `(space): If the result of a signed conversion does not start with a sign character, or is
///   empty, space is prepended to the result.
///   - It is ignored if `+` flag is present.
/// - `#`: Alternative form of the conversion is performed. See the documentation for each
///   [conversion format] for details.
/// - `0`: For integer and floating-point number conversions, leading zeros are used to pad the
///   field instead of space characters.
///   - For integer numbers it is ignored if the precision is explicitly specified.
///   - For other conversions using this flag results in undefined behavior.
///   - It is ignored if `-` flag is present.
///
/// ## Field Width
/// Specifies minimum field width. This makes the result to be padded (with spaces by default, with
/// zeroes if `0` conversion flag is specified) if the converted value has fewer characters than the
/// specified width. It can take three forms:
///
/// - `N` where N is a positive integer: Specifies the field width value of `N`.
/// - `*`: The width is specified by an extra argument of type [`int`], which has to appear before
///   the argument to be converted and the [precision] (if specified with `.*`).
///   - If the value of e extra argument is negative, it is interpreted as with `-` [conversion
///     flag], i.e. left-justified result.
/// - `*P$` where P is a positive integer: The width is specified by an extra argument of type
///   [`int`], which has to appear exactly at the position specified by `P`.
///   - This is a popular extension of the C and POSIX standards.
///   - If the value of e extra argument is negative, it is interpreted as with `-` [conversion
///     flag], i.e. left-justified result.
///
/// ## Precision
/// Specifies the precision of the conversion.
///
/// For integer [conversion formats], this specifies the number of digits to appear in the result.
///
/// For float point [conversion formats], this specifies the number of digits to appear after the
/// decimal-point character.
///
/// It can take three forms:
///
/// - `.N` where N is a positive integer: Specifies the precision value of `N`.
/// - `.*`: The precision is specified by an extra argument of type [`int`], which  has to appear
///   before the argument to be converted and after the the [field width] (if specified with `*`).
///   - If the value of e extra argument is negative, it is interpreted as if the precision were
///     omitted.
/// - `.*P$` where P is a positive integer: The precision is specified by an extra argument of type
///   [`int`], which has to appear exactly at the position specified by `P`.
///   - This is an popular extension of the C and POSIX standards.
///   - If the value of e extra argument is negative, it is interpreted as if the precision were
///     omitted.
///
/// ## Length Modifier
/// Specifies the size of the argument. In combination with the [conversion format], it specifies
/// the type of the corresponding argument.
///
/// - `hh`: Byte size
///   - Works with integer conversion formats (`d`, `i`, `o`, `x`, `X`, `b`, `B`)
///   - Works with written number conversion format (`n`)
/// - `h`: Short size
///   - Works with integer conversion formats (`d`, `i`, `o`, `x`, `X`, `b`, `B`)
///   - Works with written number conversion format (`n`)
/// - `l`: Long size
///   - Works with integer conversion formats (`d`, `i`, `o`, `x`, `X`, `b`, `B`)
///   - Works with character conversion format (`c`)
///   - Works with string conversion format (`s`)
///   - Works with written number conversion format (`n`)
///   - Works with float conversion formats (`f`, `F`, `e`, `E`, `a`, `A`, `g`, `G`) (C99)
/// - `ll`: Long long size
///   - Works with integer conversion formats (`d`, `i`, `o`, `x`, `X`, `b`, `B`)
///   - Works with written number conversion format (`n`)
/// - `j`: Maximum width
///   - Works with integer conversion formats (`d`, `i`, `o`, `x`, `X`, `b`, `B`)
///   - Works with written number conversion format (`n`)
/// - `z`: Pointer width size
///   - Works with integer conversion formats (`d`, `i`, `o`, `x`, `X`, `b`, `B`)
///   - Works with written number conversion format (`n`)
/// - `t`: Pointer diff width
///   - Works with integer conversion formats (`d`, `i`, `o`, `x`, `X`, `b`, `B`)
///   - Works with written number conversion format (`n`)
/// - `wN` (C23 ※): Specifies that the size should be N bits width version of the supported
///   conversion format.
///   - Works with integer conversion formats (`d`, `i`, `o`, `x`, `X`, `b`, `B`)
///   - The supported values of `N` must be the same as the widths specified in `stdint.h`
/// - `wfN` (C23 ※): Specifies that the size should be the fast N bits width version of the
///   supported conversion format.
///   - Works with integer conversion formats (`d`, `i`, `o`, `x`, `X`, `b`, `B`)
///   - The supported values of `N` must be the same as the widths specified in `stdint.h`
/// - `L`: Long double size
///   - Works with float conversion formats (`f`, `F`, `e`, `E`, `a`, `A`, `g`, `G`)
/// - `H` (C23 ※): _Decimal32 size
///   - Works with float conversion formats (`f`, `F`, `e`, `E`, `a`, `A`, `g`, `G`)
/// - `D` (C23 ※): _Decimal64 size
///   - Works with float conversion formats (`f`, `F`, `e`, `E`, `a`, `A`, `g`, `G`)
/// - `DD` (C23 ※): _Decimal128 size
///   - Works with float conversion formats (`f`, `F`, `e`, `E`, `a`, `A`, `g`, `G`)
///
/// ## Conversion Format
/// Specifies the conversion format as one of the following:
///
/// - `%`: Writes a percent symbol. The full conversion format must be `%%`.
/// - `c`: Writes as single character
///   - Without length modifier:
///     - The argument is first converted to [`unsigned char`]
///   - With `l` length modifier:
///     - The argument is first converted to a character string as if by `%ls` with a array of 2
///       [`wchar_t`] argument.
/// - `s`: Writes a character string
///   - The argument is a pointer to the first character
///   - The [precision] specifies the maximum number of bytes to be written. If not specified,
///     writes up to the first null character found.
/// - `d` and `i`: Writes a decimal representation of a signed integer
///   - The [precision] specifies the minimal number to appear (defaults to `1`).
///   - If the precision is zero and the value to be written is also zero, the result is no
///     characters written.
/// - `u`: Writes the decimal representation of a unsigned integer
///   - The [precision] specifies the minimal number to appear (defaults to `1`).
///   - If the precision is zero and the value to be written is also zero, the result is no
///     characters written.
/// - `o`: Writes the octal representation of a unsigned integer.
///   - The [precision] specifies the minimal number to appear (defaults to `1`).
///   - If the precision is zero and the value to be written is also zero, the result is no
///     characters written.
///   - The alternative representation includes a leading `0`.
///   - The types are the same as `u`
/// - `x`: Writes the hexadecimal representation of a unsigned integer with lowercase characters.
///   - The [precision] specifies the minimal number to appear (defaults to `1`).
///   - If the precision is zero and the value to be written is also zero, the result is no
///     characters written.
///   - The alternative representation includes a leading `0x`.
///   - The types are the same as `u`
/// - `X`: Writes the hexadecimal representation of a unsigned integer with uppercase characters.
///   - The [precision] specifies the minimal number to appear (defaults to `1`).
///   - If the precision is zero and the value to be written is also zero, the result is no
///     characters written.
///   - The alternative representation includes a leading `0X`.
///   - The types are the same as `u`.
/// - `b` | `B` (C23): Writes the binary representation of a unsigned integer.
///   - The [precision] specifies the minimal number to appear (defaults to `1`).
///   - If the precision is zero and the value to be written is also zero, the result is no
///     characters written.
///   - The alternative representation includes a leading `0b` and `0B`, respectively.
///   - The types are the same as `u`.
/// - `f` | `F`: Writes the decimal representation of a float point number.
///   - The [precision] specifies the exact number of digits to appear after the decimal point
///     character (defaults to `6`).
///   - The alternative representation, the decimal point character is written even if no digits
///     follow it.
/// - `e` | `E`: Writes the float point number with the decimal exponential notation (\[-\]d.ddd
///   **e**±dd | \[-\]d.ddd **E**±dd)
///   - The [precision] specifies the exact number of digits to appear after the decimal point
///     character (defaults to `6`).
///   - The exponent contains at least two digits, more digits are used only if necessary.
///   - If the value is ​zero, the exponent is also ​zero​.
///   - The alternative representation: decimal point character is written even if no digits follow
///     it.
/// - `a` | `A`: Writes the float point number with the hexadecimal exponential notation (\[-\]
///   **0x**h.hhh **p**±d | \[-\] **0X**h.hhh **P**±d)
///   - The [precision] specifies the exact number of digits to appear after the hexadecimal point
///     character (defaults to `6`).
///   - If the value is ​zero, the exponent is also ​zero​.
///   - The alternative representation: decimal point character is written even if no digits follow
///     it.
/// - `g` | `G`: Writes the float point number to decimal or decimal exponent notation depending on
///   the value and the [precision].
///   - Let `P` equal the precision if nonzero, `6` if the precision is not specified, or `1` if the
///     precision is `​0`​. Then, if a conversion with style `E` would have an exponent of `X`:
///     - If `P > X ≥ −4`, the conversion is with the format `f` and precision `P − 1 − X`.
///     - Otherwise, the conversion is with the format `e` or `E` and precision `P − 1`.
///   - Unless alternative representation is requested, the trailing zeros are removed. Also the
///     decimal point character is removed if no fractional part is left.
/// - `n`: Writes the number of characters written in the call into the argument pointer
///   - It can not contain any [conversion flag], [field width], or [precision].
/// - `p`: Writes an implementation defined character sequence defining a pointer.
///
/// ### Types
/// The types expected by the format string can change with the [length modifier].
///
/// For the `c`:
/// - Without length modifier: [`int`]
/// - With `l` length modifier: [`wint_t`]
///
/// For the `s`:
/// - Without length modifier: pointer to [`char`] (`char*`, `const char*`)
/// - With `l` length modifier: pointer to [`wchar_t`] (`wchar_t*`, `const wchar_t*`)
///
/// For the `d` and `i`:
/// - Without length modifier: [`int`]
/// - With `hh` length modifier: [`signed char`]
/// - With `h` length modifier: [`short`]
/// - With `l` length modifier: [`long`]
/// - With `ll` length modifier: [`long long`]
/// - With `j` length modifier: [`intmax_t`]
/// - With `z` length modifier: [`ssize_t`]
/// - With `t` length modifier: [`ptrdiff_t`]
///
/// For the `u`, `o`, `x`, `X`, `b`, `B`:
/// - Without length modifier: [`unsigned int`]
/// - With `hh` length modifier: [`unsigned char`]
/// - With `h` length modifier: [`unsigned short`]
/// - With `l` length modifier: [`unsigned long`]
/// - With `ll` length modifier: [`unsigned long long`]
/// - With `j` length modifier: [`uintmax_t`]
/// - With `z` length modifier: [`size_t`]
/// - With `t` length modifier: [`unsigned ptrdiff_t`]
///
/// For the `f`, `F`, `e`, `E`, `a`, `A`, `g`, `G`:
/// - Without length modifier: [`double`]
/// - With `l` length modifier: [`double`]
/// - With `L` length modifier: `long double`
/// - With `H` length modifier (C23 ※): `_Decimal32`
/// - With `D` length modifier (C23 ※): `_Decimal64`
/// - With `DD` length modifier (C23 ※): `_Decimal128`
///
/// For the `n`
/// - Without length modifier: pointer to [`int`] (`int*`)
/// - With `hh` length modifier: pointer to [`signed char`] (`signed char*`)
/// - With `h` length modifier: pointer to [`short`] (`short*`)
/// - With `l` length modifier: pointer to [`long`] (`long*`)
/// - With `ll` length modifier: pointer to [`long long`] (`long long*`)
/// - With `j` length modifier: pointer to [`intmax_t`] (`intmax_t*`)
/// - With `z` length modifier: pointer to [`ssize_t`] (`ssize_t*`)
/// - With `t` length modifier: pointer to [`ptrdiff_t`] (`ptrdiff_t*`)
///
/// For the `p`, it must always be a pointer to [`void`] (`void*` | `const void*`)
///
/// [precision]: #precision
/// [field width]: #field-width
/// [length modifier]: #length-modifier
/// [conversion format]: #conversion-format
/// [`int`]: c_int
/// [`unsigned char`]: c_uchar
/// [`unsigned short`]: c_ushort
/// [`unsigned int`]: c_uint
/// [`unsigned long`]: c_ulong
/// [`unsigned long long`]: c_ulonglong
/// [`unsigned ptrdiff_t`]: ptrdiff_t
/// [`wchar_t`]: wchar_t
/// [`char`]: c_char
/// [`signed char`]: c_schar
/// [`short`]: c_short
/// [`long`]: c_long
/// [`long long`]: c_longlong
/// [`double`]: c_double
/// [`long double`]: c_longdouble
/// [`void`]: c_void
///
/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// - `ap` must follow the safety contract of variable arguments of C.
pub unsafe fn printf(w: impl Write, format: CStr, ap: VaList) -> c_int {
    inner_printf::<c_str::Thin>(w, format, ap).unwrap_or(-1)
}
