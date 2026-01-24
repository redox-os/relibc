use core::{fmt, str::FromStr};

use crate::{
    c_str::CStr,
    fs::{self, File},
    header::fcntl,
    io::{self, BufWriter, prelude::*},
    sync::Mutex,
};

use alloc::string::String;
use log::{Metadata, Record};

const DEFAULT_LOG_LEVEL: log::LevelFilter = log::LevelFilter::Info;

pub unsafe fn init() {
    let mut logger = RedoxLogger::new();
    let log_env = c"RELIBC_LOG_LEVEL".as_ptr();
    #[cfg(feature = "no_trace")]
    let mut trace_warn = false;
    unsafe {
        if let Some(env) = CStr::from_nullable_ptr(crate::header::stdlib::getenv(log_env)) {
            if let Ok(level) = log::LevelFilter::from_str(env.to_str().unwrap_or("")) {
                #[cfg(feature = "no_trace")]
                if level == log::LevelFilter::Trace {
                    trace_warn = true;
                }

                logger = logger.with_output(OutputBuilder::stderr().with_filter(level).build());
            }
        }
    }
    if logger.enable().is_err() {
        log::error!("Logger already initialized");
    }

    #[cfg(feature = "no_trace")]
    if trace_warn {
        log::warn!(
            "The 'no_trace' feature is enabled but RELIBC_LOG_LEVEL=TRACE, there will be no trace logs"
        );
    }
}

/// Copied from redox_log crate with some modifications, in future we might use it instead?
/// An output that will be logged to. The two major outputs for most Redox system programs are
/// usually the log file, and the global stdout.
pub struct Output {
    // the actual endpoint to write to.
    endpoint: Mutex<Box<dyn fmt::Write + Send + 'static>>,

    // useful for devices like BufWrite or BufRead. You don't want the log file to never but
    // written until the program exists.
    flush_on_newline: bool,

    // specifies the maximum log level possible
    filter: log::LevelFilter,
}

impl fmt::Debug for Output {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Output")
            .field("endpoint", &"opaque")
            .field("flush_on_newline", &self.flush_on_newline)
            .field("filter", &self.filter)
            .finish()
    }
}

impl Default for Output {
    fn default() -> Self {
        // Uses default level of max_level_in_use == None  a.k.a LogLevel::Info
        OutputBuilder::stderr().build()
    }
}

pub struct OutputBuilder {
    endpoint: Box<dyn fmt::Write + Send + 'static>,
    flush_on_newline: Option<bool>,
    filter: Option<log::LevelFilter>,
    ansi: Option<bool>,
}
impl OutputBuilder {
    /*
    pub fn in_redox_logging_scheme<A, B, C>(
        category: A,
        subcategory: B,
        logfile: C,
    ) -> Result<Self, io::Error>
    where
        A: AsRef<OsStr>,
        B: AsRef<OsStr>,
        C: AsRef<OsStr>,
    {
        if !cfg!(target_os = "redox") {
            return Ok(Self::with_endpoint(Vec::new()));
        }

        let mut path = PathBuf::from("/scheme/logging/");
        path.push(category.as_ref());
        path.push(subcategory.as_ref());
        path.push(logfile.as_ref());
        path.set_extension("log");

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        Ok(Self::with_endpoint(BufWriter::new(File::create(
            path,
            fcntl::O_CREAT | fcntl::O_CLOEXEC,
            0,
        )?)))
    }
         */
    pub fn stdout() -> Self {
        Self::with_endpoint(crate::platform::FileWriter::new(1))
    }
    pub fn stderr() -> Self {
        Self::with_endpoint(crate::platform::FileWriter::new(2))
    }

    pub fn with_endpoint<T>(endpoint: T) -> Self
    where
        T: fmt::Write + Send + 'static,
    {
        Self::with_dyn_endpoint(Box::new(endpoint))
    }
    pub fn with_dyn_endpoint(endpoint: Box<dyn fmt::Write + Send + 'static>) -> Self {
        Self {
            endpoint,
            flush_on_newline: None,
            filter: None,
            ansi: None,
        }
    }
    pub fn flush_on_newline(mut self, flush: bool) -> Self {
        self.flush_on_newline = Some(flush);
        self
    }
    pub fn with_filter(mut self, filter: log::LevelFilter) -> Self {
        self.filter = Some(filter);
        self
    }
    pub fn build(self) -> Output {
        Output {
            endpoint: Mutex::new(self.endpoint),
            filter: self.filter.unwrap_or(DEFAULT_LOG_LEVEL),
            flush_on_newline: self.flush_on_newline.unwrap_or(true),
        }
    }
}

#[derive(Debug, Default)]
pub struct RedoxLogger {
    output: Output,
    min_filter: Option<log::LevelFilter>,
    max_filter: Option<log::LevelFilter>,
    max_level_in_use: Option<log::LevelFilter>,
    min_level_in_use: Option<log::LevelFilter>,
    process_name: Option<String>,
}

impl RedoxLogger {
    pub fn new() -> Self {
        Self::default()
    }
    fn adjust_output_level(
        max_filter: Option<log::LevelFilter>,
        min_filter: Option<log::LevelFilter>,
        max_in_use: &mut Option<log::LevelFilter>,
        min_in_use: &mut Option<log::LevelFilter>,
        output: &mut Output,
    ) {
        if let Some(max) = max_filter {
            output.filter = core::cmp::max(output.filter, max);
        }
        if let Some(min) = min_filter {
            output.filter = core::cmp::min(output.filter, min);
        }
        match max_in_use {
            &mut Some(ref mut max) => *max = core::cmp::max(output.filter, *max),
            max @ &mut None => *max = Some(output.filter),
        }
        match min_in_use {
            &mut Some(ref mut min) => *min = core::cmp::min(output.filter, *min),
            min @ &mut None => *min = Some(output.filter),
        }
    }
    pub fn with_output(mut self, mut output: Output) -> Self {
        Self::adjust_output_level(
            self.max_filter,
            self.min_filter,
            &mut self.max_level_in_use,
            &mut self.min_level_in_use,
            &mut output,
        );
        self.output = output;
        self
    }
    pub fn with_min_level_override(mut self, min: log::LevelFilter) -> Self {
        self.min_filter = Some(min);
        let output = &mut self.output;
        Self::adjust_output_level(
            self.max_filter,
            self.min_filter,
            &mut self.max_level_in_use,
            &mut self.min_level_in_use,
            output,
        );
        self
    }
    pub fn with_max_level_override(mut self, max: log::LevelFilter) -> Self {
        self.max_filter = Some(max);
        let output = &mut self.output;
        Self::adjust_output_level(
            self.max_filter,
            self.min_filter,
            &mut self.max_level_in_use,
            &mut self.min_level_in_use,
            output,
        );
        self
    }
    pub fn with_process_name(mut self, name: String) -> Self {
        self.process_name = Some(name);
        self
    }
    pub fn enable(self) -> Result<&'static Self, log::SetLoggerError> {
        let leak = Box::leak(Box::new(self));
        log::set_logger(leak)?;
        if let Some(max) = leak.max_level_in_use {
            log::set_max_level(max);
        } else {
            log::set_max_level(DEFAULT_LOG_LEVEL);
        }
        Ok(leak)
    }
    fn write_record<W: fmt::Write + ?Sized>(
        record: &Record,
        process_name: Option<&str>,
        writer: &mut W,
    ) -> fmt::Result {
        use log::Level;

        let target = record.module_path().unwrap_or(record.target());
        let level = record.level();
        let message = record.args();

        let show_lines = true;
        let line_number = if show_lines { record.line() } else { None };

        let process_name = process_name.unwrap_or("");
        let line = &LineFmt(line_number);
        writeln!(writer, "[{process_name}@{target}{line} {level}] {message}",)
    }
}

impl log::Log for RedoxLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.max_level_in_use
            .map(|min| metadata.level() >= min)
            .unwrap_or(false)
            && self
                .min_level_in_use
                .map(|max| metadata.level() <= max)
                .unwrap_or(false)
    }
    fn log(&self, record: &Record) {
        let output = &self.output;
        if record.metadata().level() <= output.filter {
            let mut endpoint_guard = output.endpoint.lock();

            let _ = Self::write_record(
                record,
                self.process_name.as_deref(),
                endpoint_guard.as_mut(),
            );
        }
    }
    fn flush(&self) {
        // no-op
    }
}

struct LineFmt(Option<u32>);
impl fmt::Display for LineFmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(line) = self.0 {
            write!(f, ":{line}")
        } else {
            write!(f, "")
        }
    }
}
