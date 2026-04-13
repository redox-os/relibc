use super::reader::Reader;
use crate::{c_str::Kind, platform::types::*};
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

fn next_char<T: Kind>(lar: &mut Peekable<Reader<'_, T>>) -> Result<T::Char, c_int> {
    if let Some(c) = lar.next().transpose()? {
        Ok(c)
    } else {
        Ok(T::Char::from(0u8))
    }
}

fn get_char_from_wint<T: Kind>(wc: wint_t) -> Result<T::Char, i32> {
    if let Some(wc_char) = T::chars_from_bytes(&wc.to_be_bytes())
        && wc_char.len() == 1
    {
        Ok(wc_char[0])
    } else {
        Err(-1)
    }
}

macro_rules! wc_as_char {
    ($c:ident) => {
        char::try_from($c.into()).map_err(|_| -1)?
    };
}

unsafe fn inner_scanf<T: Kind>(
    mut r: Reader<T>,
    format: Reader<T>,
    mut ap: va_list,
) -> Result<c_int, c_int> {
    let mut matched = 0;
    let mut wchar: T::Char = T::NUL;
    let mut wwchar: char = '\0';
    let mut skip_read = false;
    let mut count = 0;
    let mut format = format.peekable();

    macro_rules! read {
        () => {{
            match r.next() {
                None => false,
                Some(Ok(b)) => {
                    wchar = b;
                    wwchar = wc_as_char!(wchar);
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
        let mut cc: char = wc_as_char!(c);

        if cc == ' ' {
            maybe_read!(noreset);

            while (wwchar).is_whitespace() {
                if !read!() {
                    return Ok(matched);
                }
            }

            skip_read = true;
        } else if cc != '%' {
            maybe_read!();
            if c != wchar {
                return Ok(matched);
            }
        } else {
            c = next_char(&mut format)?;
            cc = wc_as_char!(c);

            let mut ignore = false;
            if cc == '*' {
                ignore = true;
                c = next_char(&mut format)?;
                cc = wc_as_char!(c);
            }

            let mut width = String::new();
            while cc.is_ascii_digit() {
                width.push(cc);
                c = next_char(&mut format)?;
                cc = wc_as_char!(c);
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
                match cc {
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
                    'c' | 's' if kind == IntKind::Long => {
                        c_kind = CharKind::Wide;
                        break;
                    }
                    _ => break,
                }

                c = next_char(&mut format)?;
                cc = wc_as_char!(c);
            }

            if cc != 'n' {
                maybe_read!(noreset);
            }
            match cc {
                '%' => {
                    while (wwchar).is_whitespace() {
                        if !read!() {
                            return Ok(matched);
                        }
                    }

                    if wwchar != '%' {
                        return Err(matched);
                    } else if !read!() {
                        return Ok(matched);
                    }
                }

                'd' | 'i' | 'o' | 'u' | 'x' | 'X' | 'f' | 'e' | 'g' | 'E' | 'a' | 'p' => {
                    while wwchar.is_whitespace() {
                        if !read!() {
                            return Ok(matched);
                        }
                    }

                    let pointer = cc == 'p';
                    // Pointers aren't automatic, but we do want to parse "0x"
                    let auto = cc == 'i' || pointer;
                    let float = cc == 'f' || cc == 'e' || cc == 'g' || cc == 'E' || cc == 'a';

                    let mut radix = match cc {
                        'o' => 8,
                        'x' | 'X' | 'p' => 16,
                        _ => 10,
                    };

                    let mut n = String::new();
                    let mut dot = false;

                    while width.map(|w| w > 0).unwrap_or(true)
                        && ((wwchar as u8 >= b'0' && wwchar as u8 <= b'7')
                            || (radix >= 10 && (wwchar as u8 >= b'8' && wwchar as u8 <= b'9'))
                            || (float && !dot && wwchar as u8 == b'.')
                            || (radix == 16
                                && ((wwchar as u8 >= b'a' && wwchar as u8 <= b'f')
                                    || (wwchar as u8 >= b'A' && wwchar as u8 <= b'F'))))
                    {
                        if auto
                            && n.is_empty()
                            && wwchar == '0'
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
                                && (wwchar == 'x' || wwchar == 'X')
                            {
                                radix = 16;
                                width = width.map(|w| w - 1);
                                if width.map(|w| w > 0).unwrap_or(true) && !read!() {
                                    return Ok(matched);
                                }
                            }
                            continue;
                        }
                        if wwchar == '.' {
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
                    } else if cc == 'p' {
                        parse_type!(size_t, *mut c_void);
                    } else {
                        let unsigned = cc == 'o' || cc == 'u' || cc == 'x' || cc == 'X';

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
                                    **ptr = wwchar as $type;
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
                                    unsafe { *ptr.add(i) = wwchar as $type };
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
                    cc = wc_as_char!(c);

                    let mut matches = Vec::new();
                    let invert = if cc == '^' {
                        c = next_char(&mut format)?;
                        cc = wc_as_char!(c);
                        true
                    } else {
                        false
                    };

                    let mut prev;
                    loop {
                        matches.push(c);
                        prev = c.into();
                        c = next_char(&mut format)?;
                        cc = wc_as_char!(c);
                        if cc == '-' {
                            if prev as u8 == b']' {
                                continue;
                            }
                            c = next_char(&mut format)?;
                            if cc == ']' {
                                matches.push(get_char_from_wint::<T>('-' as wint_t)?);
                                break;
                            }
                            prev += 1;
                            while prev < c.into() {
                                matches.push(get_char_from_wint::<T>(prev as wint_t)?);
                                prev += 1;
                            }
                        } else if cc == ']' {
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
                    while width.map(|w| w > 0).unwrap_or(true) && invert != matches.contains(&wchar)
                    {
                        if let Some(ref mut ptr) = ptr {
                            unsafe { **ptr = wwchar as c_char };
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

            if width != Some(0) && cc != 'n' {
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
