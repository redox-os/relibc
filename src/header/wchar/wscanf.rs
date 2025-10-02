use super::reader::Reader;
use crate::platform::types::*;
use alloc::{string::String, vec::Vec};
use core::{ffi::VaList as va_list, iter::Peekable};

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

unsafe fn next_char(lar: &mut Peekable<Reader<'_>>) -> Result<wint_t, c_int> {
    if let Some(c) = lar.next().transpose()? {
        Ok(c)
    } else {
        Ok(0)
    }
}

macro_rules! wc_as_char {
    ($c:ident) => {
        char::try_from($c).map_err(|_| -1)?
    };
}

unsafe fn inner_scanf(mut r: Reader, mut format: Reader, mut ap: va_list) -> Result<c_int, c_int> {
    let mut matched = 0;
    let mut wchar = 0;
    let mut skip_read = false;
    let mut count = 0;
    let mut format = format.peekable();

    macro_rules! read {
        () => {{
            match r.next() {
                None => false,
                Some(Ok(b)) => {
                    wchar = b;
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

        if c as u8 == b' ' {
            maybe_read!(noreset);

            while (wc_as_char!(wchar)).is_whitespace() {
                if !read!() {
                    return Ok(matched);
                }
            }

            skip_read = true;
        } else if c as u8 != b'%' {
            maybe_read!();
            if c != wchar {
                return Ok(matched);
            }
        } else {
            c = next_char(&mut format)?;

            let mut ignore = false;
            if c as u8 == b'*' {
                ignore = true;
                c = next_char(&mut format)?;
            }

            let mut width = String::new();
            while c as u8 >= b'0' && c as u8 <= b'9' {
                width.push(wc_as_char!(c));
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
                match c as u8 {
                    b'h' => {
                        if kind == IntKind::Short || kind == IntKind::Byte {
                            kind = IntKind::Byte;
                        } else {
                            kind = IntKind::Short;
                        }
                    }
                    b'j' => kind = IntKind::IntMax,
                    b'l' => {
                        if kind == IntKind::Long || kind == IntKind::LongLong {
                            kind = IntKind::LongLong;
                        } else {
                            kind = IntKind::Long;
                        }
                    }
                    b'q' | b'L' => kind = IntKind::LongLong,
                    b't' => kind = IntKind::PtrDiff,
                    b'z' => kind = IntKind::Size,
                    // If kind is Long, means we found a 'l' before finding 'c' or 's'. In this
                    // case the format corresponds to a wide char/string
                    b'c' | b's' if kind == IntKind::Long => {
                        c_kind = CharKind::Wide;
                        break;
                    }
                    _ => break,
                }

                c = next_char(&mut format)?;
            }

            if c as u8 != b'n' {
                maybe_read!(noreset);
            }
            match c as u8 {
                b'%' => {
                    while (wc_as_char!(wchar)).is_whitespace() {
                        if !read!() {
                            return Ok(matched);
                        }
                    }

                    if wchar as u8 != b'%' {
                        return Err(matched);
                    } else if !read!() {
                        return Ok(matched);
                    }
                }

                b'd' | b'i' | b'o' | b'u' | b'x' | b'X' | b'f' | b'e' | b'g' | b'E' | b'a'
                | b'p' => {
                    while (wc_as_char!(wchar)).is_whitespace() {
                        if !read!() {
                            return Ok(matched);
                        }
                    }

                    let pointer = c as u8 == b'p';
                    // Pointers aren't automatic, but we do want to parse "0x"
                    let auto = c as u8 == b'i' || pointer;
                    let float = c as u8 == b'f'
                        || c as u8 == b'e'
                        || c as u8 == b'g'
                        || c as u8 == b'E'
                        || c as u8 == b'a';

                    let mut radix = match c as u8 {
                        b'o' => 8,
                        b'x' | b'X' | b'p' => 16,
                        _ => 10,
                    };

                    let mut n = String::new();
                    let mut dot = false;

                    while width.map(|w| w > 0).unwrap_or(true)
                        && ((wchar as u8 >= b'0' && wchar as u8 <= b'7')
                            || (radix >= 10 && (wchar as u8 >= b'8' && wchar as u8 <= b'9'))
                            || (float && !dot && wchar as u8 == b'.')
                            || (radix == 16
                                && ((wchar as u8 >= b'a' && wchar as u8 <= b'f')
                                    || (wchar as u8 >= b'A' && wchar as u8 <= b'F'))))
                    {
                        if auto
                            && n.is_empty()
                            && wchar as u8 == b'0'
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
                                && (wchar as u8 == b'x' || wchar as u8 == b'X')
                            {
                                radix = 16;
                                width = width.map(|w| w - 1);
                                if width.map(|w| w > 0).unwrap_or(true) && !read!() {
                                    return Ok(matched);
                                }
                            }
                            continue;
                        }
                        if wchar as u8 == b'.' {
                            // Don't allow another dot
                            dot = true;
                        }
                        n.push(wc_as_char!(wchar));
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
                    } else if c as u8 == b'p' {
                        parse_type!(size_t, *mut c_void);
                    } else {
                        let unsigned = c as u8 == b'o'
                            || c as u8 == b'u'
                            || c as u8 == b'x'
                            || c as u8 == b'X';

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

                b's' => {
                    macro_rules! parse_string_type {
                        ($type:ident) => {
                            while (wc_as_char!(wchar)).is_whitespace() {
                                if !read!() {
                                    return Ok(matched);
                                }
                            }

                            let mut ptr: Option<*mut $type> =
                                if ignore { None } else { Some(ap.arg()) };

                            while width.map(|w| w > 0).unwrap_or(true)
                                && !(wc_as_char!(wchar)).is_whitespace()
                            {
                                if let Some(ref mut ptr) = ptr {
                                    **ptr = wchar as $type;
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
                        parse_string_type!(c_char);
                    } else {
                        parse_string_type!(wchar_t);
                    }
                }

                b'c' => {
                    macro_rules! parse_char_type {
                        ($type:ident) => {
                            let ptr: Option<*mut $type> =
                                if ignore { None } else { Some(ap.arg()) };

                            for i in 0..width.unwrap_or(1) {
                                if let Some(ptr) = ptr {
                                    *ptr.add(i) = wchar as $type;
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

                b'[' => {
                    c = next_char(&mut format)?;

                    let mut matches = Vec::new();
                    let invert = if c as u8 == b'^' {
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
                        if c as u8 == b'-' {
                            if prev as u8 == b']' {
                                continue;
                            }
                            c = next_char(&mut format)?;
                            if c as u8 == b']' {
                                matches.push('-' as wint_t);
                                break;
                            }
                            prev += 1;
                            while prev < c {
                                matches.push(prev);
                                prev += 1;
                            }
                        } else if c as u8 == b']' {
                            break;
                        }
                    }

                    let mut ptr: Option<*mut c_char> = if ignore { None } else { Some(ap.arg()) };

                    // While we haven't used up all the width, and it matches
                    let mut data_stored = false;
                    while width.map(|w| w > 0).unwrap_or(true)
                        && !invert == matches.contains(&wchar)
                    {
                        if let Some(ref mut ptr) = ptr {
                            **ptr = wchar as c_char;
                            *ptr = ptr.offset(1);
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
                        *ptr.unwrap() = 0;
                        matched += 1;
                    }
                }
                b'n' => {
                    if !ignore {
                        *ap.arg::<*mut c_int>() = count as c_int;
                    }
                }
                _ => return Err(-1),
            }

            if eof {
                return Ok(matched);
            }

            if width != Some(0) && c as u8 != b'n' {
                // It didn't hit the width, so an extra character was read and matched.
                // But this character did not match so let's reuse it.
                skip_read = true;
            }
        }
    }
    Ok(matched)
}

pub unsafe fn scanf(r: Reader, format: Reader, ap: va_list) -> c_int {
    match inner_scanf(r, format, ap) {
        Ok(n) => n,
        Err(n) => n,
    }
}
