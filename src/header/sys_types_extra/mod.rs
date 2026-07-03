//! Non-POSIX extras for `sys/types.h` implementation.

use crate::platform::types::{c_char, c_longlong, c_uchar, c_uint, c_ulong, c_ulonglong, c_ushort};

/// Intended as a convenience type.
pub type u_char = c_uchar;
/// Sys V compatibility type.
pub type uchar = c_uchar;
/// Intended as a convenience type.
pub type u_short = c_ushort;
/// Sys V compatibility type.
pub type ushort = c_ushort;
/// Intended as a convenience type.
pub type u_int = c_uint;
/// Sys V compatibility type.
pub type uint = c_uint;
/// Intended as a convenience type.
pub type u_long = c_ulong;
/// Sys V compatibility type.
pub type ulong = c_ulong;
/// Always 64bit, always `long long`.
pub type quad_t = c_longlong;
/// Always 64bit, always `unsigned long long`.
pub type u_quad_t = c_ulonglong;
/// Legacy BSD type.
pub type caddr_t = *mut c_char;
