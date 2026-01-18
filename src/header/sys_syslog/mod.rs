//! `syslog.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/syslog.h.html>.

// Exported as both syslog.h and sys/syslog.h.

// TODO: set this for entire crate when possible
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

pub mod logger;

use core::ffi::VaList;

use crate::{
    c_str::CStr,
    platform::types::{c_char, c_int},
};
use logger::{LOGGER, Priority};

/// Record the caller's PID in log messages.
pub const LOG_PID: c_int = 0x01;
/// Write to /dev/console if [`syslog`] fails.
pub const LOG_CONS: c_int = 0x02;
/// Open the log on the first call to [`syslog`] rather than opening it early.
/// This is the default behavior and setting or unsetting this option does nothing.
pub const LOG_ODELAY: c_int = 0x04;
/// Open the log file immediately.
pub const LOG_NDELAY: c_int = 0x08;
pub const LOG_NOWAIT: c_int = 0x10;
/// Print log message to stderr as well as the log.
pub const LOG_PERROR: c_int = 0x20;

pub const LOG_KERN: c_int = 0 << 3;
pub const LOG_USER: c_int = 1 << 3;
pub const LOG_MAIL: c_int = 2 << 3;
pub const LOG_DAEMON: c_int = 3 << 3;
pub const LOG_AUTH: c_int = 4 << 3;
pub const LOG_SYSLOG: c_int = 5 << 3;
pub const LOG_LPR: c_int = 6 << 3;
pub const LOG_NEWS: c_int = 7 << 3;
pub const LOG_UUCP: c_int = 8 << 3;
pub const LOG_CRON: c_int = 9 << 3;
pub const LOG_AUTHPRIV: c_int = 10 << 3;
pub const LOG_FTP: c_int = 11 << 3;

pub const LOG_LOCAL0: c_int = 16 << 3;
pub const LOG_LOCAL1: c_int = 17 << 3;
pub const LOG_LOCAL2: c_int = 18 << 3;
pub const LOG_LOCAL3: c_int = 19 << 3;
pub const LOG_LOCAL4: c_int = 20 << 3;
pub const LOG_LOCAL5: c_int = 21 << 3;
pub const LOG_LOCAL6: c_int = 22 << 3;
pub const LOG_LOCAL7: c_int = 23 << 3;
pub const LOG_NFACILITIES: c_int = 24;

// Priorities
pub const LOG_EMERG: c_int = 0;
pub const LOG_ALERT: c_int = 1;
pub const LOG_CRIT: c_int = 2;
pub const LOG_ERR: c_int = 3;
pub const LOG_WARNING: c_int = 4;
pub const LOG_NOTICE: c_int = 5;
pub const LOG_INFO: c_int = 6;
pub const LOG_DEBUG: c_int = 7;

/// Create a mask that includes all levels up to a certain priority.
#[unsafe(no_mangle)]
pub const extern "C" fn LOG_UPTO(p: c_int) -> c_int {
    (1 << (p + 1)) - 1
}

/// Create a mask that enables a single priority.
#[unsafe(no_mangle)]
pub const extern "C" fn LOG_MASK(p: c_int) -> c_int {
    1 << p
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/closelog.html>.
#[unsafe(no_mangle)]
pub extern "C" fn setlogmask(mask: c_int) -> c_int {
    let mut params = LOGGER.lock();
    let old = params.mask.bits();
    if (mask != 0) {
        if let Some(mask) = params.mask.with_mask(mask) {
            params.mask = mask;
        }
    }
    old
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/closelog.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openlog(ident: *const c_char, opt: c_int, facility: c_int) {
    let ident = unsafe { CStr::from_nullable_ptr(ident) };
    let conf = logger::Config::from_bits_truncate(opt);

    let mut params = LOGGER.lock();
    params.set_identity_cstr(ident);
    params.opt = conf;
    params.mask = params.mask.with_facility(facility).unwrap_or(
        params
            .mask
            .with_facility(Priority::User.bits())
            .expect("`User` is a valid syslog facility"),
    );

    // Ensure log is ready to write now instead of checking on the first message.
    if conf.contains(logger::Config::NoDelay) {
        params.open_logger();
    }
}

/// See <https://www.man7.org/linux/man-pages/man3/vsyslog.3.html>.
///
/// Non-POSIX, 4.3BSD-Reno.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vsyslog(priority: c_int, message: *const c_char, mut ap: VaList) {
    let Some(message) = (unsafe { CStr::from_nullable_ptr(message) }) else {
        return;
    };
    let Some(priority) = Priority::from_bits(priority) else {
        return;
    };

    let mut logger = LOGGER.lock();
    if logger.mask.should_log(priority) {
        logger.write_log(priority, message, ap);
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/closelog.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syslog(priority: c_int, message: *const c_char, mut __valist: ...) {
    unsafe { vsyslog(priority, message, __valist.as_va_list()) };
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/closelog.html>.
#[unsafe(no_mangle)]
pub extern "C" fn closelog() {
    LOGGER.lock().close();
}
