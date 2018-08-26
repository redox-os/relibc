//! ctype implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/ctype.h.html

use platform;
use platform::types::*;

#[no_mangle]
pub extern "C" fn isalnum(c: c_int) -> c_int {
    (isdigit(c) != 0 || isalpha(c) != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isalpha(c: c_int) -> c_int {
    (islower(c) != 0 || isupper(c) != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isascii(c: c_int) -> c_int {
    ((c & !0x7f) == 0) as c_int
}

#[no_mangle]
pub extern "C" fn isblank(c: c_int) -> c_int {
    (c == ' ' as c_int || c == '\t' as c_int) as c_int
}

#[no_mangle]
pub extern "C" fn iscntrl(c: c_int) -> c_int {
    ((c as c_uint) < 0x20 || c == 0x7f) as c_int
}

#[no_mangle]
pub extern "C" fn isdigit(c: c_int) -> c_int {
    (((c - 0x30) as c_uint) < 10) as c_int
}

#[no_mangle]
pub extern "C" fn isgraph(c: c_int) -> c_int {
    (((c - 0x21) as c_uint) < 0x5e) as c_int
}

#[no_mangle]
pub extern "C" fn islower(c: c_int) -> c_int {
    (((c - 0x61) as c_uint) < 26) as c_int
}

#[no_mangle]
pub extern "C" fn isprint(c: c_int) -> c_int {
    (((c - 0x20) as c_uint) < 0x5f) as c_int
}

#[no_mangle]
pub extern "C" fn ispunct(c: c_int) -> c_int {
    (isgraph(c) != 0 && !isalnum(c) != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isspace(c: c_int) -> c_int {
    (c == 0x20) as c_int
}

#[no_mangle]
pub extern "C" fn isupper(c: c_int) -> c_int {
    (((c - 0x41) as c_uint) < 26) as c_int
}

#[no_mangle]
pub extern "C" fn isxdigit(c: c_int) -> c_int {
    (isdigit(c) != 0 || ((c as c_int) | 32) - ('a' as c_int) < 6) as c_int
}

#[no_mangle]
/// The comment in musl:
/// "nonsense function that should NEVER be used!"
pub extern "C" fn toascii(c: c_int) -> c_int {
    c & 0x7f
}

#[no_mangle]
pub extern "C" fn tolower(c: c_int) -> c_int {
    if isupper(c) != 0 {
        c + 0x20
    } else {
        c
    }
}

#[no_mangle]
pub extern "C" fn toupper(c: c_int) -> c_int {
    if islower(c) != 0 {
        c - 0x20
    } else {
        c
    }
}
