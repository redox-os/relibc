//! `sys/utsname.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_utsname.h.html>.

use crate::{
    error::ResultExt,
    out::Out,
    platform::{
        Pal, Sys,
        types::{c_char, c_int},
    },
};

/// Non-POSIX.
///
/// Length of each character array in `utsname`.
pub const UTSLENGTH: usize = 65;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_utsname.h.html>.
#[repr(C)]
#[derive(Clone, Debug, OutProject)]
pub struct utsname {
    /// Name of this implementation of the operating system.
    pub sysname: [c_char; UTSLENGTH],
    /// Name of this node within the communications network to which this node
    /// is attached, if any.
    pub nodename: [c_char; UTSLENGTH],
    /// Current release level of this implementation.
    pub release: [c_char; UTSLENGTH],
    /// Current version level of this release.
    pub version: [c_char; UTSLENGTH],
    /// Name of the hardware type on which this system is running.
    pub machine: [c_char; UTSLENGTH],
    /// Non-POSIX.
    pub domainname: [c_char; UTSLENGTH],
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/uname.html>.
///
/// Stores information identifying the current system in the structure pointed
/// to by `name`.
///
/// Returns a non-negative value upon success, `-1` upon failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uname(name: *mut utsname) -> c_int {
    Sys::uname(unsafe { Out::nonnull(name) })
        .map(|()| 0)
        .or_minus_one_errno()
}
