//! `sys/types.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_types.h.html>.
//!
//! Note that the `useconds_t` type provided in the `sys/types.h` header was
//! removed in the Open Group Base Specifications Issue 7, see
//! <https://pubs.opengroup.org/onlinepubs/009695399/basedefs/sys/types.h.html>
//! for the old specification.

#[expect(deprecated)]
pub use crate::header::bits_useconds_t::useconds_t;
pub use crate::header::{
    bits_clock_t::clock_t,
    bits_clockid_t::clockid_t,
    bits_dev_t::dev_t,
    bits_gid_t::gid_t,
    bits_id_t::id_t,
    bits_ino_t::ino_t,
    bits_key_t::key_t,
    bits_mode_t::mode_t,
    bits_nlink_t::nlink_t,
    bits_off_t::off_t,
    bits_pid_t::pid_t,
    bits_reclen_t::reclen_t,
    bits_size_t::size_t,
    bits_ssize_t::ssize_t,
    bits_suseconds_t::suseconds_t,
    bits_sys_stat::{blkcnt_t, blksize_t},
    bits_sys_statvfs::{fsblkcnt_t, fsfilcnt_t},
    bits_time_t::time_t,
    bits_timer_t::timer_t,
    bits_uid_t::uid_t,
};
use crate::platform::types::{c_char, c_longlong, c_uchar, c_uint, c_ulong, c_ulonglong, c_ushort};

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
