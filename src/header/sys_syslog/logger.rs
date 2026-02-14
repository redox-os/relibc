use alloc::{borrow::ToOwned, string::String};
use core::{ffi::VaList, ptr::null_mut};

use crate::{
    c_str::CStr,
    error::Result,
    header::{
        stdio::{fprintf, printf::printf, stderr},
        time::time,
        unistd::getpid,
    },
    io::Write,
    platform::{
        self,
        types::{c_char, c_int},
    },
    sync::Mutex,
};

use bitflags::bitflags;
use chrono::{DateTime, Utc};

use super::{
    LOG_ALERT, LOG_AUTH, LOG_AUTHPRIV, LOG_CONS, LOG_CRIT, LOG_CRON, LOG_DAEMON, LOG_DEBUG,
    LOG_EMERG, LOG_ERR, LOG_FTP, LOG_INFO, LOG_KERN, LOG_LOCAL0, LOG_LOCAL1, LOG_LOCAL2,
    LOG_LOCAL3, LOG_LOCAL4, LOG_LOCAL5, LOG_LOCAL6, LOG_LOCAL7, LOG_LPR, LOG_MAIL, LOG_MASK,
    LOG_NDELAY, LOG_NEWS, LOG_NOTICE, LOG_NOWAIT, LOG_ODELAY, LOG_PERROR, LOG_PID, LOG_SYSLOG,
    LOG_UPTO, LOG_USER, LOG_UUCP, LOG_WARNING, sys::LogFile,
};

pub(super) static LOGGER: Mutex<LogParams<LogFile>> = Mutex::new(LogParams::new(None));

pub(super) struct LogParams<L: LogSink> {
    /// Identity prepended to each log message. POSIX does not specific what to do when it's empty,
    /// but the program name is a common default.
    ident: String,
    pub opt: Config,
    pub mask: Priority,
    writer: Option<L>,
}

impl<L: LogSink> LogParams<L> {
    pub const fn new(writer: Option<L>) -> Self {
        LogParams {
            ident: String::new(),
            opt: Config::DelayOpen,
            mask: Priority::from_bits_truncate(Priority::User.bits() | Priority::UpToDebug.bits()),
            writer,
        }
    }

    pub fn write_log(&mut self, priority: Priority, message: CStr<'_>, ap: VaList) {
        if message.is_empty() {
            return;
        }
        if self.ident.is_empty() {
            self.set_identity(None);
        }

        let epoch = unsafe { time(null_mut()) };
        let currtime: DateTime<Utc> = DateTime::from_timestamp(epoch, 0).unwrap_or_default();
        let currtime_s = currtime.format("%b %e %T %Y");
        let pid = self.opt.contains(Config::Pid).then(|| getpid());

        // journald from systemd rewrites log messages from syslog into its own style. We'll
        // still use the same style as other libc even though it's implementation specific.
        let mut buffer = if let Some(pid) = pid {
            format!(
                "<{}>{} {}{}: ",
                priority.bits(),
                currtime_s,
                self.ident,
                pid
            )
            .into_bytes()
        } else {
            format!("<{}>{} {}: ", priority.bits(), currtime_s, self.ident).into_bytes()
        };
        let prefix = buffer.len();

        // SAFETY:
        // * Assumes caller passed in a valid C string; printf should handle that invariant.
        // * `buffer` grows to fit the formatted string.
        unsafe { printf(&mut buffer, message, ap) };
        buffer.extend(b"\n\0");

        if self.maybe_open_logger().is_ok() {
            if self
                .writer
                .as_mut()
                .map(|w| w.writer().write_all(&buffer).is_err())
                .unwrap_or(true)
            {
                // Try reopening the log file once and retrying as musl does.
                if !self
                    .open_logger()
                    .is_ok()
                    .then(|| {
                        self.writer
                            .as_mut()
                            .map(|w| w.writer().write_all(&buffer).is_ok())
                            .unwrap_or_default()
                    })
                    .unwrap_or_default()
                    && self.opt.contains(Config::Console)
                {
                    // TODO: Log error to /dev/console & Redox equivalent
                }
            }
        }
        if self.opt.contains(Config::PError) {
            // SAFETY:
            // * `ident` is a valid byte string that is NUL terminated when set.
            // * `buffer` is a valid byte string that is NUL terminated above.
            unsafe {
                // musl and glibc only print the message rather than the prefix + message to stderr
                fprintf(
                    stderr,
                    c"%s: %s".as_ptr(),
                    self.ident.as_ptr() as *const c_char,
                    buffer[prefix..].as_ptr() as *const c_char,
                );
            }
        }
    }

    /// Set or clear log identity from a C string.
    ///
    /// Null or empty identities are valid as it just resets the global ident.
    pub fn set_identity_cstr(&mut self, ident: Option<CStr<'_>>) {
        let ident = ident
            .and_then(|ident| (!ident.is_empty()).then(|| ident.to_str().ok()))
            .flatten();
        self.set_identity(ident);
    }

    /// Set or clear log identity.
    ///
    /// The log identity is prepended to each message. If unset, the program name will be used as a
    /// default.
    pub fn set_identity(&mut self, ident: Option<&str>) {
        self.ident = ident
            .and_then(|ident| {
                let ident = ident.bytes().chain([0]).collect();
                // SAFETY: Already validated
                Some(unsafe { String::from_utf8_unchecked(ident) })
            })
            .unwrap_or_else(|| {
                unsafe { CStr::from_nullable_ptr(platform::program_invocation_short_name) }
                    .and_then(|name| {
                        let name = name.to_str().ok()?.bytes().chain([0]).collect();
                        // SAFETY: Validated above
                        Some(unsafe { String::from_utf8_unchecked(name) })
                    })
                    .unwrap_or_else(|| "\0".to_owned())
            });
    }

    /// Open the internal [`LogFile`] if it's not open.
    pub fn maybe_open_logger(&mut self) -> Result<()> {
        if self.writer.is_none() {
            self.open_logger()
        } else {
            Ok(())
        }
    }

    /// Open or reopen the internal [`LogFile`].
    pub fn open_logger(&mut self) -> Result<()> {
        L::open().map(|file| {
            self.writer.replace(file);
        })
    }

    /// Close the open writer to the system logger (optional).
    pub fn close(&mut self) {
        self.writer.take();
    }
}

/// Operating system specific log handling.
pub(super) trait LogSink {
    type Sink: Write;

    fn open() -> Result<Self>
    where
        Self: Sized;

    fn writer(&mut self) -> &mut Self::Sink;
}

bitflags! {
    #[derive(Clone, Copy)]
    pub struct Config: c_int {
        const Pid = LOG_PID;
        const Console = LOG_CONS;
        const DelayOpen = LOG_ODELAY;
        const NoDelay = LOG_NDELAY;
        const NoWait = LOG_NOWAIT;
        const PError = LOG_PERROR;
    }
}

bitflags! {
    /// Packed Facility-Priority bit field.
    #[derive(Clone, Copy)]
    pub struct Priority: c_int {
        const Emerg = LOG_EMERG;
        const Alert = LOG_ALERT;
        const Crit = LOG_CRIT;
        const Err = LOG_ERR;
        const Warn = LOG_WARNING;
        const Notice = LOG_NOTICE;
        const Info = LOG_INFO;
        const Debug = LOG_DEBUG;

        const UpToEmerg = LOG_UPTO(LOG_EMERG);
        const UpToAlert = LOG_UPTO(LOG_ALERT);
        const UpToCrit = LOG_UPTO(LOG_CRIT);
        const UpToErr = LOG_UPTO(LOG_ERR);
        const UpToWarn = LOG_UPTO(LOG_WARNING);
        const UpToNotice = LOG_UPTO(LOG_NOTICE);
        const UpToInfo = LOG_UPTO(LOG_INFO);
        const UpToDebug = LOG_UPTO(LOG_DEBUG);

        const Kern = LOG_KERN;
        const User = LOG_USER;
        const Mail = LOG_MAIL;
        const Daemon = LOG_DAEMON;
        const Auth = LOG_AUTH;
        const Syslog = LOG_SYSLOG;
        const Printer = LOG_LPR;
        const News = LOG_NEWS;
        const UUCP = LOG_UUCP;
        const CRON = LOG_CRON;
        const AuthPriv = LOG_AUTHPRIV;
        const FTP = LOG_FTP;
        const Local0 = LOG_LOCAL0;
        const Local1 = LOG_LOCAL1;
        const Local2 = LOG_LOCAL2;
        const Local3 = LOG_LOCAL3;
        const Local4 = LOG_LOCAL4;
        const Local5 = LOG_LOCAL5;
        const Local6 = LOG_LOCAL6;
        const Local7 = LOG_LOCAL7;

        // Internal constants for extracting facility or priority from a packed bitfield.
        const FacilityMask = 0x3ff;
        const PriorityMask = !Self::FacilityMask.bits();
    }
}

impl Priority {
    /// Keep facility but replace severity mask.
    pub fn with_mask(self, mask: c_int) -> Option<Self> {
        // Fail on invalid bits and drop invalid facility bits.
        let mask = Self::from_bits(mask)? & Self::FacilityMask;
        let facility = self & Self::PriorityMask;
        Some(mask | facility)
    }

    /// Keep mask but replace facility.
    pub fn with_facility(self, facility: c_int) -> Option<Self> {
        // Fail on invalid bits and drop invalid priority bits.
        let facility = Self::from_bits(facility)? & Self::PriorityMask;
        let mask = self & Self::FacilityMask;
        Some(mask | facility)
    }

    /// Returns if a message of `priority` should be retained by this mask.
    pub fn should_log(self, priority: Self) -> bool {
        (self & Self::FacilityMask)
            .contains(Priority::from_bits_truncate(LOG_MASK(priority.bits())))
    }
}
