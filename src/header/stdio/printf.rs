use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::ffi::VaList as va_list;
use core::ops::Range;
use core::{fmt, slice};
use io::{self, Write};

use platform;
use platform::types::*;

#[derive(PartialEq, Eq)]
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
    unsafe fn resolve(&self, ap: &mut BufferedVaList) -> usize {
        match *self {
            Number::Static(num) => num,
            Number::Index(i) => ap.index(ArgKind::Int, i).int as usize,
            Number::Next => ap.next(ArgKind::Int).int as usize,
        }
    }
}
enum ArgKind {
    Byte,
    Short,
    Int,
    Long,
    LongLong,
    PtrDiff,
    Size,
    IntMax,
    Double,
    CharPtr,
    VoidPtr,
    IntPtr,
    ArgDefault,
}

#[derive(Clone, Copy)]
union VaArg {
    byte: c_char,
    short: c_short,
    int: c_int,
    long: c_long,
    longlong: c_longlong,
    ptrdiff: ptrdiff_t,
    size: ssize_t,
    intmax: intmax_t,
    double: c_double,
    char_ptr: *const c_char,
    void_ptr: *const c_void,
    int_ptr: *mut c_int,
    arg_default: usize,
}

struct BufferedVaList<'a> {
    list: va_list<'a>,
    buf: Vec<VaArg>,
    i: usize,
}

impl<'a> BufferedVaList<'a> {
    fn new(list: va_list<'a>) -> Self {
        Self {
            list,
            buf: Vec::new(),
            i: 0,
        }
    }

    unsafe fn get_arg(&mut self, ty: ArgKind) -> VaArg {
        match ty {
            ArgKind::Byte => VaArg {
                byte: self.list.arg::<c_char>(),
            },
            ArgKind::Short => VaArg {
                short: self.list.arg::<c_short>(),
            },
            ArgKind::Int => VaArg {
                int: self.list.arg::<c_int>(),
            },
            ArgKind::Long => VaArg {
                long: self.list.arg::<c_long>(),
            },
            ArgKind::LongLong => VaArg {
                longlong: self.list.arg::<c_longlong>(),
            },
            ArgKind::PtrDiff => VaArg {
                ptrdiff: self.list.arg::<ptrdiff_t>(),
            },
            ArgKind::Size => VaArg {
                size: self.list.arg::<ssize_t>(),
            },
            ArgKind::IntMax => VaArg {
                intmax: self.list.arg::<intmax_t>(),
            },
            ArgKind::Double => VaArg {
                double: self.list.arg::<c_double>(),
            },
            ArgKind::CharPtr => VaArg {
                char_ptr: self.list.arg::<*const c_char>(),
            },
            ArgKind::VoidPtr => VaArg {
                void_ptr: self.list.arg::<*const c_void>(),
            },
            ArgKind::IntPtr => VaArg {
                int_ptr: self.list.arg::<*mut c_int>(),
            },
            ArgKind::ArgDefault => VaArg {
                arg_default: self.list.arg::<usize>(),
            },
        }
    }

    unsafe fn get(&mut self, ty: ArgKind, i: Option<usize>) -> VaArg {
        match i {
            None => self.next(ty),
            Some(i) => self.index(ty, i),
        }
    }

    unsafe fn next(&mut self, ty: ArgKind) -> VaArg {
        if self.i >= self.buf.len() {
            let arg = self.get_arg(ty);
            self.buf.push(arg);
        }
        let arg = self.buf[self.i];
        self.i += 1;
        arg
    }

    unsafe fn index(&mut self, ty: ArgKind, i: usize) -> VaArg {
        if self.i >= self.buf.len() {
            while self.buf.len() < (i - 1) {
                // Getting a usize here most definitely isn't sane, however,
                // there's no way to know the type!
                // Just take this for example:
                //
                // printf("%*4$d\n", "hi", 0, "hello", 10);
                //
                // This chooses the width 10. How does it know the type of 0 and "hello"?
                // It clearly can't.

                let arg = self.get_arg(ArgKind::ArgDefault);
                self.buf.push(arg);
            }
            let arg = self.get_arg(ty);
            self.buf.push(arg);
        }
        self.buf[i - 1]
    }
}

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

unsafe fn inner_printf<W: Write>(w: W, format: *const c_char, ap: va_list) -> io::Result<c_int> {
    let w = &mut platform::CountingWriter::new(w);
    let mut ap = BufferedVaList::new(ap);

    let iterator = PrintfIter {
        format: format as *const u8
    };

    for section in iterator {
        let arg = match section {
            Ok(PrintfFmt::Plain(text)) => {
                w.write_all(text)?;
                continue;
            },
            Ok(PrintfFmt::Arg(arg)) => arg,
            Err(()) => return Ok(-1)
        };
        let index = arg.index;
        let alternate = arg.alternate;
        let zero = arg.zero;
        let left = arg.left;
        let sign_reserve = arg.sign_reserve;
        let sign_always = arg.sign_always;
        let min_width = arg.min_width.resolve(&mut ap);
        let precision = arg.precision.map(|n| n.resolve(&mut ap));
        let pad_space = arg.pad_space.resolve(&mut ap);
        let pad_zero = arg.pad_zero.resolve(&mut ap);
        let intkind = arg.intkind;
        let fmt = arg.fmt;
        let fmtkind = arg.fmtkind;

        // Finally, type:
        match fmtkind {
            FmtKind::Percent => w.write_all(&[b'%'])?,
            FmtKind::Signed => {
                let string = match intkind {
                    // Per the C standard using va_arg with a type with a size
                    // less than that of an int for integers and double for floats
                    // is invalid. As a result any arguments smaller than an int or
                    // double passed to a function will be promoted to the smallest
                    // possible size. The va_list::arg function will handle this
                    // automagically.
                    IntKind::Byte => ap.get(ArgKind::Byte, index).byte.to_string(),
                    IntKind::Short => ap.get(ArgKind::Short, index).short.to_string(),
                    // Types that will not be promoted
                    IntKind::Int => ap.get(ArgKind::Int, index).int.to_string(),
                    IntKind::Long => ap.get(ArgKind::Long, index).long.to_string(),
                    IntKind::LongLong => ap.get(ArgKind::LongLong, index).longlong.to_string(),
                    IntKind::PtrDiff => ap.get(ArgKind::PtrDiff, index).ptrdiff.to_string(),
                    IntKind::Size => ap.get(ArgKind::Size, index).size.to_string(),
                    IntKind::IntMax => ap.get(ArgKind::IntMax, index).intmax.to_string(),
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
                let string = match intkind {
                    // va_list will promote the following two to a c_int
                    IntKind::Byte => fmt_int(fmt, ap.get(ArgKind::Byte, index).byte),
                    IntKind::Short => fmt_int(fmt, ap.get(ArgKind::Short, index).short),
                    IntKind::Int => fmt_int(fmt, ap.get(ArgKind::Int, index).int),
                    IntKind::Long => fmt_int(fmt, ap.get(ArgKind::Long, index).long),
                    IntKind::LongLong => {
                        fmt_int(fmt, ap.get(ArgKind::LongLong, index).longlong)
                    }
                    IntKind::PtrDiff => fmt_int(fmt, ap.get(ArgKind::PtrDiff, index).ptrdiff),
                    IntKind::Size => fmt_int(fmt, ap.get(ArgKind::Size, index).size),
                    IntKind::IntMax => fmt_int(fmt, ap.get(ArgKind::IntMax, index).intmax),
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
                let mut float = ap.get(ArgKind::Double, index).double;
                let precision = precision.unwrap_or(6);

                fmt_float_exp(w, fmt, None, false, precision, float, left, pad_space, pad_zero)?;
            },
            FmtKind::Decimal => {
                let mut float = ap.get(ArgKind::Double, index).double;
                let precision = precision.unwrap_or(6);

                fmt_float_normal(w, false, precision, float, left, pad_space, pad_zero)?;
            },
            FmtKind::AnyNotation => {
                let exp_fmt = b'E' | (fmt & 32);
                let mut float = ap.get(ArgKind::Double, index).double;
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

                let ptr = ap.get(ArgKind::CharPtr, index).char_ptr;

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

                let c = ap.get(ArgKind::Byte, index).byte;

                pad(w, !left, b' ', 1..pad_space)?;
                w.write_all(&[c as u8])?;
                pad(w, left, b' ', 1..pad_space)?;
            },
            FmtKind::Pointer => {
                let ptr = ap.get(ArgKind::VoidPtr, index).int_ptr;

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
                let ptr = ap.get(ArgKind::IntPtr, index).int_ptr;
                *ptr = w.written as c_int;
            }
        }
    }
    Ok(w.written as c_int)
}

pub unsafe fn printf<W: Write>(w: W, format: *const c_char, ap: va_list) -> c_int {
    inner_printf(w, format, ap).unwrap_or(-1)
}
