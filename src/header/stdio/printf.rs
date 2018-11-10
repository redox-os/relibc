use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::ops::Range;
use core::{fmt, slice};

use io::{self, Write};
use platform;
use platform::types::*;
use va_list::{VaList, VaPrimitive};

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

trait IntoUsize {
    fn into_usize(self) -> usize;
    fn from_usize(i: usize) -> Self;
}
macro_rules! impl_intousize {
    ($($kind:tt;)*) => {
        $(impl IntoUsize for $kind {
            fn into_usize(self) -> usize {
                self as usize
            }
            fn from_usize(i: usize) -> Self {
                i as Self
            }
        })*
    }
}
impl_intousize! {
    i32;
    u32;
    i64;
    u64;
    isize;
    usize;
}
impl<T> IntoUsize for *const T {
    fn into_usize(self) -> usize {
        self as usize
    }
    fn from_usize(i: usize) -> Self {
        i as Self
    }
}
impl<T> IntoUsize for *mut T {
    fn into_usize(self) -> usize {
        self as usize
    }
    fn from_usize(i: usize) -> Self {
        i as Self
    }
}
impl IntoUsize for f32 {
    fn into_usize(self) -> usize {
        self.to_bits() as usize
    }
    fn from_usize(i: usize) -> Self {
        Self::from_bits(i as u32)
    }
}
impl IntoUsize for f64 {
    fn into_usize(self) -> usize {
        self.to_bits() as usize
    }
    fn from_usize(i: usize) -> Self {
        Self::from_bits(i as u64)
    }
}

struct BufferedVaList {
    list: VaList,
    buf: Vec<usize>,
    i: usize
}
impl BufferedVaList {
    fn new(list: VaList) -> Self {
        Self {
            list,
            buf: Vec::new(),
            i: 0
        }
    }
    unsafe fn get<T: VaPrimitive + IntoUsize>(&mut self, i: Option<usize>) -> T {
        match i {
            None => self.next(),
            Some(i) => self.index(i),
        }
    }
    unsafe fn next<T: VaPrimitive + IntoUsize>(&mut self) -> T {
        if self.i >= self.buf.len() {
            self.buf.push(self.list.get::<T>().into_usize());
        }
        let arg = T::from_usize(self.buf[self.i]);
        self.i += 1;
        arg
    }
    unsafe fn index<T: VaPrimitive + IntoUsize>(&mut self, i: usize) -> T {
        while self.buf.len() < i {
            // Getting a usize here most definitely isn't sane, however,
            // there's no way to know the type!
            // Just take this for example:
            //
            // printf("%*4$d\n", "hi", 0, "hello", 10);
            //
            // This chooses the width 10. How does it know the type of 0 and "hello"?
            // It clearly can't.

            self.buf.push(self.list.get::<usize>());
        }
        T::from_usize(self.buf[i - 1])
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
                return Some(ap.index::<usize>(i))
            }
        }

        Some(ap.next::<usize>())
    } else {
        pop_int_raw(format)
    }
}
unsafe fn fmt_int<I>(fmt: u8, i: I) -> String
    where I: fmt::Display + fmt::Octal + fmt::LowerHex + fmt::UpperHex
{
    match fmt {
        b'o' => format!("{:o}", i),
        b'u' => i.to_string(),
        b'x' => format!("{:x}", i),
        b'X' => format!("{:X}", i),
        _ => panic!("fmt_int should never be called with the fmt {}", fmt)
    }
}
fn pad<W: Write>(w: &mut W, current_side: bool, pad_char: u8, range: Range<usize>) -> io::Result<()> {
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

    if range.map(|(start, end)| exp >= start && exp < end).unwrap_or(false) {
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
unsafe fn inner_printf<W: Write>(w: W, format: *const c_char, ap: VaList) -> io::Result<c_int> {
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
                    _ => break
                }
                format = format.offset(1);
            }

            // Width and precision:
            let min_width = pop_int(&mut format, &mut ap).unwrap_or(0);
            let precision = if *format == b'.' {
                format = format.offset(1);
                match pop_int(&mut format, &mut ap) {
                    int@Some(_) => int,
                    None => return Ok(-1)
                }
            } else {
                None
            };

            let pad_space = if zero { 0 } else { min_width };
            let pad_zero  = if zero { min_width } else { 0 };

            // Integer size:
            let mut kind = IntKind::Int;
            loop {
                kind = match *format {
                    b'h' => if kind == IntKind::Short || kind == IntKind::Byte {
                        IntKind::Byte
                    } else {
                        IntKind::Short
                    },
                    b'j' => IntKind::IntMax,
                    b'l' => if kind == IntKind::Long || kind == IntKind::LongLong {
                        IntKind::LongLong
                    } else {
                        IntKind::Long
                    },
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
                        // VaList does not seem to support these two:
                        //   IntKind::Byte     => ap.get::<c_char>(index).to_string(),
                        //   IntKind::Short    => ap.get::<c_short>(index).to_string(),
                        IntKind::Byte     => (ap.get::<c_int>(index) as c_char).to_string(),
                        IntKind::Short    => (ap.get::<c_int>(index) as c_short).to_string(),
                        IntKind::Int      => ap.get::<c_int>(index).to_string(),
                        IntKind::Long     => ap.get::<c_long>(index).to_string(),
                        IntKind::LongLong => ap.get::<c_longlong>(index).to_string(),
                        IntKind::PtrDiff  => ap.get::<ptrdiff_t>(index).to_string(),
                        IntKind::Size     => ap.get::<ssize_t>(index).to_string(),
                        IntKind::IntMax   => ap.get::<intmax_t>(index).to_string(),
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
                b'o' | b'u' | b'x' | b'X' => {
                    let fmt = *format;
                    let string = match kind {
                        // VaList does not seem to support these two:
                        //   IntKind::Byte     => fmt_int(kind ap.get::<c_char>(index)),
                        //   IntKind::Short     => fmt_int(kind ap.get::<c_short>(index)),
                        IntKind::Byte     => fmt_int(fmt, ap.get::<c_uint>(index) as c_uchar),
                        IntKind::Short    => fmt_int(fmt, ap.get::<c_uint>(index) as c_ushort),
                        IntKind::Int      => fmt_int(fmt, ap.get::<c_uint>(index)),
                        IntKind::Long     => fmt_int(fmt, ap.get::<c_ulong>(index)),
                        IntKind::LongLong => fmt_int(fmt, ap.get::<c_ulonglong>(index)),
                        IntKind::PtrDiff  => fmt_int(fmt, ap.get::<ptrdiff_t>(index)),
                        IntKind::Size     => fmt_int(fmt, ap.get::<size_t>(index)),
                        IntKind::IntMax   => fmt_int(fmt, ap.get::<uintmax_t>(index)),
                    };
                    let zero = precision == Some(0) && string == "0";

                    // If this int is padded out to be larger than it is, don't
                    // add an extra zero if octal.
                    let no_precision = precision.map(|pad| pad < string.len()).unwrap_or(true);

                    let mut len = string.len();
                    let mut final_len = string.len().max(precision.unwrap_or(0)) +
                        if alternate && string != "0" {
                            match fmt {
                                b'o' if no_precision => 1,
                                b'x' |
                                b'X' => 2,
                                _ => 0
                            }
                        } else { 0 };

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
                            _ => ()
                        }
                    }
                    pad(w, true, b'0', len..precision.unwrap_or(pad_zero))?;

                    if !zero {
                        w.write_all(string.as_bytes())?;
                    }

                    pad(w, left, b' ', final_len..pad_space)?;
                },
                b'e' | b'E' => {
                    let exp_fmt = *format;
                    let mut float = ap.get::<c_double>(index);
                    let precision = precision.unwrap_or(6);

                    fmt_float_exp(w, exp_fmt, None, false, precision, float, left, pad_space, pad_zero)?;
                },
                b'f' | b'F' => {
                    let mut float = ap.get::<c_double>(index);
                    let precision = precision.unwrap_or(6);

                    fmt_float_normal(w, false, precision, float, left, pad_space, pad_zero)?;
                },
                b'g' | b'G' => {
                    let exp_fmt = b'E' | (*format & 32);
                    let mut float = ap.get::<c_double>(index);
                    let precision = precision.unwrap_or(6);

                    if !fmt_float_exp(w, exp_fmt, Some((-4, precision as isize)), true,
                            precision, float, left, pad_space, pad_zero)? {
                        fmt_float_normal(w, true, precision, float, left, pad_space, pad_zero)?;
                    }
                }
                b's' => {
                    // if kind == IntKind::Long || kind == IntKind::LongLong, handle *const wchar_t

                    let ptr = ap.get::<*const c_char>(index);

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
                },
                b'c' => {
                    // if kind == IntKind::Long || kind == IntKind::LongLong, handle wint_t

                    let c = ap.get::<c_int>(index) as c_char;

                    pad(w, !left, b' ', 1..pad_space)?;
                    w.write_all(&[c as u8])?;
                    pad(w, left, b' ', 1..pad_space)?;
                },
                b'p' => {
                    let ptr = ap.get::<*const c_void>(index);

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
                b'n' => {
                    let ptr = ap.get::<*mut c_int>(index);
                    *ptr = w.written as c_int;
                },
                _ => return Ok(-1)
            }
        }
        format = format.offset(1);
    }
    Ok(w.written as c_int)
}
pub unsafe fn printf<W: Write>(w: W, format: *const c_char, ap: VaList) -> c_int {
    inner_printf(w, format, ap).unwrap_or(-1)
}
