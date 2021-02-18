use crate::platform::types::*;

pub use self::sys::*;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

pub(crate) const UTC: *const c_char = b"UTC\0".as_ptr().cast();

pub(crate) const DAY_NAMES: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
pub(crate) const MON_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub const CLOCK_PROCESS_CPUTIME_ID: clockid_t = 2;
// Can't be time_t because cbindgen UGH
pub const CLOCKS_PER_SEC: c_long = 1_000_000;
