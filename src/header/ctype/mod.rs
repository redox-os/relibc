//! ctype implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/ctype.h.html

use crate::platform::types::*;

#[no_mangle]
pub extern "C" fn isalnum(c: c_int) -> c_int {
    c_int::from(isdigit(c) != 0 || isalpha(c) != 0)
}

#[no_mangle]
pub extern "C" fn isalpha(c: c_int) -> c_int {
    c_int::from(islower(c) != 0 || isupper(c) != 0)
}

#[no_mangle]
pub extern "C" fn isascii(c: c_int) -> c_int {
    c_int::from((c & !0x7f) == 0)
}

#[no_mangle]
pub extern "C" fn isblank(c: c_int) -> c_int {
    c_int::from(c == c_int::from(b' ') || c == c_int::from(b'\t'))
}

#[no_mangle]
pub extern "C" fn iscntrl(c: c_int) -> c_int {
    c_int::from((c >= 0x00 && c <= 0x1f) || c == 0x7f)
}

#[no_mangle]
pub extern "C" fn isdigit(c: c_int) -> c_int {
    c_int::from(c >= c_int::from(b'0') && c <= c_int::from(b'9'))
}

#[no_mangle]
pub extern "C" fn isgraph(c: c_int) -> c_int {
    c_int::from(c >= 0x21 && c <= 0x7e)
}

#[no_mangle]
pub extern "C" fn islower(c: c_int) -> c_int {
    c_int::from(c >= c_int::from(b'a') && c <= c_int::from(b'z'))
}

#[no_mangle]
pub extern "C" fn isprint(c: c_int) -> c_int {
    c_int::from(c >= 0x20 && c < 0x7f)
}

#[no_mangle]
pub extern "C" fn ispunct(c: c_int) -> c_int {
    c_int::from(
        (c >= c_int::from(b'!') && c <= c_int::from(b'/'))
            || (c >= c_int::from(b':') && c <= c_int::from(b'@'))
            || (c >= c_int::from(b'[') && c <= c_int::from(b'`'))
            || (c >= c_int::from(b'{') && c <= c_int::from(b'~')),
    )
}

#[no_mangle]
pub extern "C" fn isspace(c: c_int) -> c_int {
    c_int::from(
        c == c_int::from(b' ')
            || c == c_int::from(b'\t')
            || c == c_int::from(b'\n')
            || c == c_int::from(b'\r')
            || c == 0x0b
            || c == 0x0c,
    )
}

#[no_mangle]
pub extern "C" fn isupper(c: c_int) -> c_int {
    c_int::from(c >= c_int::from(b'A') && c <= c_int::from(b'Z'))
}

#[no_mangle]
pub extern "C" fn isxdigit(c: c_int) -> c_int {
    c_int::from(isdigit(c) != 0 || (c | 32 >= c_int::from(b'a') && c | 32 <= c_int::from(b'f')))
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
        c | 0x20
    } else {
        c
    }
}

#[no_mangle]
pub extern "C" fn toupper(c: c_int) -> c_int {
    if islower(c) != 0 {
        c & !0x20
    } else {
        c
    }
}
