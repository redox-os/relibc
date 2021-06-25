//! wchar implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/wchar.h.html

use crate::platform::types::*;
use crate::c_str::CStr;

mod alpha;
use alpha::alpha_table;
mod casecmp;
use casecmp::casemap;
mod punct;
use punct::punct_table;
pub const WEOF: wint_t = 0xFFFF_FFFFu32;

#[no_mangle]
pub extern "C" fn iswalnum(wc: wint_t) -> c_int {
    ((iswdigit(wc) == 1) || (iswalpha(wc) == 1)) as c_int
}

#[no_mangle]
pub extern "C" fn iswalpha(wc: wint_t) -> c_int {
    if wc < 0x20000 {
        let a = wc >> 8;
        let b = alpha_table[a as usize] as u32;
        let c = b * 32 + ((wc & 255) >> 3);
        let d = alpha_table[c as usize] as u32;
        ((d >> (wc & 7)) & 1) as c_int
    } else if wc < 0x2fffe {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn iswblank(wc: wint_t) -> c_int {
    if wc == (' ' as u32) {
        1
    } else if wc == ('\t' as u32) {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn iswcntrl(wc: wint_t) -> c_int {
    (wc < 32 || (wc - 0x7f) < 33 || (wc - 0x2028) < 2 || (wc - 0xfff9) < 3) as c_int
}

#[no_mangle]
pub extern "C" fn iswdigit(wc: wint_t) -> c_int {
    (48 <= wc && wc <= 57) as c_int
}

#[no_mangle]
pub extern "C" fn iswgraph(wc: wint_t) -> c_int {
    ((iswspace(wc) == 0) && (iswprint(wc) == 1)) as c_int
}

#[no_mangle]
pub extern "C" fn iswlower(wc: wint_t) -> c_int {
    return (towupper(wc) != wc) as c_int;
}

#[no_mangle]
pub extern "C" fn iswprint(wc: wint_t) -> c_int {
    if wc < 0xff {
        (wc + 1 & 0x7f >= 0x21) as c_int
    } else if wc < 0x2028 {
        1
    } else if wc - 0x202a < 0xd800 - 0x202a {
        1
    } else if wc - 0xe000 < 0xfff9 - 0xe000 {
        1
    } else if wc - 0xfffc > 0x10ffff - 0xfffc {
        0
    } else if wc & 0xfffe == 0xfffe {
        0
    } else {
        1
    }
}

#[no_mangle]
pub extern "C" fn iswpunct(wc: wint_t) -> c_int {
    if wc < 0x20000 {
        let a = wc >> 8;
        let b = punct_table[a as usize] as u32;
        let c = b * 32 + ((wc & 255) >> 3);
        let d = punct_table[c as usize] as u32;
        ((d >> (wc & 7)) & 1) as c_int
    } else {
        0
    }
}

// #[no_mangle]
pub extern "C" fn iswspace(wc: wint_t) -> c_int {
    let spaces: [wint_t; 21] = [
        (' ' as wint_t),
        ('\t' as wint_t),
        ('\n' as wint_t),
        ('\r' as wint_t),
        11,
        12,
        0x0085,
        0x2000,
        0x2001,
        0x2002,
        0x2003,
        0x2004,
        0x2005,
        0x2006,
        0x2008,
        0x2009,
        0x200a,
        0x2028,
        0x2029,
        0x205f,
        0x3000,
    ];
    spaces.contains(&wc) as c_int
}

#[no_mangle]
pub extern "C" fn iswupper(wc: wint_t) -> c_int {
    return (towlower(wc) != wc) as c_int;
}

#[no_mangle]
pub extern "C" fn iswxdigit(wc: wint_t) -> c_int {
    if 48 <= wc && wc <= 57 {
        1
    } else if 65 <= wc && wc <= 70 {
        1
    } else if 97 <= wc && wc <= 102 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn iswctype(wc: wint_t, charclass: wctype_t) -> c_int {
    match charclass {
        1 => iswalnum(wc),
        2 => iswalpha(wc),
        3 => iswblank(wc),
        4 => iswcntrl(wc),
        5 => iswdigit(wc),
        6 => iswgraph(wc),
        7 => iswlower(wc),
        8 => iswprint(wc),
        9 => iswpunct(wc),
        10 => iswupper(wc),
        11 => iswxdigit(wc),
        _ => 0,
    }
}


#[no_mangle]
pub fn towctrans(wc: wint_t, trans: wctrans_t) -> wint_t {
    match trans {
        1 => towupper(wc),
        2 => towlower(wc),
        _ => wc,
    }
}

#[no_mangle]
pub extern "C" fn towlower(wc: wint_t) -> wint_t {
    casemap(wc, 0)
}

#[no_mangle]
pub extern "C" fn towupper(wc: wint_t) -> wint_t {
    casemap(wc, 1)
}

#[no_mangle]
pub unsafe extern "C" fn wctype(property: *const c_char) -> wctype_t {
    let property = CStr::from_ptr(property).to_bytes();
    let names = [
        "alnum".as_bytes(), "alpha".as_bytes(), "blank".as_bytes(),
        "cntrl".as_bytes(), "digit".as_bytes(), "graph".as_bytes(),
        "lower".as_bytes(), "print".as_bytes(), "punct".as_bytes(),
        "space".as_bytes(), "upper".as_bytes(), "xdigit".as_bytes()
    ];
    for (i, name) in names.iter().enumerate() {
        if property.eq(*name) {
            return i as wctype_t
        }
    }
    return 0
}


#[no_mangle]
pub unsafe extern "C" fn wctrans(class: *const c_char) -> wctrans_t
{
    let class = CStr::from_ptr(class).to_bytes();
    if class.eq("toupper".as_bytes()) {
        1
    } else if class.eq("tolower".as_bytes()) {
        2
    } else {
        0
    }
}
