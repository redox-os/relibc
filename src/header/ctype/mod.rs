//! `ctype.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/ctype.h.html>.

// TODO: *_l functions

use crate::platform::types::c_int;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isalnum.html>.
#[unsafe(no_mangle)]
pub extern "C" fn isalnum(c: c_int) -> c_int {
    c_int::from(isdigit(c) != 0 || isalpha(c) != 0)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isalpha.html>.
#[unsafe(no_mangle)]
pub extern "C" fn isalpha(c: c_int) -> c_int {
    c_int::from(islower(c) != 0 || isupper(c) != 0)
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/isascii.html>.
///
/// The `isascii()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 7, and removed in Issue 8.
#[deprecated]
#[unsafe(no_mangle)]
pub extern "C" fn isascii(c: c_int) -> c_int {
    c_int::from((c & !0x7f) == 0)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isblank.html>.
#[unsafe(no_mangle)]
pub extern "C" fn isblank(c: c_int) -> c_int {
    c_int::from(c == c_int::from(b' ') || c == c_int::from(b'\t'))
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/iscntrl.html>.
#[unsafe(no_mangle)]
pub extern "C" fn iscntrl(c: c_int) -> c_int {
    c_int::from((c >= 0x00 && c <= 0x1f) || c == 0x7f)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isdigit.html>.
#[unsafe(no_mangle)]
pub extern "C" fn isdigit(c: c_int) -> c_int {
    c_int::from(c >= c_int::from(b'0') && c <= c_int::from(b'9'))
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isgraph.html>.
#[unsafe(no_mangle)]
pub extern "C" fn isgraph(c: c_int) -> c_int {
    c_int::from(c >= 0x21 && c <= 0x7e)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/islower.html>.
#[unsafe(no_mangle)]
pub extern "C" fn islower(c: c_int) -> c_int {
    c_int::from(c >= c_int::from(b'a') && c <= c_int::from(b'z'))
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isprint.html>.
#[unsafe(no_mangle)]
pub extern "C" fn isprint(c: c_int) -> c_int {
    c_int::from(c >= 0x20 && c < 0x7f)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ispunct.html>.
#[unsafe(no_mangle)]
pub extern "C" fn ispunct(c: c_int) -> c_int {
    c_int::from(
        (c >= c_int::from(b'!') && c <= c_int::from(b'/'))
            || (c >= c_int::from(b':') && c <= c_int::from(b'@'))
            || (c >= c_int::from(b'[') && c <= c_int::from(b'`'))
            || (c >= c_int::from(b'{') && c <= c_int::from(b'~')),
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isspace.html>.
#[unsafe(no_mangle)]
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

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isupper.html>.
#[unsafe(no_mangle)]
pub extern "C" fn isupper(c: c_int) -> c_int {
    c_int::from(c >= c_int::from(b'A') && c <= c_int::from(b'Z'))
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isxdigit.html>.
#[unsafe(no_mangle)]
pub extern "C" fn isxdigit(c: c_int) -> c_int {
    c_int::from(isdigit(c) != 0 || (c | 32 >= c_int::from(b'a') && c | 32 <= c_int::from(b'f')))
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/toascii.html>.
///
/// The `toascii()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 7, and removed in Issue 8.
#[deprecated]
#[unsafe(no_mangle)]
pub extern "C" fn toascii(c: c_int) -> c_int {
    c & 0x7f
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tolower.html>.
#[unsafe(no_mangle)]
pub extern "C" fn tolower(c: c_int) -> c_int {
    if isupper(c) != 0 { c | 0x20 } else { c }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/toupper.html>.
#[unsafe(no_mangle)]
pub extern "C" fn toupper(c: c_int) -> c_int {
    if islower(c) != 0 { c & !0x20 } else { c }
}
