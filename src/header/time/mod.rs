//! `time.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/time.h.html>.

use crate::{
    c_str::{CStr, CString},
    error::{Errno, ResultExt},
    fs::File,
    header::{
        errno::{EFAULT, EOVERFLOW},
        fcntl::O_RDONLY,
        signal::sigevent,
        stdlib::getenv,
        unistd::readlink,
    },
    io::Read,
    out::Out,
    platform::{
        self, Pal, Sys,
        types::{
            c_char, c_double, c_int, c_long, c_ulong, clock_t, clockid_t, pid_t, pthread_t, size_t,
            time_t, timer_t,
        },
    },
    sync::{Mutex, MutexGuard},
};
use alloc::{boxed::Box, collections::BTreeSet, string::String, vec::Vec};
use chrono::{
    DateTime, Datelike, FixedOffset, NaiveDate, NaiveDateTime, Offset, ParseError, TimeZone,
    Timelike, Utc, format::ParseErrorKind, offset::MappedLocalTime,
};
use chrono_tz::{OffsetComponents, OffsetName, Tz};
use core::{
    cell::OnceCell,
    convert::{TryFrom, TryInto},
    fmt::Debug,
    mem, ptr,
};

pub use self::constants::*;

pub mod constants;

mod strftime;
mod strptime;
pub use strptime::strptime;

const YEARS_PER_ERA: time_t = 400;
const DAYS_PER_ERA: time_t = 146097;
const SECS_PER_DAY: time_t = 24 * 60 * 60;
const NANOSECONDS: i64 = 1_000_000_000;
const UTC_STR: &core::ffi::CStr = c"UTC";

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/time.h.html>.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct timespec {
    pub tv_sec: time_t,
    pub tv_nsec: c_long,
}

impl timespec {
    // TODO: Write test

    /// similar logic with timeradd
    pub fn add(base: timespec, interval: timespec) -> Option<timespec> {
        let delta_sec = base.tv_sec + interval.tv_sec;
        let delta_nsec = base.tv_nsec + interval.tv_nsec;

        if delta_sec < 0 || delta_nsec < 0 {
            return None;
        }

        Some(Self {
            tv_sec: delta_sec + (delta_nsec / NANOSECONDS),
            tv_nsec: delta_nsec % NANOSECONDS,
        })
    }
    /// similar logic with timersub
    pub fn subtract(later: timespec, earlier: timespec) -> Option<timespec> {
        let delta_sec = later.tv_sec - earlier.tv_sec;
        let delta_nsec = later.tv_nsec - earlier.tv_nsec;

        let time = if delta_nsec < 0 {
            let roundup_sec = -delta_nsec / NANOSECONDS + 1;
            timespec {
                tv_sec: delta_sec - roundup_sec,
                tv_nsec: roundup_sec * NANOSECONDS - delta_nsec,
            }
        } else {
            timespec {
                tv_sec: delta_sec + (delta_nsec / NANOSECONDS),
                tv_nsec: delta_nsec % NANOSECONDS,
            }
        };

        if time.tv_sec < 0 {
            // https://man7.org/linux/man-pages/man2/settimeofday.2.html
            // caller should return EINVAL
            return None;
        }

        Some(time)
    }
    pub fn is_default(&self) -> bool {
        return self.tv_nsec == 0 && self.tv_sec == 0;
    }
}

#[cfg(target_os = "redox")]
impl<'a> From<&'a timespec> for syscall::TimeSpec {
    fn from(tp: &timespec) -> Self {
        Self {
            tv_sec: tp.tv_sec as _,
            tv_nsec: tp.tv_nsec as _,
        }
    }
}

/// timer_t internal data, ABI unstable
#[repr(C)]
#[derive(Clone)]
#[cfg(target_os = "redox")]
pub(crate) struct timer_internal_t {
    pub clockid: clockid_t,
    pub timerfd: usize,
    pub eventfd: usize,
    pub evp: sigevent,
    pub thread: pthread_t,
    pub caller_thread: crate::pthread::OsTid,
    // relibc handles it_interval, not the kernel
    pub next_wake_time: itimerspec,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/time.h.html>.
#[repr(C)]
pub struct tm {
    pub tm_sec: c_int,          // 0 - 60
    pub tm_min: c_int,          // 0 - 59
    pub tm_hour: c_int,         // 0 - 23
    pub tm_mday: c_int,         // 1 - 31
    pub tm_mon: c_int,          // 0 - 11
    pub tm_year: c_int,         // years since 1900
    pub tm_wday: c_int,         // 0 - 6 (Sunday - Saturday)
    pub tm_yday: c_int,         // 0 - 365
    pub tm_isdst: c_int,        // >0 if DST, 0 if not, <0 if unknown
    pub tm_gmtoff: c_long,      // offset from UTC in seconds
    pub tm_zone: *const c_char, // timezone abbreviation
}

unsafe impl Sync for tm {}

// The C Standard says that localtime and gmtime return the same pointer.
static mut TM: tm = blank_tm();

// The C Standard says that ctime and asctime return the same pointer.
static mut ASCTIME: [c_char; 26] = [0; 26];

#[repr(transparent)]
pub struct TzName([*mut c_char; 2]);

unsafe impl Sync for TzName {}

// Name storage for the `tm_zone` field.
static TIMEZONE_NAMES: Mutex<OnceCell<BTreeSet<CString>>> = Mutex::new(OnceCell::new());

// Hold `TIMEZONE_LOCK` when updating `tzname`, `timezone`, and `daylight`.
static TIMEZONE_LOCK: Mutex<(Option<CString>, Option<CString>)> = Mutex::new((None, None));

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/time.h.html>.
#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut daylight: c_int = 0;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/time.h.html>.
#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut timezone: c_long = 0;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/time.h.html>.
#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut tzname: TzName = TzName([ptr::null_mut(); 2]);

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut getdate_err: c_int = 0;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/time.h.html>.
#[repr(C)]
#[derive(Clone, Default)]
pub struct itimerspec {
    pub it_interval: timespec,
    pub it_value: timespec,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/asctime.html>.
///
/// # Deprecation
/// The `asctime()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 7.
#[deprecated]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn asctime(timeptr: *const tm) -> *mut c_char {
    asctime_r(timeptr, &raw mut ASCTIME as *mut _)
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/asctime.html>.
///
/// # Deprecation
/// The `asctime_r()` was marked obsolescent in the Open Group Base
/// Specifications Issue 7, and removed in Issue 8.
#[deprecated]
#[unsafe(no_mangle)]
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

    assert!(0 <= tm_sec && tm_sec <= 99, "{}", OUT_OF_RANGE_MESSAGE);
    assert!(0 <= tm_min && tm_min <= 99, "{}", OUT_OF_RANGE_MESSAGE);
    assert!(0 <= tm_hour && tm_hour <= 99, "{}", OUT_OF_RANGE_MESSAGE);
    assert!(-99 <= tm_mday && tm_mday <= 999, "{}", OUT_OF_RANGE_MESSAGE);
    assert!(0 <= tm_mon && tm_mon <= 11, "{}", OUT_OF_RANGE_MESSAGE);
    assert!(
        -999 - 1900 <= tm_year && tm_year <= 9999 - 1900,
        "{}",
        OUT_OF_RANGE_MESSAGE
    );
    assert!(0 <= tm_wday && tm_wday <= 6, "{}", OUT_OF_RANGE_MESSAGE);

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
            ptr::null_mut()
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/clock.html>.
#[unsafe(no_mangle)]
pub extern "C" fn clock() -> clock_t {
    let mut ts = mem::MaybeUninit::<timespec>::uninit();

    if unsafe { clock_gettime(CLOCK_PROCESS_CPUTIME_ID, ts.as_mut_ptr()) } != 0 {
        return -1;
    }
    let ts = unsafe { ts.assume_init() };

    let clocks =
        ts.tv_sec * CLOCKS_PER_SEC as i64 + (ts.tv_nsec / (1_000_000_000 / CLOCKS_PER_SEC)) as i64;
    match clock_t::try_from(clocks) {
        Ok(ok) => ok,
        Err(_err) => -1,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/clock_getcpuclockid.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn clock_getcpuclockid(pid: pid_t, clock_id: *mut clockid_t) -> c_int {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/clock_getres.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn clock_getres(clock_id: clockid_t, res: *mut timespec) -> c_int {
    Sys::clock_getres(clock_id, Out::nullable(res))
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/clock_getres.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn clock_gettime(clock_id: clockid_t, tp: *mut timespec) -> c_int {
    Sys::clock_gettime(clock_id, Out::nonnull(tp))
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/clock_nanosleep.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn clock_nanosleep(
    clock_id: clockid_t,
    flags: c_int,
    rqtp: *const timespec,
    rmtp: *mut timespec,
) -> c_int {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/clock_getres.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn clock_settime(clock_id: clockid_t, tp: *const timespec) -> c_int {
    Sys::clock_settime(clock_id, tp)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ctime.html>.
///
/// # Deprecation
/// The `ctime()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 7.
#[deprecated]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ctime(clock: *const time_t) -> *mut c_char {
    asctime(localtime(clock))
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/ctime.html>.
///
/// # Deprecation
/// The `ctime_r()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 7, and removed in Issue 8.
#[deprecated]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ctime_r(clock: *const time_t, buf: *mut c_char) -> *mut c_char {
    // Using MaybeUninit<tm> seems to cause a panic during the build process
    let mut tm1 = blank_tm();
    localtime_r(clock, &mut tm1);
    asctime_r(&tm1, buf)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/difftime.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn difftime(time1: time_t, time0: time_t) -> c_double {
    (time1 - time0) as _
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getdate.html>.
// #[unsafe(no_mangle)]
pub unsafe extern "C" fn getdate(string: *const c_char) -> *const tm {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/gmtime.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gmtime(timer: *const time_t) -> *mut tm {
    gmtime_r(timer, &raw mut TM)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/gmtime.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gmtime_r(clock: *const time_t, result: *mut tm) -> *mut tm {
    let _ = get_localtime(*clock, result);
    result
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/localtime.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn localtime(clock: *const time_t) -> *mut tm {
    localtime_r(clock, &raw mut TM)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/localtime.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn localtime_r(clock: *const time_t, t: *mut tm) -> *mut tm {
    let mut lock = TIMEZONE_LOCK.lock();
    clear_timezone(&mut lock);
    if let (Some(std_time), dst_time) = get_localtime(*clock, t) {
        set_timezone(&mut lock, &std_time, dst_time);
    }
    t
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mktime.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mktime(timeptr: *mut tm) -> time_t {
    let mut lock = TIMEZONE_LOCK.lock();
    clear_timezone(&mut lock);

    let year = (*timeptr).tm_year + 1900;
    let month = ((*timeptr).tm_mon + 1) as _;
    let day = (*timeptr).tm_mday as _;
    let hour = (*timeptr).tm_hour as _;
    let minute = (*timeptr).tm_min as _;
    let second = (*timeptr).tm_sec as _;

    let naive_local = match NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|date| date.and_hms_opt(hour, minute, second))
    {
        Some(datetime) => datetime,
        None => {
            platform::ERRNO.set(EOVERFLOW);
            return -1;
        }
    };

    let offset = get_offset((*timeptr).tm_gmtoff).unwrap();
    let tz = time_zone();
    // Create DateTime<FixedOffset>
    let datetime = match offset.from_local_datetime(&naive_local) {
        MappedLocalTime::Single(datetime) => datetime,
        _ => {
            platform::ERRNO.set(EOVERFLOW);
            return -1;
        }
    };

    // Convert to UTC and get timestamp
    let tz_datetime = datetime.with_timezone(&tz);
    let timestamp = tz_datetime.timestamp();

    ptr::write(timeptr, datetime_to_tm(&tz_datetime));

    // Convert UTC time to local time
    if let (std_time, dst_time) = match tz.timestamp_opt(timestamp, 0) {
        MappedLocalTime::Single(t) => (t, None),
        // This variant contains the two possible results, in the order (earliest, latest).
        MappedLocalTime::Ambiguous(t1, t2) => (t2, Some(t1)),
        MappedLocalTime::None => return timestamp,
    } {
        set_timezone(&mut lock, &std_time, dst_time);
    }

    timestamp
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/nanosleep.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> c_int {
    Sys::nanosleep(rqtp, rmtp).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strftime.html>.
#[unsafe(no_mangle)]
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
    if ret < maxsize { ret } else { 0 }
}

// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strftime.html>.
// TODO: needs locale_t
// #[unsafe(no_mangle)]
/*pub extern "C" fn strftime_l(s: *mut char, maxsize: size_t, format: *const c_char, timeptr: *const tm, locale: locale_t) -> size_t {
    unimplemented!();
}*/

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/time.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn time(tloc: *mut time_t) -> time_t {
    let mut ts = timespec::default();
    Sys::clock_gettime(CLOCK_REALTIME, Out::from_mut(&mut ts));
    if !tloc.is_null() {
        *tloc = ts.tv_sec
    };
    ts.tv_sec
}

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/timegm.3.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn timegm(tm: *mut tm) -> time_t {
    let tm_val = &mut *tm;
    let dt = match convert_tm_generic(&Utc, tm_val) {
        Some(dt) => dt,
        None => return -1,
    };

    (*tm).tm_wday = dt.weekday().num_days_from_sunday() as _;
    (*tm).tm_yday = dt.ordinal0() as _; // day of year starting at 0
    (*tm).tm_isdst = 0; // UTC does not use DST
    (*tm).tm_gmtoff = 0; // UTC offset is zero
    (*tm).tm_zone = UTC_STR.as_ptr() as *const c_char;

    dt.timestamp()
}

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/timegm.3.html>.
#[deprecated]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn timelocal(tm: *mut tm) -> time_t {
    let tm_val = &mut *tm;
    let tz = time_zone();
    let dt = match convert_tm_generic(&tz, tm_val) {
        Some(dt) => dt,
        None => return -1,
    };

    let tz_name = CString::new(tz.name()).unwrap();
    (*tm).tm_wday = dt.weekday().num_days_from_sunday() as _;
    (*tm).tm_yday = dt.ordinal0() as _; // day of year starting at 0
    (*tm).tm_isdst = dt.offset().dst_offset().num_hours() as _;
    (*tm).tm_gmtoff = dt.offset().fix().local_minus_utc() as _;
    (*tm).tm_zone = tz_name.into_raw().cast();

    dt.timestamp()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/timer_create.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn timer_create(
    clock_id: clockid_t,
    evp: *mut sigevent,
    timerid: *mut timer_t,
) -> c_int {
    if evp.is_null() || timerid.is_null() {
        return Err(Errno(EFAULT)).or_minus_one_errno();
    }
    let (evp, timerid) = unsafe { (&*evp, Out::nonnull(timerid)) };
    Sys::timer_create(clock_id, &evp, timerid)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/timer_delete.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn timer_delete(timerid: timer_t) -> c_int {
    if timerid.is_null() {
        return Err(Errno(EFAULT)).or_minus_one_errno();
    }
    Sys::timer_delete(timerid).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/timer_getoverrun.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn timer_getoverrun(timerid: timer_t) -> c_int {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/timer_getoverrun.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn timer_gettime(timerid: timer_t, value: *mut itimerspec) -> c_int {
    if timerid.is_null() || value.is_null() {
        return Err(Errno(EFAULT)).or_minus_one_errno();
    }
    let value = unsafe { Out::nonnull(value) };
    Sys::timer_gettime(timerid, value)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/timer_getoverrun.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn timer_settime(
    timerid: timer_t,
    flags: c_int,
    value: *const itimerspec,
    ovalue: *mut itimerspec,
) -> c_int {
    if timerid.is_null() || value.is_null() {
        return Err(Errno(EFAULT)).or_minus_one_errno();
    }
    let (value, ovalue) = unsafe { (&*value, Out::nullable(ovalue)) };
    Sys::timer_settime(timerid, flags, value, ovalue)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// ISO C equivalent to [`Sys::clock_gettime`].
///
/// The main differences are that this function:
/// * returns `0` on error and `base` on success
/// * only mandates TIME_UTC as a base
///
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/timespec_get.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn timespec_get(tp: *mut timespec, base: c_int) -> c_int {
    let tp = unsafe { Out::nonnull(tp) };
    Sys::clock_gettime(base - 1, tp).map(|()| base).unwrap_or(0)
}

/// ISO C equivalent to [`Sys::clock_getres`].
///
/// The main differences are that this function:
/// * returns `0` on error and `base` on success
/// * only mandates TIME_UTC as a base
#[unsafe(no_mangle)]
pub unsafe extern "C" fn timespec_getres(res: *mut timespec, base: c_int) -> c_int {
    let res = unsafe { Out::nullable(res) };
    Sys::clock_getres(base - 1, res).map(|()| base).unwrap_or(0)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tzset.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tzset() {
    let mut lock = TIMEZONE_LOCK.lock();
    unsafe { clear_timezone(&mut lock) };

    let tz = time_zone();
    let datetime = now();
    let (std_time, dst_time) = match tz.from_local_datetime(&datetime) {
        MappedLocalTime::Single(t) => (t, None),
        // This variant contains the two possible results, in the order (earliest, latest).
        MappedLocalTime::Ambiguous(t1, t2) => (t2, Some(t1)),
        MappedLocalTime::None => return,
    };

    set_timezone(&mut lock, &std_time, dst_time)
}

fn convert_tm_generic<Tz: TimeZone>(tz: &Tz, tm_val: &tm) -> Option<DateTime<Tz>> {
    // Adjust fields: tm_year is years since 1900; tm_mon is 0-indexed.
    let year = tm_val.tm_year + 1900;
    let month = tm_val.tm_mon + 1; // convert to 1-indexed
    let day = tm_val.tm_mday;
    let hour = tm_val.tm_hour;
    let minute = tm_val.tm_min;
    let second = tm_val.tm_sec;

    match tz.with_ymd_and_hms(
        year,
        month as u32,
        day as u32,
        hour as u32,
        minute as u32,
        second as u32,
    ) {
        MappedLocalTime::Single(dt) => Some(dt),
        MappedLocalTime::Ambiguous(dt1, _dt2) => Some(dt1), // choose the earliest value
        _ => None,
    }
}

fn clear_timezone(guard: &mut MutexGuard<'_, (Option<CString>, Option<CString>)>) {
    guard.0 = None;
    guard.1 = None;
    unsafe {
        tzname.0[0] = ptr::null_mut();
        tzname.0[1] = ptr::null_mut();
        timezone = 0;
        daylight = 0;
    }
}

#[inline(always)]
fn get_system_time_zone<'a>() -> Option<&'a str> {
    // Resolve the symlink for localtime
    const BSIZE: size_t = 100;
    let mut buffer: [u8; BSIZE] = [0; BSIZE];

    #[cfg(not(target_os = "redox"))]
    let (localtime, prefix) = (c"/etc/localtime", "/usr/share/zoneinfo/");

    #[cfg(target_os = "redox")]
    let (localtime, prefix) = (c"/etc/localtime", "/usr/share/zoneinfo/");

    if unsafe { readlink(localtime.as_ptr().cast(), buffer.as_mut_ptr().cast(), BSIZE) } == -1 {
        return None;
    }

    let path = unsafe { CStr::from_ptr(buffer.as_mut_ptr().cast()) };

    if let Ok(tz_name) = path.to_str() {
        if let Some(stripped) = tz_name.strip_prefix(prefix) {
            return Some(stripped);
        }
    }

    None
}

fn get_current_time_zone<'a>() -> &'a str {
    // Check the `TZ` environment variable
    let tz_env = unsafe { getenv(c"TZ".as_ptr() as _) };
    if !tz_env.is_null() {
        if let Ok(tz) = unsafe { CStr::from_ptr(tz_env) }.to_str() {
            return tz;
        }
    }

    // Fallback to the system's default time zone
    if let Some(tz) = get_system_time_zone() {
        return tz;
    }

    // If all else fails, use UTC
    "UTC"
}

#[inline(always)]
fn time_zone() -> Tz {
    get_current_time_zone().parse().unwrap_or(Tz::UTC)
}

#[inline(always)]
fn now() -> NaiveDateTime {
    let mut now = timespec::default();
    unsafe {
        Sys::clock_gettime(CLOCK_REALTIME, Out::from_mut(&mut now));
    }
    NaiveDateTime::from_timestamp(now.tv_sec, now.tv_nsec as _)
}

#[inline(always)]
fn get_localtime(clock: time_t, t: *mut tm) -> (Option<DateTime<Tz>>, Option<DateTime<Tz>>) {
    let tz = time_zone();

    // Convert UTC time to local time
    let (std_time, dst_time) = match tz.timestamp_opt(clock, 0) {
        MappedLocalTime::Single(t) => (Some(t), None),
        // This variant contains the two possible results, in the order (earliest, latest).
        MappedLocalTime::Ambiguous(t1, t2) => (Some(t2), Some(t1)),
        MappedLocalTime::None => return (None, None),
    };

    unsafe { ptr::write(t, datetime_to_tm(&std_time.unwrap())) };
    (std_time, dst_time)
}

unsafe fn datetime_to_tm(local_time: &DateTime<Tz>) -> tm {
    let tz = local_time.timezone().name();
    let tz = tz.strip_prefix("Etc/").or(Some(tz)).unwrap();

    let mut t = blank_tm();
    // Populate the `tm` structure
    t.tm_sec = local_time.second() as _;
    t.tm_min = local_time.minute() as _;
    t.tm_hour = local_time.hour() as _;
    t.tm_mday = local_time.day() as _;
    t.tm_mon = local_time.month0() as _; // 0-based month
    t.tm_year = (local_time.year() - 1900) as _; // Years since 1900
    t.tm_wday = local_time.weekday().num_days_from_sunday() as _;
    t.tm_yday = local_time.ordinal0() as _; // 0-based day of year

    let offset = local_time.offset();
    t.tm_isdst = offset.dst_offset().num_hours() as _;
    // Get the UTC offset in seconds
    t.tm_gmtoff = offset.fix().local_minus_utc() as _;

    let tm_zone = {
        let mut timezone_names = TIMEZONE_NAMES.lock();
        timezone_names.get_or_init(BTreeSet::new);
        let cstr = CString::new(tz).unwrap();
        timezone_names.get_mut().unwrap().insert(cstr.clone());
        timezone_names.get().unwrap().get(&cstr).unwrap().as_ptr()
    };

    t.tm_zone = tm_zone.cast();
    t
}

unsafe fn set_timezone(
    guard: &mut MutexGuard<'_, (Option<CString>, Option<CString>)>,
    std: &DateTime<Tz>,
    dst: Option<DateTime<Tz>>,
) {
    let ut_offset = std.offset();

    guard.0 = Some(CString::new(ut_offset.abbreviation().expect("Wrong timezone")).unwrap());
    tzname.0[0] = guard.0.as_ref().unwrap().as_ptr().cast_mut();

    match dst {
        Some(dst) => {
            guard.1 =
                Some(CString::new(dst.offset().abbreviation().expect("Wrong timezone")).unwrap());
            tzname.0[1] = guard.1.as_ref().unwrap().as_ptr().cast_mut();
            daylight = 1;
        }
        None => {
            guard.1 = None;
            tzname.0[1] = guard.0.as_ref().unwrap().as_ptr().cast_mut();
            daylight = 0;
        }
    }

    timezone = -c_long::from(ut_offset.fix().local_minus_utc());
}

#[inline(always)]
pub const fn get_offset(off: c_long) -> Option<FixedOffset> {
    if off < 0 {
        FixedOffset::west_opt(off as _)
    } else {
        FixedOffset::east_opt(off as _)
    }
}

const fn blank_tm() -> tm {
    tm {
        tm_year: 0,
        tm_mon: 0,
        tm_mday: 0,
        tm_hour: 0,
        tm_min: 0,
        tm_sec: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: -1,
        tm_gmtoff: 0,
        tm_zone: ptr::null_mut(),
    }
}
