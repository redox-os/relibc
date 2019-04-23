use alloc::string::String;
use alloc::string::ToString;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::ffi::VaList;
use core::ops::Range;
use core::{fmt, slice};
use io::{self, Write};

use platform;
use platform::types::*;

//  ____        _ _                 _       _
// | __ )  ___ (_) | ___ _ __ _ __ | | __ _| |_ ___ _
// |  _ \ / _ \| | |/ _ \ '__| '_ \| |/ _` | __/ _ (_)
// | |_) | (_) | | |  __/ |  | |_) | | (_| | ||  __/_
// |____/ \___/|_|_|\___|_|  | .__/|_|\__,_|\__\___(_)
//                           |_|


#[derive(Clone, Copy, PartialEq, Eq)]
enum IntKind {
    Byte,
    Short,
    Int,
    Long,
    LongLong,
    IntMax,
    PtrDiff,
    Size,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum FmtKind {
    Percent,

    Signed,
    Unsigned,

    Scientific,
    Decimal,
    AnyNotation,

    String,
    Char,
    Pointer,
    GetWritten
}
#[derive(Clone, Copy, Debug)]
enum Number {
    Static(usize),
    Index(usize),
    Next
}
impl Number {
    unsafe fn resolve(&self, varargs: &mut VaListCache, ap: &mut VaList) -> usize {
        let arg = match *self {
            Number::Static(num) => return num,
            Number::Index(i) => varargs.get(i-1, ap, None),
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
            VaArg::ssize_t(i) => i as usize
        }
    }
}
#[derive(Clone, Copy)]
enum VaArg {
    c_char(c_char),
    c_double(c_double),
    c_int(c_int),
    c_long(c_long),
    c_longlong(c_longlong),
    c_short(c_short),
    intmax_t(intmax_t),
    pointer(*const c_void),
    ptrdiff_t(ptrdiff_t),
    ssize_t(ssize_t)
}
impl VaArg {
    unsafe fn arg_from(arg: &PrintfArg, ap: &mut VaList) -> VaArg {
        // Per the C standard using va_arg with a type with a size
        // less than that of an int for integers and double for floats
        // is invalid. As a result any arguments smaller than an int or
        // double passed to a function will be promoted to the smallest
        // possible size. The VaList::arg function will handle this
        // automagically.

        match (arg.fmtkind, arg.intkind) {
            (FmtKind::Percent, _) => panic!("Can't call arg_from on %"),

            (FmtKind::Char, _) |
            (FmtKind::Unsigned, IntKind::Byte) |
            (FmtKind::Signed, IntKind::Byte) => VaArg::c_char(ap.arg::<c_char>()),
            (FmtKind::Unsigned, IntKind::Short) |
            (FmtKind::Signed, IntKind::Short) => VaArg::c_short(ap.arg::<c_short>()),
            (FmtKind::Unsigned, IntKind::Int) |
            (FmtKind::Signed, IntKind::Int) => VaArg::c_int(ap.arg::<c_int>()),
            (FmtKind::Unsigned, IntKind::Long) |
            (FmtKind::Signed, IntKind::Long) => VaArg::c_long(ap.arg::<c_long>()),
            (FmtKind::Unsigned, IntKind::LongLong) |
            (FmtKind::Signed, IntKind::LongLong) => VaArg::c_longlong(ap.arg::<c_longlong>()),
            (FmtKind::Unsigned, IntKind::IntMax) |
            (FmtKind::Signed, IntKind::IntMax) => VaArg::intmax_t(ap.arg::<intmax_t>()),
            (FmtKind::Unsigned, IntKind::PtrDiff) |
            (FmtKind::Signed, IntKind::PtrDiff) => VaArg::ptrdiff_t(ap.arg::<ptrdiff_t>()),
            (FmtKind::Unsigned, IntKind::Size) |
            (FmtKind::Signed, IntKind::Size) => VaArg::ssize_t(ap.arg::<ssize_t>()),

            (FmtKind::AnyNotation, _) | (FmtKind::Decimal, _) | (FmtKind::Scientific, _)
                => VaArg::c_double(ap.arg::<c_double>()),

            (FmtKind::GetWritten, _) | (FmtKind::Pointer, _) | (FmtKind::String, _)
                => VaArg::pointer(ap.arg::<*const c_void>()),
        }
    }
    unsafe fn transmute(&self, arg: &PrintfArg) -> VaArg {
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
            ssize_t: ssize_t
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
            VaArg::ssize_t(i) => Untyped { ssize_t: i }
        };
        match (arg.fmtkind, arg.intkind) {
            (FmtKind::Percent, _) => panic!("Can't call transmute on %"),

            (FmtKind::Char, _) |
            (FmtKind::Unsigned, IntKind::Byte) |
            (FmtKind::Signed, IntKind::Byte) => VaArg::c_char(untyped.c_char),
            (FmtKind::Unsigned, IntKind::Short) |
            (FmtKind::Signed, IntKind::Short) => VaArg::c_short(untyped.c_short),
            (FmtKind::Unsigned, IntKind::Int) |
            (FmtKind::Signed, IntKind::Int) => VaArg::c_int(untyped.c_int),
            (FmtKind::Unsigned, IntKind::Long) |
            (FmtKind::Signed, IntKind::Long) => VaArg::c_long(untyped.c_long),
            (FmtKind::Unsigned, IntKind::LongLong) |
            (FmtKind::Signed, IntKind::LongLong) => VaArg::c_longlong(untyped.c_longlong),
            (FmtKind::Unsigned, IntKind::IntMax) |
            (FmtKind::Signed, IntKind::IntMax) => VaArg::intmax_t(untyped.intmax_t),
            (FmtKind::Unsigned, IntKind::PtrDiff) |
            (FmtKind::Signed, IntKind::PtrDiff) => VaArg::ptrdiff_t(untyped.ptrdiff_t),
            (FmtKind::Unsigned, IntKind::Size) |
            (FmtKind::Signed, IntKind::Size) => VaArg::ssize_t(untyped.ssize_t),

            (FmtKind::AnyNotation, _) | (FmtKind::Decimal, _) | (FmtKind::Scientific, _)
                => VaArg::c_double(untyped.c_double),

            (FmtKind::GetWritten, _) | (FmtKind::Pointer, _) | (FmtKind::String, _)
                => VaArg::pointer(untyped.pointer),
        }
    }
}
#[derive(Default)]
struct VaListCache {
    args: Vec<VaArg>,
    i: usize
}
impl VaListCache {
    unsafe fn get(&mut self, i: usize, ap: &mut VaList, default: Option<&PrintfArg>) -> VaArg {
        if let Some(&arg) = self.args.get(i) {
            let mut arg = arg;
            if let Some(default) = default {
                arg = arg.transmute(default);
            }
            return arg;
        }
        while self.args.len() < i {
            // We can't POSSIBLY know the type if we reach this
            // point. Reaching here means there are unused gaps in the
            // arguments. Ultimately we'll have to settle down with
            // defaulting to c_int.
            self.args.push(VaArg::c_int(ap.arg::<c_int>()))
        }
        self.args.push(match default {
            Some(default) => VaArg::arg_from(default, ap),
            None => VaArg::c_int(ap.arg::<c_int>())
        });
        self.args[i]
    }
}

//  ___                 _                           _        _   _
// |_ _|_ __ ___  _ __ | | ___ _ __ ___   ___ _ __ | |_ __ _| |_(_) ___  _ __  _
//  | || '_ ` _ \| '_ \| |/ _ \ '_ ` _ \ / _ \ '_ \| __/ _` | __| |/ _ \| '_ \(_)
//  | || | | | | | |_) | |  __/ | | | | |  __/ | | | || (_| | |_| | (_) | | | |_
// |___|_| |_| |_| .__/|_|\___|_| |_| |_|\___|_| |_|\__\__,_|\__|_|\___/|_| |_(_)
//               |_|


unsafe fn pop_int_raw(format: &mut *const u8) -> Option<usize> {
    let mut int = None;
    while let Some(digit) = (**format as char).to_digit(10) {
        *format = format.add(1);
        if int.is_none() {
            int = Some(0);
        }
        *int.as_mut().unwrap() *= 10;
        *int.as_mut().unwrap() += digit as usize;
    }
    int
}
unsafe fn pop_index(format: &mut *const u8) -> Option<usize> {
    // Peek ahead for a positional argument:
    let mut format2 = *format;
    if let Some(i) = pop_int_raw(&mut format2) {
        if *format2 == b'$' {
            *format = format2.add(1);
            return Some(i);
        }
    }
    None
}
unsafe fn pop_int(format: &mut *const u8) -> Option<Number> {
    if **format == b'*' {
        *format = format.add(1);
        Some(
            pop_index(format)
                .map(Number::Index)
                .unwrap_or(Number::Next)
        )
    } else {
        pop_int_raw(format).map(Number::Static)
    }
}

unsafe fn fmt_int<I>(fmt: u8, i: I) -> String
where
    I: fmt::Display + fmt::Octal + fmt::LowerHex + fmt::UpperHex,
{
    match fmt {
        b'o' => format!("{:o}", i),
        b'u' => i.to_string(),
        b'x' => format!("{:x}", i),
        b'X' => format!("{:X}", i),
        _ => panic!("fmt_int should never be called with the fmt {:?}", fmt as char),
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
    if trim {
        let truncate = {
            let mut slice = string.trim_end_matches('0');
            if slice.ends_with('.') {
                slice.len() - 1
            } else {
                slice.len()
            }
        };
        string.truncate(truncate);
    }
    string
}

fn fmt_float_exp<W: Write>(
    w: &mut W,
    exp_fmt: u8,
    range: Option<(isize, isize)>,
    trim: bool,
    precision: usize,
    mut float: c_double,
    left: bool,
    pad_space: usize,
    pad_zero: usize,
) -> io::Result<bool> {
    let mut exp: isize = 0;
    while float >= 10.0 || float <= -10.0 {
        float /= 10.0;
        exp += 1;
    }
    while (float > 0.0 && float < 1.0) || (float > -1.0 && float < 0.0) {
        float *= 10.0;
        exp -= 1;
    }

    if range
        .map(|(start, end)| exp >= start && exp < end)
        .unwrap_or(false)
    {
        return Ok(false);
    }

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
    write!(w, "{}{:+03}", exp_fmt as char, exp)?;
    pad(w, left, b' ', len..pad_space)?;

    Ok(true)
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

#[derive(Clone, Copy)]
struct PrintfIter {
    format: *const u8
}
#[derive(Clone, Copy)]
struct PrintfArg {
    index: Option<usize>,
    alternate: bool,
    zero: bool,
    left: bool,
    sign_reserve: bool,
    sign_always: bool,
    min_width: Number,
    precision: Option<Number>,
    pad_space: Number,
    pad_zero: Number,
    intkind: IntKind,
    fmt: u8,
    fmtkind: FmtKind
}
enum PrintfFmt {
    Plain(&'static [u8]),
    Arg(PrintfArg)
}
impl Iterator for PrintfIter {
    type Item = Result<PrintfFmt, ()>;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            // Send PrintfFmt::Plain until the next %
            let mut len = 0;
            while *self.format.add(len) != 0 && *self.format.add(len) != b'%' {
                len += 1;
            }
            if len > 0 {
                let slice = slice::from_raw_parts(self.format as *const u8, len);
                self.format = self.format.add(len);
                return Some(Ok(PrintfFmt::Plain(slice)));
            }
            self.format = self.format.add(len);
            if *self.format == 0 {
                return None;
            }

            // *self.format is guaranteed to be '%' at this point
            self.format = self.format.add(1);

            let mut peekahead = self.format;
            let index = pop_index(&mut peekahead)
                .map(|i| {
                    self.format = peekahead;
                    i
                });

            // Flags:
            let mut alternate = false;
            let mut zero = false;
            let mut left = false;
            let mut sign_reserve = false;
            let mut sign_always = false;

            loop {
                match *self.format {
                    b'#' => alternate = true,
                    b'0' => zero = true,
                    b'-' => left = true,
                    b' ' => sign_reserve = true,
                    b'+' => sign_always = true,
                    _ => break,
                }
                self.format = self.format.add(1);
            }

            // Width and precision:
            let min_width = pop_int(&mut self.format).unwrap_or(Number::Static(0));
            let precision = if *self.format == b'.' {
                self.format = self.format.add(1);
                match pop_int(&mut self.format) {
                    int @ Some(_) => int,
                    None => return Some(Err(())),
                }
            } else {
                None
            };

            let pad_space = if zero { Number::Static(0) } else { min_width };
            let pad_zero = if zero { min_width } else { Number::Static(0) };

            // Integer size:
            let mut intkind = IntKind::Int;
            loop {
                intkind = match *self.format {
                    b'h' => {
                        if intkind == IntKind::Short || intkind == IntKind::Byte {
                            IntKind::Byte
                        } else {
                            IntKind::Short
                        }
                    }
                    b'j' => IntKind::IntMax,
                    b'l' => {
                        if intkind == IntKind::Long || intkind == IntKind::LongLong {
                            IntKind::LongLong
                        } else {
                            IntKind::Long
                        }
                    }
                    b'q' | b'L' => IntKind::LongLong,
                    b't' => IntKind::PtrDiff,
                    b'z' => IntKind::Size,
                    _ => break,
                };

                self.format = self.format.add(1);
            }
            let fmt = *self.format;
            let fmtkind = match fmt {
                b'%' => FmtKind::Percent,
                b'd' | b'i' => FmtKind::Signed,
                b'o' | b'u' | b'x' | b'X' => FmtKind::Unsigned,
                b'e' | b'E' => FmtKind::Scientific,
                b'f' | b'F' => FmtKind::Decimal,
                b'g' | b'G' => FmtKind::AnyNotation,
                b's' => FmtKind::String,
                b'c' => FmtKind::Char,
                b'p' => FmtKind::Pointer,
                b'n' => FmtKind::GetWritten,
                _ => return Some(Err(())),
            };
            self.format = self.format.add(1);

            Some(Ok(PrintfFmt::Arg(PrintfArg {
                index,
                alternate,
                zero,
                left,
                sign_reserve,
                sign_always,
                min_width,
                precision,
                pad_space,
                pad_zero,
                intkind,
                fmt,
                fmtkind
            })))
        }
    }
}


unsafe fn inner_printf<W: Write>(w: W, format: *const c_char, mut ap: VaList) -> io::Result<c_int> {
    let w = &mut platform::CountingWriter::new(w);

    let iterator = PrintfIter {
        format: format as *const u8
    };

    // Pre-fetch vararg types
    let mut varargs = VaListCache::default();
    let mut positional = BTreeMap::new();
    // ^ NOTE: This depends on the sorted order, do not change to HashMap or whatever

    for section in iterator {
        let arg = match section {
            Ok(PrintfFmt::Plain(text)) => continue,
            Ok(PrintfFmt::Arg(arg)) => arg,
            Err(()) => return Ok(-1)
        };
        if arg.fmtkind == FmtKind::Percent {
            continue;
        }
        if let Some(i) = arg.index {
            positional.insert(i-1, arg);
        } else {
            varargs.args.push(VaArg::arg_from(&arg, &mut ap));
        }
    }
    // Make sure, in order, the positional arguments exist with the specified type
    for (i, arg) in positional {
        varargs.get(i, &mut ap, Some(&arg));
    }

    // Main loop
    for section in iterator {
        let arg = match section {
            Ok(PrintfFmt::Plain(text)) => {
                w.write_all(text)?;
                continue;
            },
            Ok(PrintfFmt::Arg(arg)) => arg,
            Err(()) => return Ok(-1)
        };
        let alternate = arg.alternate;
        let zero = arg.zero;
        let left = arg.left;
        let sign_reserve = arg.sign_reserve;
        let sign_always = arg.sign_always;
        let min_width = arg.min_width.resolve(&mut varargs, &mut ap);
        let precision = arg.precision.map(|n| n.resolve(&mut varargs, &mut ap));
        let pad_space = arg.pad_space.resolve(&mut varargs, &mut ap);
        let pad_zero = arg.pad_zero.resolve(&mut varargs, &mut ap);
        let intkind = arg.intkind;
        let fmt = arg.fmt;
        let fmtkind = arg.fmtkind;

        let index = arg.index
            .map(|i| i-1)
            .unwrap_or_else(|| if fmtkind == FmtKind::Percent {
                0
            } else {
                let i = varargs.i;
                varargs.i += 1;
                i
            });

        match fmtkind {
            FmtKind::Percent => w.write_all(&[b'%'])?,
            FmtKind::Signed => {
                let string = match varargs.get(index, &mut ap, Some(&arg)) {
                    VaArg::c_char(i) => i.to_string(),
                    VaArg::c_double(i) => panic!("this should not be possible"),
                    VaArg::c_int(i) => i.to_string(),
                    VaArg::c_long(i) => i.to_string(),
                    VaArg::c_longlong(i) => i.to_string(),
                    VaArg::c_short(i) => i.to_string(),
                    VaArg::intmax_t(i) => i.to_string(),
                    VaArg::pointer(i) => (i as usize).to_string(),
                    VaArg::ptrdiff_t(i) => i.to_string(),
                    VaArg::ssize_t(i) => i.to_string()
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
            },
            FmtKind::Unsigned => {
                let string = match varargs.get(index, &mut ap, Some(&arg)) {
                    VaArg::c_char(i) => fmt_int(fmt, i as c_uchar),
                    VaArg::c_double(i) => panic!("this should not be possible"),
                    VaArg::c_int(i) => fmt_int(fmt, i as c_uint),
                    VaArg::c_long(i) => fmt_int(fmt, i as c_ulong),
                    VaArg::c_longlong(i) => fmt_int(fmt, i as c_ulonglong),
                    VaArg::c_short(i) => fmt_int(fmt, i as c_ushort),
                    VaArg::intmax_t(i) => fmt_int(fmt, i as uintmax_t),
                    VaArg::pointer(i) => fmt_int(fmt, i as usize),
                    VaArg::ptrdiff_t(i) => fmt_int(fmt, i as size_t),
                    VaArg::ssize_t(i) => fmt_int(fmt, i as size_t)
                };
                let zero = precision == Some(0) && string == "0";

                // If this int is padded out to be larger than it is, don't
                // add an extra zero if octal.
                let no_precision = precision.map(|pad| pad < string.len()).unwrap_or(true);

                let mut len = string.len();
                let mut final_len = string.len().max(precision.unwrap_or(0))
                    + if alternate && string != "0" {
                        match fmt {
                            b'o' if no_precision => 1,
                            b'x' | b'X' => 2,
                            _ => 0,
                        }
                    } else {
                        0
                    };

                if zero {
                    len = 0;
                    final_len = 0;
                }

                pad(w, !left, b' ', final_len..pad_space)?;

                if alternate && string != "0" {
                    match fmt {
                        b'o' if no_precision => w.write_all(&[b'0'])?,
                        b'x' => w.write_all(&[b'0', b'x'])?,
                        b'X' => w.write_all(&[b'0', b'X'])?,
                        _ => (),
                    }
                }
                pad(w, true, b'0', len..precision.unwrap_or(pad_zero))?;

                if !zero {
                    w.write_all(string.as_bytes())?;
                }

                pad(w, left, b' ', final_len..pad_space)?;
            },
            FmtKind::Scientific => {
                let mut float = match varargs.get(index, &mut ap, Some(&arg)) {
                    VaArg::c_double(i) => i,
                    _ => panic!("this should not be possible")
                };
                let precision = precision.unwrap_or(6);

                fmt_float_exp(w, fmt, None, false, precision, float, left, pad_space, pad_zero)?;
            },
            FmtKind::Decimal => {
                let mut float = match varargs.get(index, &mut ap, Some(&arg)) {
                    VaArg::c_double(i) => i,
                    _ => panic!("this should not be possible")
                };
                let precision = precision.unwrap_or(6);

                fmt_float_normal(w, false, precision, float, left, pad_space, pad_zero)?;
            },
            FmtKind::AnyNotation => {
                let mut float = match varargs.get(index, &mut ap, Some(&arg)) {
                    VaArg::c_double(i) => i,
                    _ => panic!("this should not be possible")
                };
                let exp_fmt = b'E' | (fmt & 32);
                let precision = precision.unwrap_or(6);

                if !fmt_float_exp(
                    w,
                    exp_fmt,
                    Some((-4, precision as isize)),
                    true,
                    precision,
                    float,
                    left,
                    pad_space,
                    pad_zero,
                )? {
                    fmt_float_normal(w, true, precision, float, left, pad_space, pad_zero)?;
                }
            },
            FmtKind::String => {
                // if intkind == IntKind::Long || intkind == IntKind::LongLong, handle *const wchar_t

                let mut ptr = match varargs.get(index, &mut ap, Some(&arg)) {
                    VaArg::pointer(p) => p,
                    _ => panic!("this should not be possible")
                } as *const c_char;

                if ptr.is_null() {
                    w.write_all(b"(null)")?;
                } else {
                    let max = precision.unwrap_or(::core::usize::MAX);
                    let mut len = 0;
                    while *ptr.add(len) != 0 && len < max {
                        len += 1;
                    }

                    pad(w, !left, b' ', len..pad_space)?;
                    w.write_all(slice::from_raw_parts(ptr as *const u8, len))?;
                    pad(w, left, b' ', len..pad_space)?;
                }
            },
            FmtKind::Char => {
                // if intkind == IntKind::Long || intkind == IntKind::LongLong, handle wint_t

                let c = match varargs.get(index, &mut ap, Some(&arg)) {
                    VaArg::c_char(c) => c,
                    _ => panic!("this should not be possible")
                };

                pad(w, !left, b' ', 1..pad_space)?;
                w.write_all(&[c as u8])?;
                pad(w, left, b' ', 1..pad_space)?;
            },
            FmtKind::Pointer => {
                let mut ptr = match varargs.get(index, &mut ap, Some(&arg)) {
                    VaArg::pointer(p) => p,
                    _ => panic!("this should not be possible")
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
            },
            FmtKind::GetWritten => {
                let mut ptr = match varargs.get(index, &mut ap, Some(&arg)) {
                    VaArg::pointer(p) => p,
                    _ => panic!("this should not be possible")
                };

                match intkind {
                    IntKind::Byte => *(ptr as *mut c_char) = w.written as c_char,
                    IntKind::Short => *(ptr as *mut c_short) = w.written as c_short,
                    IntKind::Int => *(ptr as *mut c_int) = w.written as c_int,
                    IntKind::Long => *(ptr as *mut c_long) = w.written as c_long,
                    IntKind::LongLong => *(ptr as *mut c_longlong) = w.written as c_longlong,
                    IntKind::IntMax => *(ptr as *mut intmax_t) = w.written as intmax_t,
                    IntKind::PtrDiff => *(ptr as *mut ptrdiff_t) = w.written as ptrdiff_t,
                    IntKind::Size => *(ptr as *mut size_t) = w.written as size_t
                }
            }
        }
    }
    Ok(w.written as c_int)
}

pub unsafe fn printf<W: Write>(w: W, format: *const c_char, ap: VaList) -> c_int {
    inner_printf(w, format, ap).unwrap_or(-1)
}
