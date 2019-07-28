//! fnmatch implementation

use alloc::{borrow::Cow, vec::Vec};
use core::slice;

use crate::platform::types::*;
use posix_regex::{
    compile::{Collation, Range, Token},
    PosixRegex,
};

const ONCE: Range = Range(1, Some(1));

pub const FNM_NOMATCH: c_int = 1;

pub const FNM_NOESCAPE: c_int = 1;
pub const FNM_PATHNAME: c_int = 2;
pub const FNM_PERIOD: c_int = 4;
pub const FNM_CASEFOLD: c_int = 8;
// TODO: FNM_EXTMATCH

unsafe fn tokenize(mut pattern: *const u8, flags: c_int) -> Vec<(Token, Range)> {
    fn any(leading: bool, flags: c_int) -> Token {
        let mut list = Vec::new();
        if flags & FNM_PATHNAME == FNM_PATHNAME {
            list.push(Collation::Char(b'/'))
        }
        if leading && flags & FNM_PERIOD == FNM_PERIOD {
            list.push(Collation::Char(b'.'))
        }
        Token::OneOf { invert: true, list }
    }
    fn can_push(leading: bool, flags: c_int, c: u8) -> bool {
        (c != b'/' || flags & FNM_PATHNAME != FNM_PATHNAME)
            && (c != b'.' || !leading || flags & FNM_PERIOD != FNM_PERIOD)
    }
    fn is_leading(flags: c_int, c: u8) -> bool {
        c == b'/' && flags & FNM_PATHNAME == FNM_PATHNAME
    }

    let mut tokens = Vec::new();
    let mut leading = true;

    while *pattern != 0 {
        let was_leading = leading;
        leading = false;

        let c = *pattern;
        pattern = pattern.offset(1);

        tokens.push(match c {
            b'\\' if flags & FNM_NOESCAPE == FNM_NOESCAPE => {
                let c = *pattern;
                if c == 0 {
                    // Trailing backslash. Maybe error here?
                    break;
                }
                pattern = pattern.offset(1);
                leading = is_leading(flags, c);
                (Token::Char(c), ONCE)
            }
            b'?' => (any(was_leading, flags), ONCE),
            b'*' => (any(was_leading, flags), Range(0, None)),
            b'[' => {
                let mut list: Vec<Collation> = Vec::new();
                let invert = if *pattern == b'!' {
                    pattern = pattern.offset(1);
                    true
                } else {
                    false
                };

                loop {
                    let mut c = *pattern;
                    if c == 0 {
                        break;
                    }
                    pattern = pattern.offset(1);
                    match c {
                        b']' => break,
                        b'\\' => {
                            c = *pattern;
                            pattern = pattern.offset(1);
                            if c == 0 {
                                // Trailing backslash. Maybe error?
                                break;
                            }
                        }
                        _ => (),
                    }
                    if *pattern == b'-' && *pattern.offset(1) != 0 {
                        let end = *pattern.offset(1);
                        pattern = pattern.offset(2);
                        for c in c..=end {
                            if can_push(was_leading, flags, c) {
                                list.push(Collation::Char(c));
                            }
                        }
                    } else if can_push(was_leading, flags, c) {
                        list.push(Collation::Char(c));
                    }
                }
                // Otherwise, there was no closing ]. Maybe error?

                (Token::OneOf { invert, list }, ONCE)
            }
            c => {
                leading = is_leading(flags, c);
                (Token::Char(c), ONCE)
            }
        })
    }
    tokens
}

#[no_mangle]
#[linkage = "weak"] // often redefined in GNU programs
pub unsafe extern "C" fn fnmatch(
    pattern: *const c_char,
    input: *const c_char,
    flags: c_int,
) -> c_int {
    let mut len = 0;
    while *input.offset(len) != 0 {
        len += 1;
    }
    let input = slice::from_raw_parts(input as *const u8, len as usize);

    let mut tokens = tokenize(pattern as *const u8, flags);
    tokens.push((Token::End, ONCE));

    if PosixRegex::new(Cow::Owned(vec![tokens]))
        .case_insensitive(flags & FNM_CASEFOLD == FNM_CASEFOLD)
        .matches_exact(input)
        .is_some()
    {
        0
    } else {
        FNM_NOMATCH
    }
}
