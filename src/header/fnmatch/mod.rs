//! fnmatch implementation

#![deny(unsafe_op_in_unsafe_fn)]

use alloc::{borrow::Cow, vec::Vec};
use core::slice;

use crate::platform::types::*;
use posix_regex::{
    compile::{Collation, Range, Token},
    tree::{Tree, TreeBuilder},
    PosixRegex,
};

const ONCE: Range = Range(1, Some(1));

pub const FNM_NOMATCH: c_int = 1;

pub const FNM_NOESCAPE: c_int = 1;
pub const FNM_PATHNAME: c_int = 2;
pub const FNM_PERIOD: c_int = 4;
pub const FNM_CASEFOLD: c_int = 8;
// TODO: FNM_EXTMATCH

unsafe fn tokenize(mut pattern: *const u8, flags: c_int) -> Tree {
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

    let mut leading = true;

    let mut builder = TreeBuilder::default();
    builder.start_internal(Token::Root, Range(1, Some(1)));
    builder.start_internal(Token::Alternative, Range(1, Some(1)));

    while unsafe { *pattern != 0 } {
        let was_leading = leading;
        leading = false;

        let c = unsafe { *pattern };
        pattern = unsafe { pattern.offset(1) };

        let (token, range) = match c {
            b'\\' if flags & FNM_NOESCAPE == FNM_NOESCAPE => {
                let c = unsafe { *pattern };
                if c == 0 {
                    // Trailing backslash. Maybe error here?
                    break;
                }
                pattern = unsafe { pattern.offset(1) };
                leading = is_leading(flags, c);
                (Token::Char(c), ONCE)
            }
            b'?' => (any(was_leading, flags), ONCE),
            b'*' => (any(was_leading, flags), Range(0, None)),
            b'[' => {
                let mut list: Vec<Collation> = Vec::new();
                let invert = if unsafe { *pattern == b'!' } {
                    pattern = unsafe { pattern.offset(1) };
                    true
                } else {
                    false
                };

                loop {
                    let mut c = unsafe { *pattern };
                    if c == 0 {
                        break;
                    }
                    pattern = unsafe { pattern.offset(1) };
                    match c {
                        b']' => break,
                        b'\\' => {
                            c = unsafe { *pattern };
                            pattern = unsafe { pattern.offset(1) };
                            if c == 0 {
                                // Trailing backslash. Maybe error?
                                break;
                            }
                        }
                        _ => (),
                    }
                    if unsafe { *pattern == b'-' && *pattern.offset(1) != 0 } {
                        let end = unsafe { *pattern.offset(1) };
                        pattern = unsafe { pattern.offset(2) };
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
        };
        builder.leaf(token, range);
    }
    builder.leaf(Token::End, ONCE);
    builder.finish_internal();
    builder.finish_internal();
    builder.finish()
}

#[no_mangle]
#[linkage = "weak"] // often redefined in GNU programs
pub unsafe extern "C" fn fnmatch(
    pattern: *const c_char,
    input: *const c_char,
    flags: c_int,
) -> c_int {
    let mut len = 0;
    while unsafe { *input.offset(len) != 0 } {
        len += 1;
    }
    let input = unsafe { slice::from_raw_parts(input as *const u8, len as usize) };

    let tokens = unsafe { tokenize(pattern as *const u8, flags) };

    if PosixRegex::new(Cow::Owned(tokens))
        .case_insensitive(flags & FNM_CASEFOLD == FNM_CASEFOLD)
        .matches_exact(input)
        .is_some()
    {
        0
    } else {
        FNM_NOMATCH
    }
}
