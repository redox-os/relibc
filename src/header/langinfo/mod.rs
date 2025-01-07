// langinfo.h implementation for Redox, following the POSIX standard.
// Following https://pubs.opengroup.org/onlinepubs/7908799/xsh/langinfo.h.html
//
// TODO : involve loading locale data. Currently, the implementation only supports the "C" locale.

use core::ffi::c_char;

/// POSIX type for items used with `nl_langinfo`
/// In practice, this is an integer index into the string table.
pub type nl_item = i32;

// Static string table for langinfo constants
static STRING_TABLE: [&[u8]; 57] = [
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
    b".\0",                    // CRNCYSTR
    b"^[yY]\0",                // YESEXPR
    b"^[nN]\0",                // NOEXPR
    b"yes\0",                  // YESSTR
    b"no\0",                   // NOSTR
];

/// Get a string from the langinfo table
///
/// # Safety
/// - Caller must ensure `item` is a valid `nl_item` index.
/// - Returns a pointer to a null-terminated string, or an empty string if the item is invalid.
#[no_mangle]
pub unsafe extern "C" fn nl_langinfo(item: nl_item) -> *const c_char {
    // Validate the item and perform the lookup
    if (item as usize) < STRING_TABLE.len() {
        STRING_TABLE[item as usize].as_ptr() as *const c_char
    } else {
        // Return a pointer to an empty string if the item is invalid
        b"\0".as_ptr() as *const c_char
    }
}
