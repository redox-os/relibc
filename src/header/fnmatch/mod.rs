//! fnmatch implementation

use alloc::vec::Vec;

use platform::types::*;

pub const FNM_NOMATCH: c_int = 1;

pub const FNM_NOESCAPE: c_int = 1;
pub const FNM_PATHNAME: c_int = 2;
pub const FNM_PERIOD: c_int = 4;
pub const FNM_CASEFOLD: c_int = 8;

#[derive(Debug)]
enum Token {
    Any,
    Char(u8),
    Match(bool, Vec<u8>),
    Wildcard,
    // TODO: FNM_EXTMATCH, which is basically a whole another custom regex
    // format that's ambigious and ugh. The C standard library is really bloaty
    // and I sure hope we can get away with delaying this as long as possible.
    // If you need to implement this, you can contact jD91mZM2 for assistance
    // in reading this code in case it's ugly.
}

unsafe fn next_token(pattern: &mut *const c_char, flags: c_int) -> Option<Token> {
    let c = **pattern as u8;
    if c == 0 {
        return None;
    }
    *pattern = pattern.offset(1);
    Some(match c {
        b'\\' if flags & FNM_NOESCAPE == FNM_NOESCAPE => {
            let c = **pattern as u8;
            if c == 0 {
                // Trailing backslash. Maybe error here?
                return None;
            }
            *pattern = pattern.offset(1);
            Token::Char(c)
        }
        b'?' => Token::Any,
        b'*' => Token::Wildcard,
        b'[' => {
            let mut matches = Vec::new();
            let invert = if **pattern as u8 == b'!' {
                *pattern = pattern.offset(1);
                true
            } else {
                false
            };

            loop {
                let mut c = **pattern as u8;
                if c == 0 {
                    break;
                }
                *pattern = pattern.offset(1);
                match c {
                    b']' => break,
                    b'\\' => {
                        c = **pattern as u8;
                        *pattern = pattern.offset(1);
                        if c == 0 {
                            // Trailing backslash. Maybe error?
                            break;
                        }
                    }
                    _ => (),
                }
                if matches.len() >= 2 && matches[matches.len() - 1] == b'-' {
                    let len = matches.len();
                    let start = matches[len - 2];
                    matches.drain(len - 2..);
                    // Exclusive range because we'll push C later
                    for c in start..c {
                        matches.push(c);
                    }
                }
                matches.push(c);
            }
            // Otherwise, there was no closing ]. Maybe error?

            Token::Match(invert, matches)
        }
        c => Token::Char(c),
    })
}

#[no_mangle]
pub unsafe extern "C" fn fnmatch(
    mut pattern: *const c_char,
    mut input: *const c_char,
    flags: c_int,
) -> c_int {
    let pathname = flags & FNM_PATHNAME == FNM_PATHNAME;
    let casefold = flags & FNM_CASEFOLD == FNM_CASEFOLD;

    let mut leading = true;

    loop {
        if *input == 0 {
            return if *pattern == 0 { 0 } else { FNM_NOMATCH };
        }
        if leading && flags & FNM_PERIOD == FNM_PERIOD {
            if *input as u8 == b'.' && *pattern as u8 != b'.' {
                return FNM_NOMATCH;
            }
        }
        leading = false;
        match next_token(&mut pattern, flags) {
            Some(Token::Any) => {
                if pathname && *input as u8 == b'/' {
                    return FNM_NOMATCH;
                }
                input = input.offset(1);
            }
            Some(Token::Char(c)) => {
                let mut a = *input as u8;
                if casefold && a >= b'a' && a <= b'z' {
                    a -= b'a' - b'A';
                }
                let mut b = c;
                if casefold && b >= b'a' && b <= b'z' {
                    b -= b'a' - b'A';
                }
                if a != b {
                    return FNM_NOMATCH;
                }
                if pathname && a == b'/' {
                    leading = true;
                }
                input = input.offset(1);
            }
            Some(Token::Match(invert, matches)) => {
                if (pathname && *input as u8 == b'/') || matches.contains(&(*input as u8)) == invert
                {
                    // Found it, but it's inverted! Or vise versa.
                    return FNM_NOMATCH;
                }
                input = input.offset(1);
            }
            Some(Token::Wildcard) => {
                loop {
                    let c = *input as u8;
                    if c == 0 {
                        return if *pattern == 0 { 0 } else { FNM_NOMATCH };
                    }

                    let ret = fnmatch(pattern, input, flags);
                    if ret == FNM_NOMATCH {
                        input = input.offset(1);
                    } else {
                        // Either an error or a match. Forward the return.
                        return ret;
                    }

                    if pathname && c == b'/' {
                        // End of segment, no match yet
                        return FNM_NOMATCH;
                    }
                }
            }
            None => return FNM_NOMATCH, // Pattern ended but there's still some input
        }
    }
}
