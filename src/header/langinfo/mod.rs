//! `langinfo.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/langinfo.h.html>.

// TODO : involve loading locale data. Currently, the implementation only supports the "C" locale.

use crate::header::bits_locale_t::locale_t;
use core::ffi::c_char;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/langinfo.h.html>.
///
/// POSIX type for items used with `nl_langinfo`
/// In practice, this is an integer index into the string table.
pub type nl_item = i32;

// Static string table for langinfo constants
static STRING_TABLE: [&[u8]; 81] = [
    b"UTF-8\0",                // CODESET
    b"%a %b %e %H:%M:%S %Y\0", // D_T_FMT
    b"%m/%d/%y\0",             // D_FMT
    b"%H:%M:%S\0",             // T_FMT
    b"%I:%M:%S %p\0",          // T_FMT_AMPM
    b"AM\0",                   // AM_STR
    b"PM\0",                   // PM_STR
    b"Sunday\0",               // DAY_1
    b"Monday\0",               // DAY_2
    b"Tuesday\0",              // DAY_3
    b"Wednesday\0",            // DAY_4
    b"Thursday\0",             // DAY_5
    b"Friday\0",               // DAY_6
    b"Saturday\0",             // DAY_7
    b"Sun\0",                  // ABDAY_1
    b"Mon\0",                  // ABDAY_2
    b"Tue\0",                  // ABDAY_3
    b"Wed\0",                  // ABDAY_4
    b"Thu\0",                  // ABDAY_5
    b"Fri\0",                  // ABDAY_6
    b"Sat\0",                  // ABDAY_7
    b"January\0",              // MON_1
    b"February\0",             // MON_2
    b"March\0",                // MON_3
    b"April\0",                // MON_4
    b"May\0",                  // MON_5
    b"June\0",                 // MON_6
    b"July\0",                 // MON_7
    b"August\0",               // MON_8
    b"September\0",            // MON_9
    b"October\0",              // MON_10
    b"November\0",             // MON_11
    b"December\0",             // MON_12
    b"Jan\0",                  // ABMON_1
    b"Feb\0",                  // ABMON_2
    b"Mar\0",                  // ABMON_3
    b"Apr\0",                  // ABMON_4
    b"May\0",                  // ABMON_5
    b"Jun\0",                  // ABMON_6
    b"Jul\0",                  // ABMON_7
    b"Aug\0",                  // ABMON_8
    b"Sep\0",                  // ABMON_9
    b"Oct\0",                  // ABMON_10
    b"Nov\0",                  // ABMON_11
    b"Dec\0",                  // ABMON_12
    b"\0",                     // ERA
    b"\0",                     // ERA_D_FMT
    b"\0",                     // ERA_D_T_FMT
    b"\0",                     // ERA_T_FMT
    b"\0",                     // ALT_DIGITS
    b".\0",                    // RADIXCHAR
    b"\0",                     // THOUSEP
    b"^[yY]\0",                // YESEXPR
    b"^[nN]\0",                // NOEXPR
    b"yes\0",                  // YESSTR
    b"no\0",                   // NOSTR
    b".\0",                    // CRNCYSTR
    // Some languages have alternative names for
    // months. For the "C" locale, we just use one.
    b"January\0",   // ALTMON_1
    b"February\0",  // ALTMON_2
    b"March\0",     // ALTMON_3
    b"April\0",     // ALTMON_4
    b"May\0",       // ALTMON_5
    b"June\0",      // ALTMON_6
    b"July\0",      // ALTMON_7
    b"August\0",    // ALTMON_8
    b"September\0", // ALTMON_9
    b"October\0",   // ALTMON_10
    b"November\0",  // ALTMON_11
    b"December\0",  // ALTMON_12
    b"Jan\0",       // ABALTMON_1
    b"Feb\0",       // ABALTMON_2
    b"Mar\0",       // ABALTMON_3
    b"Apr\0",       // ABALTMON_4
    b"May\0",       // ABALTMON_5
    b"Jun\0",       // ABALTMON_6
    b"Jul\0",       // ABALTMON_7
    b"Aug\0",       // ABALTMON_8
    b"Sep\0",       // ABALTMON_9
    b"Oct\0",       // ABALTMON_10
    b"Nov\0",       // ABALTMON_11
    b"Dec\0",       // ABALTMON_12
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
pub const YESSTR: nl_item = 54; // Legacy
pub const NOSTR: nl_item = 55; // Legacy
pub const CRNCYSTR: nl_item = 56;

pub const ALTMON_1: nl_item = 57;
pub const ALTMON_2: nl_item = 58;
pub const ALTMON_3: nl_item = 59;
pub const ALTMON_4: nl_item = 60;
pub const ALTMON_5: nl_item = 61;
pub const ALTMON_6: nl_item = 62;
pub const ALTMON_7: nl_item = 63;
pub const ALTMON_8: nl_item = 64;
pub const ALTMON_9: nl_item = 65;
pub const ALTMON_10: nl_item = 66;
pub const ALTMON_11: nl_item = 67;
pub const ALTMON_12: nl_item = 68;

pub const ABALTMON_1: nl_item = 69;
pub const ABALTMON_2: nl_item = 70;
pub const ABALTMON_3: nl_item = 71;
pub const ABALTMON_4: nl_item = 72;
pub const ABALTMON_5: nl_item = 73;
pub const ABALTMON_6: nl_item = 74;
pub const ABALTMON_7: nl_item = 75;
pub const ABALTMON_8: nl_item = 76;
pub const ABALTMON_9: nl_item = 77;
pub const ABALTMON_10: nl_item = 78;
pub const ABALTMON_11: nl_item = 79;
pub const ABALTMON_12: nl_item = 80;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/nl_langinfo.html>.
///
/// Get a string from the langinfo table
///
/// # Safety
/// - Caller must ensure `item` is a valid `nl_item` index.
/// - Returns a pointer to a null-terminated string, or an empty string if the item is invalid.
/// - Compatibility requires mutable pointer to be returned, but it should not be mutated!
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nl_langinfo(item: nl_item) -> *mut c_char {
    // Validate the item and perform the lookup
    let ptr = if (item as usize) < STRING_TABLE.len() {
        STRING_TABLE[item as usize].as_ptr() as *const c_char
    } else {
        // Return a pointer to an empty string if the item is invalid
        b"\0".as_ptr() as *const c_char
    };
    // Mutable pointer is required (unsafe!)
    ptr as *mut c_char
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/nl_langinfo_l.html>.
///
/// Get a string from the langinfo table
///
/// # Safety
/// - Caller must ensure `item` is a valid `nl_item` index.
/// - Returns a pointer to a null-terminated string, or an empty string if the item is invalid.
/// - Compatibility requires mutable pointer to be returned, but it should not be mutated!
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nl_langinfo_l(item: nl_item, _loc: locale_t) -> *mut c_char {
    unsafe { nl_langinfo(item) }
}
