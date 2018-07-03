use alloc::String;
use alloc::Vec;
use platform::types::*;
use platform::Read;
#[cfg(not(test))]
use vl::VaList;

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

/// Helper function for progressing a C string
unsafe fn next_byte(string: &mut *const c_char) -> Result<u8, c_int> {
    let c = **string as u8;
    *string = string.offset(1);
    if c == 0 {
        Err(-1)
    } else {
        Ok(c)
    }
}

unsafe fn inner_scanf<R: Read>(
    mut r: R,
    mut format: *const c_char,
    mut ap: VaList,
) -> Result<c_int, c_int> {
    let mut matched = 0;
    let mut byte = 0;
    let mut skip_read = false;
    let mut count = 0;

    macro_rules! read {
        () => {{
            let n = r.read_u8(&mut byte);
            if n {
                count += 1;
            }
            n
        }};
    }

    macro_rules! maybe_read {
        () => {
            maybe_read!(inner false);
        };
        (noreset) => {
            maybe_read!(inner);
        };
        (inner $($placeholder:expr)*) => {
            if !skip_read {
                if !read!() {
                    return Ok(matched);
                }
            }
            $(else {
                // Hacky way of having this optional
                skip_read = $placeholder;
            })*
        }
    }

    while *format != 0 {
        let mut c = *format as u8;
        format = format.offset(1);

        if c == b' ' {
            maybe_read!(noreset);

            while (byte as char).is_whitespace() {
                if !read!() {
                    return Ok(matched);
                }
            }

            skip_read = true;
        } else if c != b'%' {
            maybe_read!();
            if c != byte {
                return Ok(matched);
            }
        } else {
            c = next_byte(&mut format)?;

            let mut ignore = false;
            if c == b'*' {
                ignore = true;
                c = next_byte(&mut format)?;
            }

            let mut width = String::new();
            while c >= b'0' && c <= b'9' {
                width.push(c as char);
                c = next_byte(&mut format)?;
            }
            let mut width = if width.is_empty() {
                None
            } else {
                match width.parse::<usize>() {
                    Ok(n) => Some(n),
                    Err(_) => return Err(-1),
                }
            };

            let mut kind = IntKind::Int;
            loop {
                kind = match c {
                    b'h' => if kind == IntKind::Short {
                        IntKind::Byte
                    } else {
                        IntKind::Short
                    },
                    b'j' => IntKind::IntMax,
                    b'l' => IntKind::Long,
                    b'q' | b'L' => IntKind::LongLong,
                    b't' => IntKind::PtrDiff,
                    b'z' => IntKind::Size,
                    _ => break,
                };

                c = next_byte(&mut format)?;
            }

            if c != b'n' {
                maybe_read!(noreset);
            }
            match c {
                b'%' => {
                    while (byte as char).is_whitespace() {
                        if !read!() {
                            return Ok(matched);
                        }
                    }

                    if byte != b'%' {
                        return Err(matched);
                    } else if !read!() {
                        return Ok(matched);
                    }
                }
                b'd' | b'i' | b'o' | b'u' | b'x' | b'X' | b'f' | b'e' | b'g' | b'E' | b'a'
                | b'p' => {
                    while (byte as char).is_whitespace() {
                        if !read!() {
                            return Ok(matched);
                        }
                    }

                    let pointer = c == b'p';
                    // Pointers aren't automatic, but we do want to parse "0x"
                    let auto = c == b'i' || pointer;
                    let float = c == b'f' || c == b'e' || c == b'g' || c == b'E' || c == b'a';

                    let mut radix = match c {
                        b'o' => 8,
                        b'x' | b'X' | b'p' => 16,
                        _ => 10,
                    };

                    let mut n = String::new();
                    let mut dot = false;

                    while width.map(|w| w > 0).unwrap_or(true)
                        && ((byte >= b'0' && byte <= b'7')
                            || (radix >= 10 && (byte >= b'8' && byte <= b'9'))
                            || (float && !dot && byte == b'.')
                            || (radix == 16
                                && ((byte >= b'a' && byte <= b'f')
                                    || (byte >= b'A' && byte <= b'F'))))
                    {
                        if auto
                            && n.is_empty()
                            && byte == b'0'
                            && width.map(|w| w > 0).unwrap_or(true)
                        {
                            if !pointer {
                                radix = 8;
                            }
                            width = width.map(|w| w - 1);
                            if !read!() {
                                break;
                            }
                            if width.map(|w| w > 0).unwrap_or(true)
                                && (byte == b'x' || byte == b'X')
                            {
                                radix = 16;
                                width = width.map(|w| w - 1);
                                if width.map(|w| w > 0).unwrap_or(true) {
                                    if !read!() {
                                        break;
                                    }
                                }
                            }
                            continue;
                        }
                        if byte == b'.' {
                            // Don't allow another dot
                            dot = true;
                        }
                        n.push(byte as char);
                        width = width.map(|w| w - 1);
                        if width.map(|w| w > 0).unwrap_or(true) {
                            if !read!() {
                                break;
                            }
                        }
                    }

                    macro_rules! parse_type {
                        (noformat $type:ident) => {{
                            let n = if n.is_empty() {
                                0 as $type
                            } else {
                                n.parse::<$type>().map_err(|_| 0)?
                            };
                            if !ignore {
                                *ap.get::<*mut $type>() = n;
                                matched += 1;
                            }
                        }};
                        (c_double) => {
                            parse_type!(noformat c_double);
                        };
                        (c_float) => {
                            parse_type!(noformat c_float);
                        };
                        ($type:ident) => {
                            parse_type!($type, $type);
                        };
                        ($type:ident, $final:ty) => {{
                            let n = if n.is_empty() {
                                0 as $type
                            } else {
                                $type::from_str_radix(&n, radix).map_err(|_| 0)?
                            };
                            if !ignore {
                                *ap.get::<*mut $final>() = n as $final;
                                matched += 1;
                            }
                        }};
                    }

                    if float {
                        if kind == IntKind::Long || kind == IntKind::LongLong {
                            parse_type!(c_double);
                        } else {
                            parse_type!(c_float);
                        }
                    } else if c == b'p' {
                        parse_type!(size_t, *mut c_void);
                    } else {
                        let unsigned = c == b'o' || c == b'u' || c == b'x' || c == b'X';

                        match kind {
                            IntKind::Byte => if unsigned {
                                parse_type!(c_uchar);
                            } else {
                                parse_type!(c_char);
                            },
                            IntKind::Short => if unsigned {
                                parse_type!(c_ushort)
                            } else {
                                parse_type!(c_short)
                            },
                            IntKind::Int => if unsigned {
                                parse_type!(c_uint)
                            } else {
                                parse_type!(c_int)
                            },
                            IntKind::Long => if unsigned {
                                parse_type!(c_ulong)
                            } else {
                                parse_type!(c_long)
                            },
                            IntKind::LongLong => if unsigned {
                                parse_type!(c_ulonglong)
                            } else {
                                parse_type!(c_longlong)
                            },
                            IntKind::IntMax => if unsigned {
                                parse_type!(uintmax_t)
                            } else {
                                parse_type!(intmax_t)
                            },
                            IntKind::PtrDiff => parse_type!(ptrdiff_t),
                            IntKind::Size => if unsigned {
                                parse_type!(size_t)
                            } else {
                                parse_type!(ssize_t)
                            },
                        }
                    }
                }
                b's' => {
                    while (byte as char).is_whitespace() {
                        if !read!() {
                            return Ok(matched);
                        }
                    }

                    let mut ptr: Option<*mut c_char> = if ignore { None } else { Some(ap.get()) };

                    while width.map(|w| w > 0).unwrap_or(true) && !(byte as char).is_whitespace() {
                        if let Some(ref mut ptr) = ptr {
                            **ptr = byte as c_char;
                            *ptr = ptr.offset(1);
                        }
                        width = width.map(|w| w - 1);
                        if width.map(|w| w > 0).unwrap_or(true) {
                            if !read!() {
                                break;
                            }
                        }
                    }

                    if let Some(ptr) = ptr {
                        *ptr = 0;
                        matched += 1;
                    }
                }
                b'c' => {
                    let mut ptr: Option<*mut c_char> = if ignore { None } else { Some(ap.get()) };

                    for i in 0..width.unwrap_or(1) {
                        if let Some(ptr) = ptr {
                            *ptr.offset(i as isize) = byte as c_char;
                        }
                        width = width.map(|w| w - 1);
                        if width.map(|w| w > 0).unwrap_or(true) {
                            if !read!() {
                                break;
                            }
                        }
                    }

                    if ptr.is_some() {
                        matched += 1;
                    }
                }
                b'[' => {
                    c = next_byte(&mut format)?;

                    let mut matches = Vec::new();
                    let mut invert = false;
                    if c == b'^' {
                        c = next_byte(&mut format)?;
                        invert = true;
                    }

                    let mut prev;
                    loop {
                        matches.push(c);
                        prev = c;
                        c = next_byte(&mut format)?;
                        if c == b'-' {
                            if prev == b']' {
                                continue;
                            }
                            c = next_byte(&mut format)?;
                            if c == b']' {
                                matches.push(b'-');
                                break;
                            }
                            prev += 1;
                            while prev < c {
                                matches.push(prev);
                                prev += 1;
                            }
                        } else if c == b']' {
                            break;
                        }
                    }

                    let mut ptr: Option<*mut c_char> = if ignore { None } else { Some(ap.get()) };

                    while width.map(|w| w > 0).unwrap_or(true) && !invert == matches.contains(&byte)
                    {
                        if let Some(ref mut ptr) = ptr {
                            **ptr = byte as c_char;
                            *ptr = ptr.offset(1);
                        }
                        width = width.map(|w| w - 1);
                        if width.map(|w| w > 0).unwrap_or(true) {
                            if !read!() {
                                break;
                            }
                        }
                    }

                    if let Some(ptr) = ptr {
                        *ptr = 0;
                        matched += 1;
                    }
                }
                b'n' => {
                    if !ignore {
                        *ap.get::<*mut c_int>() = count as c_int;
                    }
                }
                _ => return Err(-1),
            }

            if width != Some(0) && c != b'n' {
                // It didn't hit the width, so an extra character was read and matched.
                // But this character did not match so let's reuse it.
                skip_read = true;
            }
        }
    }
    Ok(matched)
}
pub unsafe fn scanf<R: Read>(r: R, format: *const c_char, ap: VaList) -> c_int {
    match inner_scanf(r, format, ap) {
        Ok(n) => n,
        Err(n) => n,
    }
}

#[cfg(test)]
pub trait VaPrimitive {
    fn as_mut_ptr(&mut self) -> *mut c_void;
}
#[cfg(test)]
macro_rules! va_primitives {
    ($($type:ty),+) => {
        $(impl VaPrimitive for $type {
            fn as_mut_ptr(&mut self) -> *mut c_void {
                self as *mut _ as *mut c_void
            }
        })+
    }
}
#[cfg(test)]
va_primitives!(
    c_uchar,
    c_ushort,
    c_uint,
    c_ulong,
    c_char,
    c_short,
    c_int,
    c_long,
    c_float,
    c_double,
    *mut c_void
);

#[cfg(test)]
trait FromVoidPtr {
    fn from_void_ptr(ptr: *mut c_void) -> Self;
}
#[cfg(test)]
macro_rules! from_void_ptr {
    ($($type:ty),+) => {
        $(impl FromVoidPtr for *mut $type {
            fn from_void_ptr(ptr: *mut c_void) -> Self {
                ptr as *mut _ as Self
            }
        })+
    }
}
#[cfg(test)]
from_void_ptr!(
    c_uchar,
    c_ushort,
    c_uint,
    c_ulong,
    c_char,
    c_short,
    c_int,
    c_long,
    c_float,
    c_double,
    size_t,
    ssize_t,
    *mut c_void
);

#[cfg(test)]
pub struct VaList<'a> {
    args: &'a mut [&'a mut VaPrimitive],
    i: usize,
}
#[cfg(test)]
impl<'a> VaList<'a> {
    pub fn new(args: &'a mut [&'a mut VaPrimitive]) -> VaList<'a> {
        VaList { args: args, i: 0 }
    }
    pub fn get<T: FromVoidPtr>(&mut self) -> T {
        let ptr = T::from_void_ptr(self.args[self.i].as_mut_ptr());
        self.i += 1;
        ptr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ptr;
    use platform::StringReader;

    fn scanf<'a>(format: &str, args: &'a mut [&'a mut VaPrimitive], input: &str) -> c_int {
        unsafe {
            let mut format = format.to_string();
            let format = format.as_bytes_mut();
            let format = format.as_mut_ptr();
            let out = super::inner_scanf(
                StringReader(input.as_bytes()),
                format as *mut c_char,
                VaList::new(args),
            ).expect("running test scanf failed");
            out
        }
    }

    #[test]
    fn ints() {
        let mut a: c_char = 0;
        let mut b: c_int = 0;
        assert_eq!(scanf("%hhd %d\0", &mut [&mut a, &mut b], "12 345"), 2);
        assert_eq!(a, 12);
        assert_eq!(b, 345);
    }
    #[test]
    fn formats() {
        let mut a: c_uint = 0;
        let mut b: c_int = 0;
        let mut c: c_int = 0;
        assert_eq!(
            scanf("%x %i %i\0", &mut [&mut a, &mut b, &mut c], "12 0x345 010"),
            3
        );
        assert_eq!(a, 0x12);
        assert_eq!(b, 0x345);
        assert_eq!(c, 0o10);
    }
    #[test]
    fn floats() {
        let mut a: c_float = 0.0;
        let mut b: c_double = 0.0;
        assert_eq!(scanf("%f.%lf\0", &mut [&mut a, &mut b], "0.1.0.2"), 2);
        assert_eq!(a, 0.1);
        assert_eq!(b, 0.2);
    }
    #[test]
    fn pointer() {
        let mut a: *mut c_void = ptr::null_mut();
        assert_eq!(scanf("%p\0", &mut [&mut a], "0xABCDEF"), 1);
        assert_eq!(a as usize, 0xABCDEF);
    }
    #[test]
    fn string() {
        let mut a = [1u8; 10];
        assert_eq!(scanf("%s\0", &mut [&mut (a[0])], "Hello World"), 1);
        assert_eq!(&a[..6], b"Hello\0");
        assert_eq!(a[6..], [1; 4]); // nothing else was modified
    }
    #[test]
    fn width() {
        let mut a: c_int = 0;
        assert_eq!(scanf("%3i\0", &mut [&mut a], "0xFF"), 1);
        assert_eq!(a, 0xF);
    }
    #[test]
    fn chars() {
        let mut a: u8 = 0;
        let mut b = [1u8; 5];
        assert_eq!(scanf("%c%3c\0", &mut [&mut a, &mut (b[0])], "hello"), 2);
        assert_eq!(a, b'h');
        assert_eq!(&b[..3], b"ell");
        assert_eq!(&b[3..5], [1; 2]); // nothing else was modified, no trailing NUL-byte
    }
    #[test]
    fn count() {
        let mut a: c_int = 0;
        let mut b: c_int = 0;
        assert_eq!(
            scanf("test: %2i%n\0", &mut [&mut a, &mut b], "test: 0xFF"),
            1
        );
        assert_eq!(a, 0);
        assert_eq!(b, 8);
    }
    #[test]
    fn literal() {
        assert_eq!(scanf("hello world%%\0", &mut [], "hello world%"), 0);
    }
}
