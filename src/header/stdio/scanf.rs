use super::reader::Reader;
use crate::{
    c_str::Kind,
    header::stdio::printf::IntKind,
    platform::types::{
        c_char, c_double, c_float, c_int, c_long, c_longlong, c_short, c_uchar, c_uint, c_ulong,
        c_ulonglong, c_ushort, c_void, intmax_t, ptrdiff_t, size_t, ssize_t, uintmax_t, wchar_t,
    },
};
use alloc::{string::String, vec::Vec};
use core::{ffi::VaList as va_list, iter::Peekable};

#[derive(PartialEq, Eq)]
enum CharKind {
    Ascii,
    Wide,
}

fn next_char<T: Kind>(lar: &mut Peekable<Reader<'_, T>>) -> Result<char, c_int> {
    if let Some(c) = lar.next().transpose()? {
        char::try_from(c.into()).map_err(|_| -1)
    } else {
        Ok('\0')
    }
}

macro_rules! wc_as_char {
    ($c:ident) => {
        char::try_from($c.into()).map_err(|_| -1)?
    };
}

pub unsafe fn inner_scanf<T: Kind>(
    mut r: Reader<T>,
    format: Reader<T>,
    mut ap: va_list,
) -> Result<c_int, c_int> {
    let mut matched = 0;
    let mut character: char = '\0';
    let mut skip_read = false;
    let mut count = 0;
    let mut format = format.peekable();

    macro_rules! read {
        () => {{
            match r.next() {
                None => false,
                Some(Ok(b)) => {
                    character = wc_as_char!(b);
                    count += 1;
                    true
                }
                Some(Err(x)) => return Err(x),
            }
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
            if !skip_read && !read!() {
                match matched {
                    0 => return Ok(-1),
                    a => return Ok(a),
                }
            }
            $(else {
                // Hacky way of having this optional
                skip_read = $placeholder;
            })*
        }
    }

    while format.peek().is_some() {
        let mut c = next_char(&mut format)?;

        if c == ' ' {
            maybe_read!(noreset);

            while (character).is_whitespace() {
                if !read!() {
                    return Ok(matched);
                }
            }

            skip_read = true;
        } else if c != '%' {
            maybe_read!();
            if c != character {
                return Ok(matched);
            }
        } else {
            c = next_char(&mut format)?;

            let mut ignore = false;
            if c == '*' {
                ignore = true;
                c = next_char(&mut format)?;
            }

            let mut width = String::new();
            while c.is_ascii_digit() {
                width.push(c);
                c = next_char(&mut format)?;
            }
            let mut width = if width.is_empty() {
                None
            } else {
                match width.parse::<usize>() {
                    Ok(n) => Some(n),
                    Err(_) => return Err(-1),
                }
            };

            // When an EOF occurs, eof is set, stuff is marked matched
            // as usual, and finally it is returned
            let mut eof = false;

            let mut kind = IntKind::Int;
            let mut c_kind = CharKind::Ascii;
            loop {
                match c {
                    'h' => {
                        if kind == IntKind::Short || kind == IntKind::Byte {
                            kind = IntKind::Byte;
                        } else {
                            kind = IntKind::Short;
                        }
                    }
                    'j' => kind = IntKind::IntMax,
                    'l' => {
                        if kind == IntKind::Long || kind == IntKind::LongLong {
                            kind = IntKind::LongLong;
                        } else {
                            kind = IntKind::Long;
                        }
                    }
                    'q' | 'L' => kind = IntKind::LongLong,
                    't' => kind = IntKind::PtrDiff,
                    'z' => kind = IntKind::Size,
                    // If kind is Long, means we found a 'l' before finding 'c' or 's'. In this
                    // case the format corresponds to a wide char/string
                    'c' | 's' if kind == IntKind::Long && !T::IS_THIN_NOT_WIDE => {
                        c_kind = CharKind::Wide;
                        break;
                    }
                    _ => break,
                }

                c = next_char(&mut format)?;
            }

            if c != 'n' {
                maybe_read!(noreset);
            }
            match c {
                '%' => {
                    while (character).is_whitespace() {
                        if !read!() {
                            return Ok(matched);
                        }
                    }

                    if character != '%' {
                        return Err(matched);
                    } else if !read!() {
                        return Ok(matched);
                    }
                }

                'd' | 'i' | 'o' | 'u' | 'x' | 'X' | 'f' | 'e' | 'g' | 'E' | 'a' | 'p' => {
                    while character.is_whitespace() {
                        if !read!() {
                            return Ok(matched);
                        }
                    }

                    let pointer = c == 'p';
                    // Pointers aren't automatic, but we do want to parse "0x"
                    let auto = c == 'i' || pointer;
                    let float = c == 'f' || c == 'e' || c == 'g' || c == 'E' || c == 'a';

                    let mut radix = match c {
                        'o' => 8,
                        'x' | 'X' | 'p' => 16,
                        _ => 10,
                    };

                    let mut n = String::new();
                    let mut dot = false;

                    while width.map(|w| w > 0).unwrap_or(true)
                        && (('0'..='7').contains(&character)
                            || (radix >= 10 && ('8'..='9').contains(&character))
                            || (float && !dot && character == '.')
                            || (radix == 16
                                && (('a'..='f').contains(&character)
                                    || ('A'..='F').contains(&character))))
                    {
                        if auto
                            && n.is_empty()
                            && character == '0'
                            && width.map(|w| w > 0).unwrap_or(true)
                        {
                            if !pointer {
                                radix = 8;
                            }
                            width = width.map(|w| w - 1);
                            if !read!() {
                                return Ok(matched);
                            }
                            if width.map(|w| w > 0).unwrap_or(true)
                                && (character == 'x' || character == 'X')
                            {
                                radix = 16;
                                width = width.map(|w| w - 1);
                                if width.map(|w| w > 0).unwrap_or(true) && !read!() {
                                    return Ok(matched);
                                }
                            }
                            continue;
                        }
                        if character == '.' {
                            // Don't allow another dot
                            dot = true;
                        }
                        n.push(character);
                        width = width.map(|w| w - 1);
                        if width.map(|w| w > 0).unwrap_or(true) && !read!() {
                            break;
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
                                unsafe { *ap.arg::<*mut $type>() = n };
                                matched += 1;
                            }
                        }};
                        (c_double) => {
                            parse_type!(noformat c_double)
                        };
                        (c_float) => {
                            parse_type!(noformat c_float)
                        };
                        ($type:ident) => {
                            parse_type!($type, $type)
                        };
                        ($type:ident, $final:ty) => {{
                            let n = if n.is_empty() {
                                0 as $type
                            } else {
                                $type::from_str_radix(&n, radix).map_err(|_| 0)?
                            };
                            if !ignore {
                                unsafe { *ap.arg::<*mut $final>() = n as $final };
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
                    } else if c == 'p' {
                        parse_type!(size_t, *mut c_void);
                    } else {
                        let unsigned = c == 'o' || c == 'u' || c == 'x' || c == 'X';

                        match kind {
                            IntKind::Byte => {
                                if unsigned {
                                    parse_type!(c_uchar);
                                } else {
                                    parse_type!(c_char);
                                }
                            }
                            IntKind::Short => {
                                if unsigned {
                                    parse_type!(c_ushort)
                                } else {
                                    parse_type!(c_short)
                                }
                            }
                            IntKind::Int => {
                                if unsigned {
                                    parse_type!(c_uint)
                                } else {
                                    parse_type!(c_int)
                                }
                            }
                            IntKind::Long => {
                                if unsigned {
                                    parse_type!(c_ulong)
                                } else {
                                    parse_type!(c_long)
                                }
                            }
                            IntKind::LongLong => {
                                if unsigned {
                                    parse_type!(c_ulonglong)
                                } else {
                                    parse_type!(c_longlong)
                                }
                            }
                            IntKind::IntMax => {
                                if unsigned {
                                    parse_type!(uintmax_t)
                                } else {
                                    parse_type!(intmax_t)
                                }
                            }
                            IntKind::PtrDiff => parse_type!(ptrdiff_t),
                            IntKind::Size => {
                                if unsigned {
                                    parse_type!(size_t)
                                } else {
                                    parse_type!(ssize_t)
                                }
                            }
                        }
                    }
                }

                's' => {
                    macro_rules! parse_string_type {
                        ($type:ident) => {
                            while character.is_whitespace() {
                                if !read!() {
                                    return Ok(matched);
                                }
                            }

                            let mut ptr: Option<*mut $type> =
                                if ignore { None } else { Some(ap.arg()) };

                            while width.map(|w| w > 0).unwrap_or(true) && !character.is_whitespace()
                            {
                                if let Some(ref mut ptr) = ptr {
                                    **ptr = character as $type;
                                    *ptr = ptr.offset(1);
                                }
                                width = width.map(|w| w - 1);
                                if width.map(|w| w > 0).unwrap_or(true) && !read!() {
                                    eof = true;
                                    break;
                                }
                            }

                            if let Some(ptr) = ptr {
                                *ptr = 0;
                                matched += 1;
                            }
                        };
                    }

                    if c_kind == CharKind::Ascii {
                        unsafe {
                            parse_string_type!(c_char);
                        }
                    } else {
                        unsafe {
                            parse_string_type!(wchar_t);
                        }
                    }
                }

                'c' => {
                    macro_rules! parse_char_type {
                        ($type:ident) => {
                            let ptr: Option<*mut $type> = if ignore {
                                None
                            } else {
                                Some(unsafe { ap.arg() })
                            };

                            for i in 0..width.unwrap_or(1) {
                                if let Some(ptr) = ptr {
                                    unsafe { *ptr.add(i) = character as $type };
                                }
                                width = width.map(|w| w - 1);
                                if width.map(|w| w > 0).unwrap_or(true) && !read!() {
                                    eof = true;
                                    break;
                                }
                            }

                            if ptr.is_some() {
                                matched += 1;
                            }
                        };
                    }

                    if c_kind == CharKind::Ascii {
                        parse_char_type!(c_char);
                    } else {
                        parse_char_type!(wchar_t);
                    }
                }

                '[' => {
                    c = next_char(&mut format)?;

                    let mut matches = Vec::new();
                    let invert = if c == '^' {
                        c = next_char(&mut format)?;
                        true
                    } else {
                        false
                    };

                    let mut prev: u32;
                    loop {
                        matches.push(c);
                        prev = c.into();
                        c = next_char(&mut format)?;
                        if c == '-' {
                            if prev as u8 == b']' {
                                continue;
                            }
                            c = next_char(&mut format)?;
                            if c == ']' {
                                matches.push('-');
                                break;
                            }
                            prev += 1;
                            while prev < c.into() {
                                matches.push(char::try_from(prev).map_err(|_| -1)?);
                                prev += 1;
                            }
                        } else if c == ']' {
                            break;
                        }
                    }

                    let mut ptr: Option<*mut c_char> = if ignore {
                        None
                    } else {
                        Some(unsafe { ap.arg() })
                    };

                    // While we haven't used up all the width, and it matches
                    let mut data_stored = false;
                    while width.map(|w| w > 0).unwrap_or(true)
                        && invert != matches.contains(&character)
                    {
                        if let Some(ref mut ptr) = ptr {
                            unsafe { **ptr = character as c_char };
                            *ptr = unsafe { ptr.offset(1) };
                            data_stored = true;
                        }
                        // Decrease the width, and read a new character unless the width is 0
                        width = width.map(|w| w - 1);
                        if width.map(|w| w > 0).unwrap_or(true) && !read!() {
                            // Reading a new character has failed, return after
                            // actually marking this as matched
                            eof = true;
                            break;
                        }
                    }

                    if data_stored {
                        unsafe { *ptr.unwrap() = 0 };
                        matched += 1;
                    }
                }
                'n' => {
                    if !ignore {
                        unsafe { *ap.arg::<*mut c_int>() = count as c_int };
                    }
                }
                _ => return Err(-1),
            }

            if eof {
                return Ok(matched);
            }

            if width != Some(0) && c != 'n' {
                // It didn't hit the width, so an extra character was read and matched.
                // But this character did not match so let's reuse it.
                skip_read = true;
            }
        }
    }
    Ok(matched)
}

pub unsafe fn scanf<T: Kind>(r: Reader<T>, format: Reader<T>, ap: va_list) -> c_int {
    match unsafe { inner_scanf(r, format, ap) } {
        Ok(n) => n,
        Err(n) => n,
    }
}
