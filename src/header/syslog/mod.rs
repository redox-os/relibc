// This is syslog.h implemented based on POSIX.1-2017
// https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/syslog.h.html

#[cfg(target_os = "redox")]
pub use self::sys::*;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;
