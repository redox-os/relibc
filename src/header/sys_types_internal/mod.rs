//! `sys/types.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_types.h.html>.
//!
//! Note that the `useconds_t` type provided in the `sys/types.h` header was
//! removed in the Open Group Base Specifications Issue 7, see
//! <https://pubs.opengroup.org/onlinepubs/009695399/basedefs/sys/types.h.html>
//! for the old specification.

use crate::platform::types::{c_char, c_int, c_long, c_longlong, c_uchar, c_uint, c_ulong, c_ulonglong, c_ushort};

/// Used for block sizes.
pub type blksize_t = c_long;
/// Used for device IDs.
pub type dev_t = c_ulonglong;
/// Used for serial numbers.
pub type ino_t = c_ulonglong;
/// Used for directory entry lengths.
pub type reclen_t = c_ushort;
/// Used for group IDs.
pub type gid_t = c_int;
/// Used for user IDs.
pub type uid_t = c_int;
/// Used for some file attributes.
pub type mode_t = c_int;
/// Used for link counts.
pub type nlink_t = c_ulong;
/// Used for file sizes.
pub type off_t = c_longlong;
/// Used for process IDs and process group IDs.
pub type pid_t = c_int;
/// Used as a general identifier; can be used to contain at least a pid_t, uid_t, or gid_t.
pub type id_t = c_uint;
/// Used for a count of bytes or an error indication.
pub type ssize_t = c_long;
/// Used for time in seconds.
pub type time_t = c_longlong;
pub type useconds_t = c_uint;

#[cfg(target_os = "linux")]
/// Used for time in microseconds.
pub type suseconds_t = c_long;
#[cfg(not(target_os = "linux"))]
/// Used for time in microseconds.
pub type suseconds_t = c_int;

/// Used for system times in clock ticks or CLOCKS_PER_SEC.
pub type clock_t = c_long;
/// Used for clock ID type in the clock and timer functions.
pub type clockid_t = c_int;

// timer_t in cbindgen after_includes (how to export void* type?)

/// Used for file block counts.
pub type blkcnt_t = c_longlong;

/// Used for file system block counts.
pub type fsblkcnt_t = c_ulong;
/// Used for file system file counts.
pub type fsfilcnt_t = c_ulong;

pub type u_char = c_uchar;
pub type uchar = c_uchar;
pub type u_short = c_ushort;
pub type ushort = c_ushort;
pub type u_int = c_uint;
pub type uint = c_uint;
pub type u_long = c_ulong;
pub type ulong = c_ulong;
pub type quad_t = c_longlong;
pub type u_quad_t = c_ulonglong;
pub type caddr_t = *mut c_char;
