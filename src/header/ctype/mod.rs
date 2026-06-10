//! `ctype.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/ctype.h.html>.

use crate::{header::bits_locale_t::locale_t, platform::types::c_int};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isalnum.html>.
///
/// Tests whether `c` is a character of class alpha or digit in the current
/// locale.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn isalnum(c: c_int) -> c_int {
    c_int::from(isdigit(c) != 0 || isalpha(c) != 0)
}

// TODO make use of `loc`.
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isalnum_l.html>.
///
/// Tests whether `c` is a character of class alpha or digit in the locale
/// specified by `loc`.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn isalnum_l(c: c_int, _loc: locale_t) -> c_int {
    isalnum(c)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isalpha.html>.
///
/// Tests whether `c` is a character of class alpha in the current locale.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn isalpha(c: c_int) -> c_int {
    c_int::from(islower(c) != 0 || isupper(c) != 0)
}

// TODO make use of `loc`.
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isalpha_l.html>.
///
/// Tests whether `c` is a character of class alpha in the locale specified
/// by `loc`.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn isalpha_l(c: c_int, _loc: locale_t) -> c_int {
    isalpha(c)
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/isascii.html>.
///
/// Tests whether `c` is a 7bit US-ASCII character code.
///
/// Returns a non-zero value if true, 0 if false.
///
/// # Deprecated
/// The `isascii()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 7, and removed in Issue 8.
///
/// Not considered portable for localized applications.
#[deprecated]
#[unsafe(no_mangle)]
pub extern "C" fn isascii(c: c_int) -> c_int {
    c_int::from((c & !0x7f) == 0)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isblank.html>.
///
/// Tests whether `c` is a character of class blank in the current locale.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn isblank(c: c_int) -> c_int {
    c_int::from(c == c_int::from(b' ') || c == c_int::from(b'\t'))
}

// TODO make use of `loc`.
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isblank_l.html>.
///
/// Tests whether `c` is a character of class blank in the locale specified
/// by `loc`.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn isblank_l(c: c_int, _loc: locale_t) -> c_int {
    isblank(c)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/iscntrl.html>.
///
/// Tests whether `c` is a character of class cntrl in the current locale.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn iscntrl(c: c_int) -> c_int {
    c_int::from((0x00..=0x1f).contains(&c) || c == 0x7f)
}

// TODO make use of `loc`.
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/iscntrl_l.html>.
///
/// Tests whether `c` is a character of class cntrl in the locale specified
/// by `loc`.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn iscntrl_l(c: c_int, _loc: locale_t) -> c_int {
    iscntrl(c)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isdigit.html>.
///
/// Tests whether `c` is a character of class digit in the current locale.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn isdigit(c: c_int) -> c_int {
    c_int::from(c >= c_int::from(b'0') && c <= c_int::from(b'9'))
}

// TODO make use of `loc`.
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isdigit_l.html>.
///
/// Tests whether `c` is a character of class digit in the locale specified
/// by `loc`.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn isdigit_l(c: c_int, _loc: locale_t) -> c_int {
    isdigit(c)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isgraph.html>.
///
/// Tests whether `c` is a character of class graph (a character with a
/// visible representation) in the current locale.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn isgraph(c: c_int) -> c_int {
    c_int::from((0x21..=0x7e).contains(&c))
}

// TODO make use of `loc`.
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isgraph_l.html>.
///
/// Tests whether `c` is a character of class graph (a character with a
/// visible representation) in the locale specified by `loc`.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn isgraph_l(c: c_int, _loc: locale_t) -> c_int {
    isgraph(c)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/islower.html>.
///
/// Tests whether `c` is a character of class lower (a lowercase letter) in the
/// current locale.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn islower(c: c_int) -> c_int {
    c_int::from(c >= c_int::from(b'a') && c <= c_int::from(b'z'))
}

// TODO make use of `loc`.
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/islower_l.html>.
///
/// Tests whether `c` is a character of class lower (a lowercase letter) in the
/// locale specified by `loc`.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn islower_l(c: c_int, _loc: locale_t) -> c_int {
    islower(c)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isprint.html>.
///
/// Tests whether `c` is a character of class print (a printable character) in
/// the current locale.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn isprint(c: c_int) -> c_int {
    c_int::from((0x20..0x7f).contains(&c))
}

// TODO make use of `loc`.
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isprint_l.html>.
///
/// Tests whether `c` is a character of class print (a printable character) in
/// the locale specified by `loc`.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn isprint_l(c: c_int, _loc: locale_t) -> c_int {
    isprint(c)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ispunct.html>.
///
/// Tests whether `c` is a character of class punct (a punctuation character)
/// in the current locale.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn ispunct(c: c_int) -> c_int {
    c_int::from(
        (c >= c_int::from(b'!') && c <= c_int::from(b'/'))
            || (c >= c_int::from(b':') && c <= c_int::from(b'@'))
            || (c >= c_int::from(b'[') && c <= c_int::from(b'`'))
            || (c >= c_int::from(b'{') && c <= c_int::from(b'~')),
    )
}

// TODO make use of `loc`.
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ispunct_l.html>.
///
/// Tests whether `c` is a character of class punct (a punctuation character)
/// in the locale specified by `loc`.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn ispunct_l(c: c_int, _loc: locale_t) -> c_int {
    ispunct(c)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isspace.html>.
///
/// Tests whether `c` is a character of class space (a white-space character)
/// in the current locale.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
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

// TODO make use of `loc`.
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isspace_l.html>.
///
/// Tests whether `c` is a character of class space (a white-space character)
/// in the locale specified by `loc`.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn isspace_l(c: c_int, _loc: locale_t) -> c_int {
    isspace(c)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isupper.html>.
///
/// Tests whether `c` is a character of class upper (an uppercase letter) in
/// the current locale.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn isupper(c: c_int) -> c_int {
    c_int::from(c >= c_int::from(b'A') && c <= c_int::from(b'Z'))
}

// TODO make use of `loc`.
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isupper_l.html>.
///
/// Tests whether `c` is a character of class upper (an uppercase letter) in
/// the locale specified by `loc`.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn isupper_l(c: c_int, _loc: locale_t) -> c_int {
    isupper(c)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isxdigit.html>.
///
/// Tests whether `c` is a character of class xdigit (a hexadecimal digit) in
/// the current locale.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn isxdigit(c: c_int) -> c_int {
    c_int::from(isdigit(c) != 0 || (c | 32 >= c_int::from(b'a') && c | 32 <= c_int::from(b'f')))
}

// TODO make use of `loc`.
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isxdigit_l.html>.
///
/// Tests whether `c` is a character of class xdigit (a hexadecimal digit) in
/// the locale specified by `loc`.
///
/// Returns a non-zero value if true, 0 if false.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn isxdigit_l(c: c_int, _loc: locale_t) -> c_int {
    isxdigit(c)
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/toascii.html>.
///
/// Converts `c` to a 7bit ASCII character.
///
/// # Deprecated
/// The `toascii()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 7, and removed in Issue 8.
///
/// Not considered portable for localized applications.
#[deprecated]
#[unsafe(no_mangle)]
pub extern "C" fn toascii(c: c_int) -> c_int {
    c & 0x7f
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tolower.html>.
///
/// Returns the corresponding lowercase character for `c` according to the
/// current locale, otherwise returns the input unchanged.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn tolower(c: c_int) -> c_int {
    if isupper(c) != 0 { c | 0x20 } else { c }
}

// TODO make use of `loc`.
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tolower_l.html>.
///
/// Returns the corresponding lowercase character for `c` according to the
/// locale specified by `loc`, otherwise returns the input unchanged.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn tolower_l(c: c_int, _loc: locale_t) -> c_int {
    tolower(c)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/toupper.html>.
///
/// Returns the corresponding uppercase character for `c` according to the
/// current locale, otherwise returns the input unchanged.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn toupper(c: c_int) -> c_int {
    if islower(c) != 0 { c & !0x20 } else { c }
}

// TODO make use of `loc`.
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/toupper_l.html>.
///
/// Returns the corresponding uppercase character for `c` according to the
/// locale specified by `loc`, otherwise returns the input unchanged.
///
/// The list of character classes defined by POSIX is specified here:
/// <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
#[unsafe(no_mangle)]
pub extern "C" fn toupper_l(c: c_int, _loc: locale_t) -> c_int {
    toupper(c)
}
