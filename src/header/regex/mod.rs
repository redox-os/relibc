//! regex.h implementation, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/regex.h.html

use crate::{header::string::strlen, platform::types::*};
use alloc::{borrow::Cow, vec::Vec};
use core::{mem, ptr, slice};
use posix_regex::{
    compile::{Error as CompileError, Range, Token},
    PosixRegex, PosixRegexBuilder,
};

pub type regoff_t = size_t;

#[repr(C)]
pub struct regex_t {
    // Can't be a normal Vec<T> because then the struct size won't be known
    // from C.
    ptr: *mut c_void,
    length: size_t,
    capacity: size_t,

    cflags: c_int,
    re_nsub: size_t,
}
#[repr(C)]
pub struct regmatch_t {
    rm_so: regoff_t,
    rm_eo: regoff_t,
}

pub const REG_EXTENDED: c_int = 1;
pub const REG_ICASE: c_int = 2;
pub const REG_NOSUB: c_int = 4;
pub const REG_NEWLINE: c_int = 8;
pub const REG_NOTBOL: c_int = 16;
pub const REG_NOTEOL: c_int = 32;

pub const REG_NOMATCH: c_int = 1;
pub const REG_BADPAT: c_int = 2;
pub const REG_ECOLLATE: c_int = 3;
pub const REG_ECTYPE: c_int = 4;
pub const REG_EESCAPE: c_int = 5;
pub const REG_ESUBREG: c_int = 6;
pub const REG_EBRACK: c_int = 7;
pub const REG_ENOSYS: c_int = 8;
pub const REG_EPAREN: c_int = 9;
pub const REG_EBRACE: c_int = 10;
pub const REG_BADBR: c_int = 11;
pub const REG_ERANGE: c_int = 12;
pub const REG_ESPACE: c_int = 13;
pub const REG_BADRPT: c_int = 14;

#[no_mangle]
#[linkage = "weak"] // redefined in GIT
pub unsafe extern "C" fn regcomp(out: *mut regex_t, pat: *const c_char, cflags: c_int) -> c_int {
    if cflags & REG_EXTENDED == REG_EXTENDED {
        return REG_ENOSYS;
    }

    let pat = slice::from_raw_parts(pat as *const u8, strlen(pat));
    let res = PosixRegexBuilder::new(pat)
        .with_default_classes()
        .compile_tokens();

    match res {
        Ok(mut branches) => {
            let re_nsub = PosixRegex::new(Cow::Borrowed(&branches)).count_groups();
            *out = regex_t {
                ptr: branches.as_mut_ptr() as *mut c_void,
                length: branches.len(),
                capacity: branches.capacity(),

                cflags,
                re_nsub,
            };
            mem::forget(branches);
            0
        }
        Err(CompileError::EmptyRepetition)
        | Err(CompileError::IntegerOverflow)
        | Err(CompileError::IllegalRange) => REG_BADBR,
        Err(CompileError::UnclosedRepetition) => REG_EBRACE,
        Err(CompileError::LeadingRepetition) => REG_BADRPT,
        Err(CompileError::UnknownCollation) => REG_ECOLLATE,
        Err(CompileError::UnknownClass(_)) => REG_ECTYPE,
        Err(_) => REG_BADPAT,
    }
}

#[no_mangle]
#[linkage = "weak"] // redefined in GIT
pub unsafe extern "C" fn regfree(regex: *mut regex_t) {
    Vec::from_raw_parts(
        (*regex).ptr as *mut Vec<(Token, Range)>,
        (*regex).length,
        (*regex).capacity,
    );
}

#[no_mangle]
#[linkage = "weak"] // redefined in GIT
pub unsafe extern "C" fn regexec(
    regex: *const regex_t,
    input: *const c_char,
    nmatch: size_t,
    pmatch: *mut regmatch_t,
    eflags: c_int,
) -> c_int {
    if eflags & REG_EXTENDED == REG_EXTENDED {
        return REG_ENOSYS;
    }

    let regex = &*regex;

    // Allow specifying a compiler argument to the executor and vise versa
    // because why not?
    let flags = regex.cflags | eflags;

    let input = slice::from_raw_parts(input as *const u8, strlen(input));
    let branches = slice::from_raw_parts(regex.ptr as *const Vec<(Token, Range)>, regex.length);

    let matches = PosixRegex::new(Cow::Borrowed(&branches))
        .case_insensitive(flags & REG_ICASE == REG_ICASE)
        .newline(flags & REG_NEWLINE == REG_NEWLINE)
        .no_start(flags & REG_NOTBOL == REG_NOTBOL)
        .no_end(flags & REG_NOTEOL == REG_NOTEOL)
        .matches(input, Some(1));

    if !matches.is_empty() && eflags & REG_NOSUB != REG_NOSUB && !pmatch.is_null() && nmatch > 0 {
        let first = &matches[0];

        for i in 0..nmatch {
            let (start, end) = first.get(i).and_then(|&range| range).unwrap_or((!0, !0));
            *pmatch.add(i) = regmatch_t {
                rm_so: start,
                rm_eo: end,
            };
        }
    }

    if matches.is_empty() {
        REG_NOMATCH
    } else {
        0
    }
}

#[no_mangle]
#[linkage = "weak"] // redefined in GIT
pub extern "C" fn regerror(
    code: c_int,
    _regex: *const regex_t,
    out: *mut c_char,
    max: size_t,
) -> size_t {
    let string = match code {
        0 => "No error\0",
        REG_NOMATCH => "No match\0",
        REG_BADPAT => "Invalid regexp\0",
        REG_ECOLLATE => "Unknown collating element\0",
        REG_ECTYPE => "Unknown character class name\0",
        REG_EESCAPE => "Trailing backslash\0",
        REG_ESUBREG => "Invalid back reference\0",
        REG_EBRACK => "Missing ']'\0",
        REG_ENOSYS => "Unsupported operation\0",
        REG_EPAREN => "Missing ')'\0",
        REG_EBRACE => "Missing '}'\0",
        REG_BADBR => "Invalid contents of {}\0",
        REG_ERANGE => "Invalid character range\0",
        REG_ESPACE => "Out of memory\0",
        REG_BADRPT => "Repetition not preceded by valid expression\0",
        _ => "Unknown error\0",
    };

    unsafe {
        ptr::copy_nonoverlapping(
            string.as_ptr(),
            out as *mut u8,
            string.len().min(max as usize),
        );
    }

    string.len()
}
