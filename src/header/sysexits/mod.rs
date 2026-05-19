//! `sysexits.h` implementation.
//!
//! Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/sysexits.h.3head.html>.

use crate::platform::types::c_int;

/// Successful termination.
pub const EX_OK: c_int = 0;
/// Command line usage error.
pub const EX_USAGE: c_int = 64;
/// Data format error.
pub const EX_DATAERR: c_int = 65;
/// Cannot open input.
pub const EX_NOINPUT: c_int = 66;
/// Addressee unknown.
pub const EX_NOUSER: c_int = 67;
/// Host name unknown.
pub const EX_NOHOST: c_int = 68;
/// Service unavailable.
pub const EX_UNAVAILABLE: c_int = 69;
/// Internal software error.
pub const EX_SOFTWARE: c_int = 70;
/// System error (e.g., can't fork).
pub const EX_OSERR: c_int = 71;
/// Critical OS file missing.
pub const EX_OSFILE: c_int = 72;
/// Can't create (user) output file.
pub const EX_CANTCREAT: c_int = 73;
/// Input/Output error.
pub const EX_IOERR: c_int = 74;
/// Temp failure; user is invited to retry.
pub const EX_TEMPFAIL: c_int = 75;
/// Remote error in protocol.
pub const EX_PROTOCOL: c_int = 76;
/// Permission denied.
pub const EX_NOPERM: c_int = 77;
/// Configuration error.
pub const EX_CONFIG: c_int = 78;
