#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

pub use sys::*;
use platform::types::*;

// Move epoch from 01.01.1970 to 01.03.0000 (yes, Year 0) - this is the first
// day of a 400-year long "era", right after additional day of leap year.
// This adjustment is required only for date calculation, so instead of
// modifying time_t value (which would require 64-bit operations to work
// correctly) it's enough to adjust the calculated number of days since epoch.
pub(crate) const EPOCH_ADJUSTMENT_DAYS: c_long = 719468;
// year to which the adjustment was made
pub(crate) const ADJUSTED_EPOCH_YEAR: c_int = 0;
// 1st March of year 0 is Wednesday
pub(crate) const ADJUSTED_EPOCH_WDAY: c_long = 3;
pub(crate) const DAYS_PER_ERA: c_long = (400 - 97) * 365 + 97 * 366;
pub(crate) const DAYS_PER_CENTURY: c_ulong = (100 - 24) * 365 + 24 * 366;
pub(crate) const DAYS_PER_4_YEARS: c_ulong = 3 * 365 + 366;
pub(crate) const DAYS_PER_YEAR: c_int = 365;
pub(crate) const DAYS_IN_JANUARY: c_int = 31;
pub(crate) const DAYS_IN_FEBRUARY: c_int = 28;
pub(crate) const YEARS_PER_ERA: c_int = 400;

pub(crate) const SECSPERMIN: c_long = 60;
pub(crate) const MINSPERHOUR: c_long = 60;
pub(crate) const HOURSPERDAY: c_long = 24;
pub(crate) const SECSPERHOUR: c_long = SECSPERMIN * MINSPERHOUR;
pub(crate) const SECSPERDAY: c_long = SECSPERHOUR * HOURSPERDAY;
pub(crate) const DAYSPERWEEK: c_int = 7;

pub(crate) const YEAR_BASE: c_int = 1900;

pub(crate) const UTC: *const c_char = b"UTC\0" as *const u8 as *const c_char;

pub(crate) const DAY_NAMES: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
pub(crate) const MON_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"
];

pub(crate) const CLOCK_REALTIME: clockid_t = 0;
pub(crate) const CLOCK_MONOTONIC: clockid_t = 1;
pub(crate) const CLOCK_PROCESS_CPUTIME_ID: clockid_t = 2;
pub(crate) const CLOCK_THREAD_CPUTIME_ID: clockid_t = 3;

pub(crate) const CLOCKS_PER_SEC: time_t = 1_000_000;