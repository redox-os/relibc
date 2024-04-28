use super::lookaheadreader::LookAheadReader;
use crate::platform::types::*;
use alloc::{string::String, vec::Vec};
use core::ffi::VaList as va_list;

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

#[derive(PartialEq, Eq)]
enum CharKind {
    Ascii,
    Wide,
}

/// Helper function for progressing a C string
unsafe fn next_char(string: &mut *const wchar_t) -> Result<char, c_int> {
    let c = **string as u32;
    *string = string.offset(1);
    if c == 0 {
        Err(-1)
    } else {
        char::from_u32(c).ok_or(-1)
    }
}

unsafe fn inner_scanf(
    mut r: LookAheadReader,
    mut format: *const wchar_t,
    mut ap: va_list,
) -> Result<c_int, c_int> {
    let mut matched = 0;
    let mut byte: char = '\0';
    let mut skip_read = false;
    let mut count = 0;

    macro_rules! read {
        () => {{
            match r.lookahead1() {
                Ok(None) => false,
                Ok(Some(b)) => {
                    byte = char::from_u32(b).ok_or(-1)?;
                    count += 1;
                    true
                }
                Err(x) => return Err(x),
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

    while *format != 0 {
        let mut c = next_char(&mut format)?;

        if c == ' ' {
            maybe_read!(noreset);

            while (byte as char).is_whitespace() {
                if !read!() {
                    return Ok(matched);
                }
            }

            skip_read = true;
        } else if c != '%' {
            maybe_read!();
            if c != byte as char {
                return Ok(matched);
            }
            r.commit();
        } else {
            c = next_char(&mut format)?;

            let mut ignore = false;
            if c == '*' {
                ignore = true;
                c = next_char(&mut format)?;
            }

            let mut width = String::new();
            while c >= '0' && c <= '9' {
                width.push(c as char);
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
                kind = match c {
                    'h' => {
                        if kind == IntKind::Short || kind == IntKind::Byte {
                            IntKind::Byte
                        } else {
                            IntKind::Short
                        }
                    }
                    'j' => IntKind::IntMax,
                    'l' => {
                        if kind == IntKind::Long || kind == IntKind::LongLong {
                            IntKind::LongLong
                        } else {
                            IntKind::Long
                        }
                    }
                    'q' | 'L' => IntKind::LongLong,
                    't' => IntKind::PtrDiff,
                    'z' => IntKind::Size,
                    // If kind is Long, means we found a 'l' before finding 'c' or 's'. In this
                    // case the format corresponds to a wide char/string
                    'c' | 's' if kind == IntKind::Long => {
                        c_kind = CharKind::Wide;
                        break;
                    }
                    _ => break,
                };

                c = next_char(&mut format)?;
            }

            if c != 'n' {
                maybe_read!(noreset);
            }
            match c {
                '%' => {
                    while (byte as char).is_whitespace() {
                        if !read!() {
                            return Ok(matched);
                        }
                    }

                    if byte != '%' {
                        return Err(matched);
                    } else if !read!() {
                        return Ok(matched);
                    }
                }

                'd' | 'i' | 'o' | 'u' | 'x' | 'X' | 'f' | 'e' | 'g' | 'E' | 'a' | 'p' => {
                    while (byte as char).is_whitespace() {
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
                        && ((byte >= '0' && byte <= '7')
                            || (radix >= 10 && (byte >= '8' && byte <= '9'))
                            || (float && !dot && byte == '.')
                            || (radix == 16
                                && ((byte >= 'a' && byte <= 'f') || (byte >= 'A' && byte <= 'F'))))
                    {
                        if auto
                            && n.is_empty()
                            && byte == '0'
                            && width.map(|w| w > 0).unwrap_or(true)
                        {
                            if !pointer {
                                radix = 8;
                            }
                            width = width.map(|w| w - 1);
                            if !read!() {
                                return Ok(matched);
                            }
                            if width.map(|w| w > 0).unwrap_or(true) && (byte == 'x' || byte == 'X')
                            {
                                radix = 16;
                                width = width.map(|w| w - 1);
                                if width.map(|w| w > 0).unwrap_or(true) && !read!() {
                                    return Ok(matched);
                                }
                            }
                            continue;
                        }
                        if byte == '.' {
                            // Don't allow another dot
                            dot = true;
                        }
                        n.push(byte as char);
                        r.commit();
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
                                *ap.arg::<*mut $type>() = n;
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
                                *ap.arg::<*mut $final>() = n as $final;
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
                            while (byte as char).is_whitespace() {
                                if !read!() {
                                    return Ok(matched);
                                }
                            }

                            let mut ptr: Option<*mut $type> =
                                if ignore { None } else { Some(ap.arg()) };

                            while width.map(|w| w > 0).unwrap_or(true)
                                && !(byte as char).is_whitespace()
                            {
                                if let Some(ref mut ptr) = ptr {
                                    **ptr = byte as $type;
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
                                r.commit();
                            }
                        };
                    }

                    if c_kind == CharKind::Ascii {
                        parse_string_type!(c_char);
                    } else {
                        parse_string_type!(wchar_t);
                    }
                }

                'c' => {
                    macro_rules! parse_char_type {
                        ($type:ident) => {
                            let ptr: Option<*mut $type> =
                                if ignore { None } else { Some(ap.arg()) };

                            for i in 0..width.unwrap_or(1) {
                                if let Some(ptr) = ptr {
                                    *ptr.add(i) = byte as $type;
                                }
                                width = width.map(|w| w - 1);
                                if width.map(|w| w > 0).unwrap_or(true) && !read!() {
                                    eof = true;
                                    break;
                                }
                            }

                            if ptr.is_some() {
                                matched += 1;
                                r.commit();
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

                    let mut prev;
                    loop {
                        matches.push(c);
                        prev = c;
                        c = next_char(&mut format)?;
                        if c == '-' {
                            if prev == ']' {
                                continue;
                            }
                            c = next_char(&mut format)?;
                            if c == ']' {
                                matches.push('-');
                                break;
                            }
                            prev = char::from_u32_unchecked(prev as u32 + 1);
                            while (prev as u32) < (c as u32) {
                                matches.push(prev);
                                prev = char::from_u32_unchecked(prev as u32 + 1);
                            }
                        } else if c == ']' {
                            break;
                        }
                    }

                    let mut ptr: Option<*mut c_char> = if ignore { None } else { Some(ap.arg()) };

                    // While we haven't used up all the width, and it matches
                    let mut data_stored = false;
                    while width.map(|w| w > 0).unwrap_or(true) && !invert == matches.contains(&byte)
                    {
                        if let Some(ref mut ptr) = ptr {
                            **ptr = byte as c_char;
                            *ptr = ptr.offset(1);
                            data_stored = true;
                        }
                        r.commit();
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
                        *ptr.unwrap() = 0;
                        matched += 1;
                    }
                }
                'n' => {
                    if !ignore {
                        *ap.arg::<*mut c_int>() = count as c_int;
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

pub unsafe fn scanf(r: LookAheadReader, format: *const wchar_t, ap: va_list) -> c_int {
    match inner_scanf(r, format, ap) {
        Ok(n) => n,
        Err(n) => n,
    }
}
