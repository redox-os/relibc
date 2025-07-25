// This is syslog.h implemented based on POSIX.1-2017
// https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/syslog.h.html

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

pub mod logger;

use core::ffi::VaList;

use crate::{c_str::CStr, platform::types::*};
use logger::LOGGER;

// Values for logopt
pub const LOG_PID: c_int = 0x01;
pub const LOG_CONS: c_int = 0x02;
pub const LOG_ODELAY: c_int = 0x04;
pub const LOG_NDELAY: c_int = 0x08;
pub const LOG_NOWAIT: c_int = 0x10;
pub const LOG_PERROR: c_int = 0x20;

// Facilities
// Note: in this case I relied more on MUSL than on POSIX1.2017
// as it appears there were some Linux-specific facilities
// Which could be used by programs we want to port over
// And GNU Libc had these too
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

// Internal constant for extracting facility from a packed facility-priority bitfield.
// TODO: Remove or use a packed i32 like musl.
const LOG_FACMASK: c_int = 0x3f8;

#[no_mangle]
pub extern "C" fn setlogmask(maskpri: c_int) -> c_int {
    let mut params = LOGGER.lock();
    let ret = params.mask;
    if (maskpri != 0) {
        params.mask = maskpri;
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn openlog(ident: *const c_char, opt: c_int, facility: c_int) {
    let ident = unsafe { CStr::from_nullable_ptr(ident) };
    let conf = logger::Config::from_bits_truncate(opt);

    let mut params = LOGGER.lock();
    params.set_identity_cstr(ident);
    params.opt = conf;
    params.facility = facility;

    // Ensure log is ready to write now instead of checking on the first message.
    if conf.contains(logger::Config::NoDelay) {
        params.open_logger();
    }
}

#[no_mangle]
pub unsafe extern "C" fn vsyslog(priority: c_int, message: *const c_char, mut ap: VaList) {
    let mut logger = LOGGER.lock();
    if (((logger.mask & (1 << (priority & 7))) == 0) || ((priority & !0x3ff) != 0)) {
        return;
    };
    logger.write_log(priority, message, ap);
}

#[no_mangle]
pub unsafe extern "C" fn syslog(priority: c_int, message: *const c_char, mut __valist: ...) {
    vsyslog(priority, message, __valist.as_va_list());
}

#[no_mangle]
pub extern "C" fn closelog() {
    LOGGER.lock().close();
}
