// langinfo.h implementation for Redox, following the POSIX standard.
// Following https://pubs.opengroup.org/onlinepubs/7908799/xsh/langinfo.h.html
//
// TODO : involve loading locale data. Currently, the implementation only supports the "C" locale.

use core::ffi::c_char;

/// TODO : the POSIX type for items used with `nl_langinfo` (not implemented), which is defined in <nl_types.h>
pub type nl_item = i32;

// Constant definitions
const ITEMS: &[(nl_item, &[u8])] = &[
    (CODESET, b"UTF-8\0"),
    (D_T_FMT, b"%a %b %e %H:%M:%S %Y\0"),
    (D_FMT, b"%m/%d/%y\0"),
    (T_FMT, b"%H:%M:%S\0"),
    (T_FMT_AMPM, b"%I:%M:%S %p\0"),
    (AM_STR, b"AM\0"),
    (PM_STR, b"PM\0"),
    (DAY_1, b"Sunday\0"),
    (DAY_2, b"Monday\0"),
    (DAY_3, b"Tuesday\0"),
    (DAY_4, b"Wednesday\0"),
    (DAY_5, b"Thursday\0"),
    (DAY_6, b"Friday\0"),
    (DAY_7, b"Saturday\0"),
    (ABDAY_1, b"Sun\0"),
    (ABDAY_2, b"Mon\0"),
    (ABDAY_3, b"Tue\0"),
    (ABDAY_4, b"Wed\0"),
    (ABDAY_5, b"Thu\0"),
    (ABDAY_6, b"Fri\0"),
    (ABDAY_7, b"Sat\0"),
    (MON_1, b"January\0"),
    (MON_2, b"February\0"),
    (MON_3, b"March\0"),
    (MON_4, b"April\0"),
    (MON_5, b"May\0"),
    (MON_6, b"June\0"),
    (MON_7, b"July\0"),
    (MON_8, b"August\0"),
    (MON_9, b"September\0"),
    (MON_10, b"October\0"),
    (MON_11, b"November\0"),
    (MON_12, b"December\0"),
    (ABMON_1, b"Jan\0"),
    (ABMON_2, b"Feb\0"),
    (ABMON_3, b"Mar\0"),
    (ABMON_4, b"Apr\0"),
    (ABMON_5, b"May\0"),
    (ABMON_6, b"Jun\0"),
    (ABMON_7, b"Jul\0"),
    (ABMON_8, b"Aug\0"),
    (ABMON_9, b"Sep\0"),
    (ABMON_10, b"Oct\0"),
    (ABMON_11, b"Nov\0"),
    (ABMON_12, b"Dec\0"),
    (ERA, b"\0"),
    (ERA_D_FMT, b"\0"),
    (ERA_D_T_FMT, b"\0"),
    (ERA_T_FMT, b"\0"),
    (ALT_DIGITS, b"\0"),
    (RADIXCHAR, b".\0"),
    (THOUSEP, b"\0"),
    (CRNCYSTR, b".\0"),
    (YESEXPR, b"^[yY]\0"),
    (NOEXPR, b"^[nN]\0"),
    (YESSTR, b"yes\0"),
    (NOSTR, b"no\0"),
];

// Item constants
pub const CODESET: nl_item = 0;
pub const D_T_FMT: nl_item = 1;
pub const D_FMT: nl_item = 2;
pub const T_FMT: nl_item = 3;
pub const T_FMT_AMPM: nl_item = 4;
pub const AM_STR: nl_item = 5;
pub const PM_STR: nl_item = 6;

pub const DAY_1: nl_item = 7;
pub const DAY_2: nl_item = 8;
pub const DAY_3: nl_item = 9;
pub const DAY_4: nl_item = 10;
pub const DAY_5: nl_item = 11;
pub const DAY_6: nl_item = 12;
pub const DAY_7: nl_item = 13;

pub const ABDAY_1: nl_item = 14;
pub const ABDAY_2: nl_item = 15;
pub const ABDAY_3: nl_item = 16;
pub const ABDAY_4: nl_item = 17;
pub const ABDAY_5: nl_item = 18;
pub const ABDAY_6: nl_item = 19;
pub const ABDAY_7: nl_item = 20;

pub const MON_1: nl_item = 21;
pub const MON_2: nl_item = 22;
pub const MON_3: nl_item = 23;
pub const MON_4: nl_item = 24;
pub const MON_5: nl_item = 25;
pub const MON_6: nl_item = 26;
pub const MON_7: nl_item = 27;
pub const MON_8: nl_item = 28;
pub const MON_9: nl_item = 29;
pub const MON_10: nl_item = 30;
pub const MON_11: nl_item = 31;
pub const MON_12: nl_item = 32;

pub const ABMON_1: nl_item = 33;
pub const ABMON_2: nl_item = 34;
pub const ABMON_3: nl_item = 35;
pub const ABMON_4: nl_item = 36;
pub const ABMON_5: nl_item = 37;
pub const ABMON_6: nl_item = 38;
pub const ABMON_7: nl_item = 39;
pub const ABMON_8: nl_item = 40;
pub const ABMON_9: nl_item = 41;
pub const ABMON_10: nl_item = 42;
pub const ABMON_11: nl_item = 43;
pub const ABMON_12: nl_item = 44;

pub const ERA: nl_item = 45;
pub const ERA_D_FMT: nl_item = 46;
pub const ERA_D_T_FMT: nl_item = 47;
pub const ERA_T_FMT: nl_item = 48;
pub const ALT_DIGITS: nl_item = 49;
pub const RADIXCHAR: nl_item = 50;
pub const THOUSEP: nl_item = 51;
pub const YESEXPR: nl_item = 52;
pub const NOEXPR: nl_item = 53;
pub const YESSTR: nl_item = 54; // Legaxy
pub const NOSTR: nl_item = 55; // Legacy
pub const CRNCYSTR: nl_item = 56;

const fn is_valid_nl_item(item: nl_item) -> bool {
    item >= CODESET && item <= CRNCYSTR
}

/// Return a pointer to a C string for the specified `nl_item`.
/// Currently only supports the "C" locale.
///
/// # Safety
/// - Returns a static string pointer valid for program lifetime
/// - Caller must not modify or free the returned pointer
/// - The returned pointer is always null-terminated
///
/// # Returns
/// - Valid pointer to a null-terminated string for valid items
/// - Pointer to empty string for invalid items
#[no_mangle]
pub unsafe extern "C" fn nl_langinfo(item: nl_item) -> *const c_char {
    if !is_valid_nl_item(item) {
        return b"\0".as_ptr() as *const c_char;
    }

    for &(id, string) in ITEMS {
        if id == item {
            return string.as_ptr() as *const c_char;
        }
    }

    b"\0".as_ptr() as *const c_char
}
