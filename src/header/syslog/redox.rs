use chrono::{DateTime, Utc};
use crate::{
c_str::CStr,
    error::Errno,
    fs::File,
io::Write,
    header::{
        fcntl,
        stdio::printf::printf,
        string::{strlen, strncpy},
        unistd::{getpid},
        time::time,
    },
    platform::types::*,
    sync::Mutex,
};
use core:: {
    ffi::VaList,
    ptr::null_mut,
};

// Values for logopt
pub const LOG_PID: c_int = 0x01;
pub const LOG_CONS: c_int = 0x02;
pub const LOG_ODELAY: c_int = 0x04;
pub const LOG_NDELAY: c_int = 0x08;
pub const LOG_NOWAIT: c_int = 0x10;

// Facilities
// Note: in this case I relied more on MUSL than on POSIX1.2017
// as it appears there were some Linux-specific facilities
// Which could be used by programs we want to port over
// And GNU Libc had these too
pub const LOG_KERN: c_int = 0<<3;
pub const LOG_USER: c_int = 1<<3;
pub const LOG_MAIL: c_int = 2<<3;
pub const LOG_DAEMON: c_int = 3<<3;
pub const LOG_AUTH: c_int = 4<<3;
pub const LOG_SYSLOG: c_int = 5<<3;
pub const LOG_LPR: c_int = 6<<3;
pub const LOG_NEWS: c_int = 7<<3;
pub const LOG_UUCP: c_int = 8<<3;
pub const LOG_CRON: c_int = 9<<3;
pub const LOG_AUTHPRIV: c_int = 10<<3;
pub const LOG_FTP: c_int = 11<<3;
pub const LOG_LOCAL0: c_int = 16<<3;
pub const LOG_LOCAL1: c_int = 17<<3;
pub const LOG_LOCAL2: c_int = 18<<3;
pub const LOG_LOCAL3: c_int = 19<<3;
pub const LOG_LOCAL4: c_int = 20<<3;
pub const LOG_LOCAL5: c_int = 21<<3;
pub const LOG_LOCAL6: c_int = 22<<3;
pub const LOG_LOCAL7: c_int = 23<<3;
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

pub const LOG_FACMASK: c_int = 0x3f8;

enum LogStatus {
    Open{
        filehandle: File,
    },
    Closed,
}

pub struct LogFile {
    status: LogStatus
}

static lock: Mutex<Option<LogFile>> = Mutex::new(Some(LogFile {status: LogStatus::Closed }));
static mut log_ident: &str = "";
static mut log_opt: i32 = 0;
static mut log_facility: i32 = LOG_USER;
static mut log_mask: i32 = 0xff;

#[no_mangle]
pub unsafe extern "C" fn setlogmask(maskpri: c_int) -> c_int {
let ret = log_mask;
    if (maskpri != 0) {
        log_mask = maskpri;
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn openlog(ident: *const c_char, opt: c_int, facility: c_int) {
let conv_ident = CStr::from_ptr(ident);
let new_ident = conv_ident.to_str().unwrap();
if new_ident.is_empty() == false {
log_ident = new_ident;
}
log_opt = opt;
log_facility = facility;

if ((opt & LOG_NDELAY) != 0) {
let mut guard = lock.lock();
match guard.as_mut() {
Some(logfile) => __openlog(logfile),
None => (),
}
}
}

fn __openlog(logfile: &mut LogFile) {
match &mut logfile.status {
LogStatus::Closed => {
let log_file = File::open(c_str!("/scheme/log"), fcntl::O_WRONLY);
match log_file {
Ok(file) => logfile.status = LogStatus::Open{filehandle: file},
Err(e) => {
},
}
}
LogStatus::Open{filehandle} => (),
}
}

unsafe fn _vsyslog(mut priority: i32, message: *const c_char, mut ap: VaList) {
let mut guard = lock.lock();
match guard.as_mut() {
Some (logfile) => __openlog(logfile),
None => (),
}
//Note: trait Local not available due to a dependency loop, so we have to query the time differently
let epoch: i64 = time(null_mut());
let currtime: DateTime<Utc> = DateTime::from_timestamp(epoch, 0).expect("Couldn't retrieve broken-down time.");
let currtime_s = currtime.format("%b %e %T %Y");
let pid = if (log_opt & LOG_PID) != 0 { getpid() } else { 0 };
if ((priority & LOG_FACMASK) == 0) {priority |= log_facility};
let mut final_logmsg = format!("<{}>{} {}{}: ", priority, currtime_s, log_ident, pid);
match guard.as_mut() {
Some(logfile) => {
match &mut logfile.status {
LogStatus::Open{ref mut filehandle} => {
filehandle.write(final_logmsg.as_bytes());
let _ = printf(&mut *filehandle, message, ap);
filehandle.write("\n".as_bytes());
},
_ => (),
}
},
None => (),
}
}

#[no_mangle]
pub unsafe extern "C" fn vsyslog(priority: c_int, message: *const c_char, mut ap: VaList) {
if (((log_mask & (1<<(priority&7))) == 0) || ((priority&!0x3ff) != 0)) {return ()};
_vsyslog(priority, message, ap);
}

#[no_mangle]
pub unsafe extern "C" fn syslog(priority: c_int, message: *const c_char, mut __valist: ...) {
vsyslog(priority, message, __valist.as_va_list());
}

#[no_mangle]
pub extern "C" fn closelog() {
let mut guard = lock.lock();
match guard.as_mut() {
Some(logfile) => {
match &logfile.status {
LogStatus::Open{filehandle} => {
logfile.status = LogStatus::Closed;
}
LogStatus::Closed => (),
}
},
None => (),
}
}
