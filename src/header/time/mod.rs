//! time implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/time.h.html

use core::convert::{TryFrom, TryInto};

use crate::{
    header::errno::EOVERFLOW,
    platform::{self, types::*, Pal, Sys},
};

pub use self::constants::*;

pub mod constants;
mod strftime;

#[repr(C)]
#[derive(Default)]
pub struct timespec {
    pub tv_sec: time_t,
    pub tv_nsec: c_long,
}

#[cfg(target_os = "redox")]
impl<'a> From<&'a timespec> for syscall::TimeSpec {
    fn from(tp: &timespec) -> Self {
        Self {
            tv_sec: tp.tv_sec,
            tv_nsec: tp.tv_nsec as i32,
        }
    }
}

#[repr(C)]
pub struct tm {
    pub tm_sec: c_int,
    pub tm_min: c_int,
    pub tm_hour: c_int,
    pub tm_mday: c_int,
    pub tm_mon: c_int,
    pub tm_year: c_int,
    pub tm_wday: c_int,
    pub tm_yday: c_int,
    pub tm_isdst: c_int,
    pub tm_gmtoff: c_long,
    pub tm_zone: *const c_char,
}

unsafe impl Sync for tm {}

// The C Standard says that localtime and gmtime return the same pointer.
static mut TM: tm = tm {
    tm_sec: 0,
    tm_min: 0,
    tm_hour: 0,
    tm_mday: 0,
    tm_mon: 0,
    tm_year: 0,
    tm_wday: 0,
    tm_yday: 0,
    tm_isdst: 0,
    tm_gmtoff: 0,
    tm_zone: UTC,
};

// The C Standard says that ctime and asctime return the same pointer.
static mut ASCTIME: [c_char; 26] = [0; 26];

#[repr(C)]
pub struct itimerspec {
    pub it_interval: timespec,
    pub it_value: timespec,
}

pub struct sigevent;

#[no_mangle]
pub unsafe extern "C" fn asctime(timeptr: *const tm) -> *mut c_char {
    asctime_r(timeptr, ASCTIME.as_mut_ptr().cast())
}

#[no_mangle]
pub unsafe extern "C" fn asctime_r(tm: *const tm, buf: *mut c_char) -> *mut c_char {
    let tm_sec = (*tm).tm_sec;
    let tm_min = (*tm).tm_min;
    let tm_hour = (*tm).tm_hour;
    let tm_mday = (*tm).tm_mday;
    let tm_mon = (*tm).tm_mon;
    let tm_year = (*tm).tm_year;
    let tm_wday = (*tm).tm_wday;

    /* Panic when we run into undefined behavior.
     *
     * POSIX says (since issue 7) that asctime()/asctime_r() cause UB
     * when the tm member values would cause out-of-bounds array access
     * or overflow the output buffer. This contrasts with ISO C11+,
     * which specifies UB for any tm members being outside their normal
     * ranges. While POSIX explicitly defers to the C standard in case
     * of contradictions, the assertions below follow the interpretation
     * that POSIX simply defines some of C's undefined behavior, rather
     * than conflict with the ISO standard.
     *
     * Note that C's "%.2d" formatting, unlike Rust's "{:02}"
     * formatting, does not count a minus sign against the two digits to
     * print, meaning that we must reject all negative values for
     * seconds, minutes and hours. However, C's "%3d" (for day-of-month)
     * is similar to Rust's "{:3}".
     *
     * To avoid year overflow problems (in Rust, where numeric overflow
     * is considered an error), we subtract 1900 from the endpoints,
     * rather than adding to the tm_year value. POSIX' requirement that
     * tm_year be at most {INT_MAX}-1990 is satisfied for all legal
     * values of {INT_MAX} through the max-4-digit requirement on the
     * year.
     *
     * The tm_mon and tm_wday fields are used for array access and thus
     * will already cause a panic in Rust code when out of range.
     * However, using the assertions below allows a consistent error
     * message for all fields. */
    const OUT_OF_RANGE_MESSAGE: &str = "tm member out of range";

    assert!(0 <= tm_sec && tm_sec <= 99, OUT_OF_RANGE_MESSAGE);
    assert!(0 <= tm_min && tm_min <= 99, OUT_OF_RANGE_MESSAGE);
    assert!(0 <= tm_hour && tm_hour <= 99, OUT_OF_RANGE_MESSAGE);
    assert!(-99 <= tm_mday && tm_mday <= 999, OUT_OF_RANGE_MESSAGE);
    assert!(0 <= tm_mon && tm_mon <= 11, OUT_OF_RANGE_MESSAGE);
    assert!(
        -999 - 1900 <= tm_year && tm_year <= 9999 - 1900,
        OUT_OF_RANGE_MESSAGE
    );
    assert!(0 <= tm_wday && tm_wday <= 6, OUT_OF_RANGE_MESSAGE);

    // At this point, we can safely use the values as given.
    let write_result = core::fmt::write(
        // buf may be either `*mut u8` or `*mut i8`
        &mut platform::UnsafeStringWriter(buf.cast()),
        format_args!(
            "{:.3} {:.3}{:3} {:02}:{:02}:{:02} {}\n",
            DAY_NAMES[usize::try_from(tm_wday).unwrap()],
            MON_NAMES[usize::try_from(tm_mon).unwrap()],
            tm_mday,
            tm_hour,
            tm_min,
            tm_sec,
            1900 + tm_year
        ),
    );
    match write_result {
        Ok(_) => buf,
        Err(_) => {
            /* asctime()/asctime_r() or the equivalent sprintf() call
             * have no defined errno setting */
            core::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn clock() -> clock_t {
    let mut ts = core::mem::MaybeUninit::<timespec>::uninit();

    if clock_gettime(CLOCK_PROCESS_CPUTIME_ID, ts.as_mut_ptr()) != 0 {
        return -1;
    }
    let ts = unsafe { ts.assume_init() };

    if ts.tv_sec > time_t::max_value() / CLOCKS_PER_SEC
        || ts.tv_nsec / (1_000_000_000 / CLOCKS_PER_SEC)
            > time_t::max_value() - CLOCKS_PER_SEC * ts.tv_sec
    {
        return -1;
    }

    ts.tv_sec * CLOCKS_PER_SEC + ts.tv_nsec / (1_000_000_000 / CLOCKS_PER_SEC)
}

// #[no_mangle]
pub extern "C" fn clock_getres(clock_id: clockid_t, res: *mut timespec) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn clock_gettime(clock_id: clockid_t, tp: *mut timespec) -> c_int {
    Sys::clock_gettime(clock_id, tp)
}

// #[no_mangle]
pub extern "C" fn clock_settime(clock_id: clockid_t, tp: *const timespec) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn ctime(clock: *const time_t) -> *mut c_char {
    asctime(localtime(clock))
}

#[no_mangle]
pub unsafe extern "C" fn ctime_r(clock: *const time_t, buf: *mut c_char) -> *mut c_char {
    // Using MaybeUninit<tm> seems to cause a panic during the build process
    let mut tm1 = tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: core::ptr::null_mut(),
    };
    localtime_r(clock, &mut tm1);
    asctime_r(&tm1, buf)
}

#[no_mangle]
pub extern "C" fn difftime(time1: time_t, time0: time_t) -> c_double {
    (time1 - time0) as c_double
}

// #[no_mangle]
pub extern "C" fn getdate(string: *const c_char) -> tm {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn gmtime(timer: *const time_t) -> *mut tm {
    gmtime_r(timer, &mut TM)
}

const MONTH_DAYS: [[c_int; 12]; 2] = [
    // Non-leap years:
    [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
    // Leap years:
    [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
];

#[inline(always)]
fn leap_year(year: c_int) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

#[no_mangle]
pub unsafe extern "C" fn gmtime_r(clock: *const time_t, result: *mut tm) -> *mut tm {
    /* For the details of the algorithm used here, see
     * http://howardhinnant.github.io/date_algorithms.html#civil_from_days
     * Note that we need 0-based months here, though.
     * Overall, this implementation should generate correct results as
     * long as the tm_year value will fit in a c_int. */
    const SECS_PER_DAY: time_t = 24 * 60 * 60;
    const DAYS_PER_ERA: time_t = 146097;

    let unix_secs = *clock;

    /* Day number here is possibly negative, remainder will always be
     * nonnegative when using Euclidean division */
    let unix_days: time_t = unix_secs.div_euclid(SECS_PER_DAY);

    /* In range [0, 86399]. Needs a u32 since this is larger (at least
     * theoretically) than the guaranteed range of c_int */
    let secs_of_day: u32 = unix_secs.rem_euclid(SECS_PER_DAY).try_into().unwrap();

    /* Shift origin from 1970-01-01 to 0000-03-01 and find out where we
     * are in terms of 400-year eras since then */
    let days_since_origin = unix_days + 719468;
    let era = days_since_origin.div_euclid(DAYS_PER_ERA);
    let day_of_era = days_since_origin.rem_euclid(DAYS_PER_ERA);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146096) / 365;

    /* "transformed" here refers to dates in a calendar where years
     * start on March 1 */
    let year_transformed = year_of_era + 400 * era; // retain large range, don't convert to c_int yet
    let day_of_year_transformed: c_int = (day_of_era
        - (365 * year_of_era + year_of_era / 4 - year_of_era / 100))
        .try_into()
        .unwrap();
    let month_transformed: c_int = (5 * day_of_year_transformed + 2) / 153;

    // Convert back to calendar with year starting on January 1
    let month: c_int = (month_transformed + 2) % 12; // adapted to 0-based months
    let year: time_t = if month < 2 {
        year_transformed + 1
    } else {
        year_transformed
    };

    /* Subtract 1900 *before* converting down to c_int in order to
     * maximize the range of input timestamps that will succeed */
    match c_int::try_from(year - 1900) {
        Ok(year_less_1900) => {
            let mday: c_int = (day_of_year_transformed - (153 * month_transformed + 2) / 5 + 1)
                .try_into()
                .unwrap();

            /* 1970-01-01 was a Thursday. Again, Euclidean division is
             * used to ensure a nonnegative remainder (range [0, 6]). */
            let wday: c_int = ((unix_days + 4).rem_euclid(7)).try_into().unwrap();

            /* Yes, duplicated code for now (to work on non-c_int-values
             * so that we are not constrained by the subtraction of
             * 1900) */
            let is_leap_year: bool = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);

            /* For dates that are March 1 or later, we can use day-of-
             * year in the transformed calendar. For January and
             * February, that value is sensitive to whether the previous
             * year is a leap year. Therefore, we use the already
             * computed date for those two months. */
            let yday: c_int = match month {
                0 => mday - 1,      // January
                1 => 31 + mday - 1, // February
                _ => day_of_year_transformed + if is_leap_year { 60 } else { 59 },
            };

            let hour: c_int = (secs_of_day / (60 * 60)).try_into().unwrap();
            let min: c_int = ((secs_of_day / 60) % 60).try_into().unwrap();
            let sec: c_int = (secs_of_day % 60).try_into().unwrap();

            *result = tm {
                tm_sec: sec,
                tm_min: min,
                tm_hour: hour,
                tm_mday: mday,
                tm_mon: month,
                tm_year: year_less_1900,
                tm_wday: wday,
                tm_yday: yday,
                tm_isdst: 0,
                tm_gmtoff: 0,
                tm_zone: UTC,
            };

            result
        }
        Err(_) => {
            platform::errno = EOVERFLOW;
            core::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn localtime(clock: *const time_t) -> *mut tm {
    localtime_r(clock, &mut TM)
}

#[no_mangle]
pub unsafe extern "C" fn localtime_r(clock: *const time_t, t: *mut tm) -> *mut tm {
    // TODO: Change tm_isdst, tm_gmtoff, tm_zone
    gmtime_r(clock, t)
}

#[no_mangle]
pub unsafe extern "C" fn mktime(t: *mut tm) -> time_t {
    let mut year = (*t).tm_year + 1900;
    let mut month = (*t).tm_mon;
    let mut day = (*t).tm_mday as i64 - 1;

    let leap = if leap_year(year) { 1 } else { 0 };

    if year < 1970 {
        day = MONTH_DAYS[if leap_year(year) { 1 } else { 0 }][(*t).tm_mon as usize] as i64 - day;

        while year < 1969 {
            year += 1;
            day += if leap_year(year) { 366 } else { 365 };
        }

        while month < 11 {
            month += 1;
            day += MONTH_DAYS[leap][month as usize] as i64;
        }

        -(day * (60 * 60 * 24)
            - (((*t).tm_hour as i64) * (60 * 60) + ((*t).tm_min as i64) * 60 + (*t).tm_sec as i64))
    } else {
        while year > 1970 {
            year -= 1;
            day += if leap_year(year) { 366 } else { 365 };
        }

        while month > 0 {
            month -= 1;
            day += MONTH_DAYS[leap][month as usize] as i64;
        }

        day * (60 * 60 * 24)
            + ((*t).tm_hour as i64) * (60 * 60)
            + ((*t).tm_min as i64) * 60
            + (*t).tm_sec as i64
    }
}

#[no_mangle]
pub extern "C" fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> c_int {
    Sys::nanosleep(rqtp, rmtp)
}

#[no_mangle]
pub unsafe extern "C" fn strftime(
    s: *mut c_char,
    maxsize: size_t,
    format: *const c_char,
    timeptr: *const tm,
) -> size_t {
    let ret = strftime::strftime(
        &mut platform::StringWriter(s as *mut u8, maxsize),
        format,
        timeptr,
    );
    if ret < maxsize {
        ret
    } else {
        0
    }
}

// #[no_mangle]
pub extern "C" fn strptime(buf: *const c_char, format: *const c_char, tm: *mut tm) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn time(tloc: *mut time_t) -> time_t {
    let mut ts = timespec::default();
    Sys::clock_gettime(CLOCK_REALTIME, &mut ts);
    if !tloc.is_null() {
        *tloc = ts.tv_sec
    };
    ts.tv_sec
}

#[no_mangle]
pub unsafe extern "C" fn timelocal(tm: *mut tm) -> time_t {
    //TODO: timezone
    timegm(tm)
}

#[no_mangle]
pub unsafe extern "C" fn timegm(tm: *mut tm) -> time_t {
    let mut y = (*tm).tm_year as time_t + 1900;
    let mut m = (*tm).tm_mon as time_t + 1;
    if m <= 2 {
        y -= 1;
        m += 12;
    }
    let d = (*tm).tm_mday as time_t;
    let h = (*tm).tm_hour as time_t;
    let mi = (*tm).tm_min as time_t;
    let s = (*tm).tm_sec as time_t;
    (365 * y + y / 4 - y / 100 + y / 400 + 3 * (m + 1) / 5 + 30 * m + d - 719561) * 86400
        + 3600 * h
        + 60 * mi
        + s
}

// #[no_mangle]
pub extern "C" fn timer_create(
    clock_id: clockid_t,
    evp: *mut sigevent,
    timerid: *mut timer_t,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn timer_delete(timerid: timer_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn tzset() {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn timer_settime(
    timerid: timer_t,
    flags: c_int,
    value: *const itimerspec,
    ovalue: *mut itimerspec,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn timer_gettime(timerid: timer_t, value: *mut itimerspec) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn timer_getoverrun(timerid: timer_t) -> c_int {
    unimplemented!();
}

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
