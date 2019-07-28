use crate::platform::types::*;

pub const CLOCK_REALTIME: c_int = 0;
pub const CLOCK_MONOTONIC: c_int = 1;
pub const CLOCK_PROCESS_CPUTIME_ID: c_int = 2;
pub const CLOCK_THREAD_CPUTIME_ID: c_int = 3;
pub const CLOCK_MONOTONIC_RAW: c_int = 4;
pub const CLOCK_REALTIME_COARSE: c_int = 5;
pub const CLOCK_MONOTONIC_COARSE: c_int = 6;
pub const CLOCK_BOOTTIME: c_int = 7;
pub const CLOCK_REALTIME_ALARM: c_int = 8;
pub const CLOCK_BOOTTIME_ALARM: c_int = 9;
pub const CLOCK_TAI: c_int = 11;
