use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::ops::Range;
use core::{fmt, slice};
use core::ffi::VaList as va_list;
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

enum ArgType {
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
    ArgDefault
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
    arg_default: usize
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

    unsafe fn get_arg(&mut self, ty: ArgType) -> VaArg {
        match ty {
            ArgType::Byte => VaArg { byte: self.list.arg::<c_char>() },
            ArgType::Short => VaArg { short: self.list.arg::<c_short>() },
            ArgType::Int => VaArg { int: self.list.arg::<c_int>() },
            ArgType::Long => VaArg { long: self.list.arg::<c_long>() },
            ArgType::LongLong => VaArg { longlong: self.list.arg::<c_longlong>() },
            ArgType::PtrDiff => VaArg { ptrdiff: self.list.arg::<ptrdiff_t>() },
            ArgType::Size => VaArg { size: self.list.arg::<ssize_t>() },
            ArgType::IntMax => VaArg { intmax: self.list.arg::<intmax_t>() },
            ArgType::Double => VaArg { double: self.list.arg::<c_double>() },
            ArgType::CharPtr => VaArg { char_ptr: self.list.arg::<*const c_char>() },
            ArgType::VoidPtr => VaArg { void_ptr: self.list.arg::<*const c_void>() },
            ArgType::IntPtr => VaArg { int_ptr: self.list.arg::<*mut c_int>() },
            ArgType::ArgDefault => VaArg { arg_default: self.list.arg::<usize>() }
        }
    }

    unsafe fn get(&mut self, ty: ArgType, i: Option<usize>) -> VaArg {
        match i {
            None => self.next(ty),
            Some(i) => self.index(ty, i),
        }
    }

    unsafe fn next(&mut self, ty: ArgType) -> VaArg {
        if self.i >= self.buf.len() {
            let arg = self.get_arg(ty);
            self.buf.push(arg);
        }
        let arg = self.buf[self.i];
        self.i += 1;
        arg
    }

    unsafe fn index(&mut self, ty: ArgType, i: usize) -> VaArg {
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

                let arg = self.get_arg(ArgType::ArgDefault);
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
        *format = format.offset(1);
        if int.is_none() {
            int = Some(0);
        }
        *int.as_mut().unwrap() *= 10;
        *int.as_mut().unwrap() += digit as usize;
    }
    int
}

unsafe fn pop_int(format: &mut *const u8, ap: &mut BufferedVaList) -> Option<usize> {
    if **format == b'*' {
        *format = format.offset(1);

        // Peek ahead for a positional argument:
        let mut format2 = *format;
        if let Some(i) = pop_int_raw(&mut format2) {
            if *format2 == b'$' {
                *format = format2.offset(1);
                return Some(ap.index(ArgType::ArgDefault, i).arg_default);
            }
        }

        Some(ap.next(ArgType::ArgDefault).arg_default)
    } else {
        pop_int_raw(format)
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
        _ => panic!("fmt_int should never be called with the fmt {}", fmt),
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
            let mut slice = string.trim_right_matches('0');
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

unsafe fn inner_printf<W: Write>(w: W, format: *const c_char, ap: va_list) -> io::Result<c_int> {
    let w = &mut platform::CountingWriter::new(w);
    let mut ap = BufferedVaList::new(ap);
    let mut format = format as *const u8;

    while *format != 0 {
        if *format != b'%' {
            w.write_all(&[*format])?;
        } else {
            format = format.offset(1);

            // Peek ahead to maybe specify argument to fetch from
            let mut index = None;
            let mut format2 = format;
            if let Some(i) = pop_int_raw(&mut format2) {
                if *format2 == b'$' {
                    format = format2.offset(1);
                    index = Some(i);
                }
            }

            // Flags:
            let mut alternate = false;
            let mut zero = false;
            let mut left = false;
            let mut sign_reserve = false;
            let mut sign_always = false;

            loop {
                match *format {
                    b'#' => alternate = true,
                    b'0' => zero = true,
                    b'-' => left = true,
                    b' ' => sign_reserve = true,
                    b'+' => sign_always = true,
                    _ => break,
                }
                format = format.offset(1);
            }

            // Width and precision:
            let min_width = pop_int(&mut format, &mut ap).unwrap_or(0);
            let precision = if *format == b'.' {
                format = format.offset(1);
                match pop_int(&mut format, &mut ap) {
                    int @ Some(_) => int,
                    None => return Ok(-1),
                }
            } else {
                None
            };

            let pad_space = if zero { 0 } else { min_width };
            let pad_zero = if zero { min_width } else { 0 };

            // Integer size:
            let mut kind = IntKind::Int;
            loop {
                kind = match *format {
                    b'h' => {
                        if kind == IntKind::Short || kind == IntKind::Byte {
                            IntKind::Byte
                        } else {
                            IntKind::Short
                        }
                    }
                    b'j' => IntKind::IntMax,
                    b'l' => {
                        if kind == IntKind::Long || kind == IntKind::LongLong {
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

                format = format.offset(1);
            }

            // Finally, type:
            match *format {
                b'%' => w.write_all(&[b'%'])?,
                b'd' | b'i' => {
                    let string = match kind {
                        // Per the C standard using va_arg with a type with a size
                        // less than that of an int for integers and double for floats
                        // is invalid. As a result any arguments smaller than an int or
                        // double passed to a function will be promoted to the smallest
                        // possible size. The va_list::arg function will handle this
                        // automagically.
                        IntKind::Byte => ap.get(ArgType::Byte, index).byte.to_string(),
                        IntKind::Short => ap.get(ArgType::Short, index).short.to_string(),
                        // Types that will not be promoted
                        IntKind::Int => ap.get(ArgType::Int, index).int.to_string(),
                        IntKind::Long => ap.get(ArgType::Long, index).long.to_string(),
                        IntKind::LongLong => ap.get(ArgType::LongLong, index).longlong.to_string(),
                        IntKind::PtrDiff => ap.get(ArgType::PtrDiff, index).ptrdiff.to_string(),
                        IntKind::Size => ap.get(ArgType::Size, index).size.to_string(),
                        IntKind::IntMax => ap.get(ArgType::IntMax, index).intmax.to_string(),
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
                b'o' | b'u' | b'x' | b'X' => {
                    let fmt = *format;
                    let string = match kind {
                        // va_list will promote the following two to a c_int
                        IntKind::Byte => fmt_int(fmt, ap.get(ArgType::Byte, index).byte),
                        IntKind::Short => fmt_int(fmt, ap.get(ArgType::Short, index).short),
                        IntKind::Int => fmt_int(fmt, ap.get(ArgType::Int, index).int),
                        IntKind::Long => fmt_int(fmt, ap.get(ArgType::Long, index).long),
                        IntKind::LongLong => fmt_int(fmt, ap.get(ArgType::LongLong, index).longlong),
                        IntKind::PtrDiff => fmt_int(fmt, ap.get(ArgType::PtrDiff, index).ptrdiff),
                        IntKind::Size => fmt_int(fmt, ap.get(ArgType::Size, index).size),
                        IntKind::IntMax => fmt_int(fmt, ap.get(ArgType::IntMax, index).intmax),
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
                }
                b'e' | b'E' => {
                    let exp_fmt = *format;
                    let mut float = ap.get(ArgType::Double, index).double;
                    let precision = precision.unwrap_or(6);

                    fmt_float_exp(
                        w, exp_fmt, None, false, precision, float, left, pad_space, pad_zero,
                    )?;
                }
                b'f' | b'F' => {
                    let mut float = ap.get(ArgType::Double, index).double;
                    let precision = precision.unwrap_or(6);

                    fmt_float_normal(w, false, precision, float, left, pad_space, pad_zero)?;
                }
                b'g' | b'G' => {
                    let exp_fmt = b'E' | (*format & 32);
                    let mut float = ap.get(ArgType::Double, index).double;
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
                }
                b's' => {
                    // if kind == IntKind::Long || kind == IntKind::LongLong, handle *const wchar_t

                    let ptr = ap.get(ArgType::CharPtr, index).char_ptr;

                    if ptr.is_null() {
                        w.write_all(b"(null)")?;
                    } else {
                        let mut len = 0;
                        while *ptr.offset(len) != 0 {
                            len += 1;
                        }

                        let len = (len as usize).min(precision.unwrap_or(::core::usize::MAX));

                        pad(w, !left, b' ', len..pad_space)?;
                        w.write_all(slice::from_raw_parts(ptr as *const u8, len))?;
                        pad(w, left, b' ', len..pad_space)?;
                    }
                }
                b'c' => {
                    // if kind == IntKind::Long || kind == IntKind::LongLong, handle wint_t

                    let c = ap.get(ArgType::Byte, index).byte;

                    pad(w, !left, b' ', 1..pad_space)?;
                    w.write_all(&[c as u8])?;
                    pad(w, left, b' ', 1..pad_space)?;
                }
                b'p' => {
                    let ptr = ap.get(ArgType::VoidPtr, index).int_ptr;

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
                b'n' => {
                    let ptr = ap.get(ArgType::IntPtr, index).int_ptr;
                    *ptr = w.written as c_int;
                }
                _ => return Ok(-1),
            }
        }
        format = format.offset(1);
    }
    Ok(w.written as c_int)
}

pub unsafe fn printf<W: Write>(w: W, format: *const c_char, ap: va_list) -> c_int {
    inner_printf(w, format, ap).unwrap_or(-1)
}
