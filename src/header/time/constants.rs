use crate::platform::types::{c_char, c_int, c_long, clockid_t};

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

pub use self::sys::*;

pub(crate) const UTC: *const c_char = b"UTC\0".as_ptr().cast();

pub(crate) const DAY_NAMES: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
pub(crate) const MON_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub const CLOCK_PROCESS_CPUTIME_ID: clockid_t = 2;
// Can't be time_t because cbindgen UGH
pub const CLOCKS_PER_SEC: c_long = 1_000_000;

pub const TIMER_ABSTIME: c_int = 1;

// Constants for timespec_get and timespec_getres which are C23 analogues to
// clock_gettime/getres.
// The values are offset by one for simplicity since zero represents an error.

/// `TIME_UTC` returns the time since the Unix epoch.
pub const TIME_UTC: c_int = CLOCK_REALTIME + 1;
/// `TIME_MONOTONIC` returns the time from the monotonically increasing clock.
pub const TIME_MONOTONIC: c_int = CLOCK_MONOTONIC + 1;
