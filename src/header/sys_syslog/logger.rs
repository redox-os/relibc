use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec::Vec,
};
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
    platform::{self, types::*},
    sync::Mutex,
};

use bitflags::bitflags;
use chrono::{DateTime, Utc};

use super::{sys::LogFile, LOG_CONS, LOG_NDELAY, LOG_NOWAIT, LOG_ODELAY, LOG_PERROR, LOG_PID};

pub(super) static LOGGER: Mutex<LogParams<LogFile>> = Mutex::new(LogParams::new(None));

pub struct LogParams<L: LogSink> {
    /// Identity prepended to each log message. POSIX does not specific what to do when it's empty,
    /// but the program name is a common default.
    ident: String,
    pub opt: Config,
    pub facility: c_int,
    pub mask: c_int,
    writer: Option<L>,
}

impl<L: LogSink> LogParams<L> {
    pub const fn new(writer: Option<L>) -> Self {
        LogParams {
            ident: String::new(),
            opt: Config::DelayOpen,
            facility: super::LOG_USER,
            mask: 0xff,
            writer,
        }
    }

    pub fn write_log(&mut self, mut priority: c_int, message: *const c_char, mut ap: VaList) {
        let mut epoch: i64 = 0;
        unsafe {
            epoch = time(null_mut());
        }
        let currtime: DateTime<Utc> =
            DateTime::from_timestamp(epoch, 0).expect("Couldn't retrieve broken-down time.");
        let currtime_s = currtime.format("%b %e %T %Y");
        let pid = self.opt.contains(Config::Pid).then(|| getpid());
        if ((priority & super::LOG_FACMASK) == 0) {
            priority |= self.facility
        };

        // journald from systemd rewrites log messages from syslog into its own style. We'll
        // still use the same style as other libc even though it's implementation specific.
        let mut buffer = if let Some(pid) = pid {
            format!("<{}>{} {}{}: ", priority, currtime_s, self.ident, pid).into_bytes()
        } else {
            format!("<{}>{} {}: ", priority, currtime_s, self.ident).into_bytes()
        };
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
            // SAFETY: `buffer` is a valid byte string that is NUL terminated.
            unsafe {
                fprintf(stderr, c"%s".as_ptr(), buffer.as_ptr() as *const c_char);
            }
        }
    }

    /// Set or clear log identity from a C string.
    ///
    /// Null or empty identities are valid as it just resets the global ident.
    pub fn set_identity_cstr(&mut self, ident: Option<CStr<'_>>) {
        let ident = ident
            .and_then(|ident| (!ident.is_empty()).then(|| ident.to_str().ok()))
            .flatten()
            .map(ToString::to_string);
        self.set_identity(ident);
    }

    /// Set or clear log identity.
    ///
    /// The log identity is prepended to each message. If unset, the program name will be used as a
    /// default.
    pub fn set_identity(&mut self, ident: Option<String>) {
        self.ident = ident.unwrap_or_else(|| {
            unsafe { CStr::from_nullable_ptr(platform::program_invocation_short_name) }
                .and_then(|name| name.to_str().ok())
                .unwrap_or_default()
                .to_owned()
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
