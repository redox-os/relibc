//! `sys/utsname.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_utsname.h.html>.

// TODO: set this for entire crate when possible
#![deny(unsafe_op_in_unsafe_fn)]

use crate::{
    error::ResultExt,
    out::Out,
    platform::{
        Pal, Sys,
        types::{c_char, c_int},
    },
};

pub const UTSLENGTH: usize = 65;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_utsname.h.html>.
#[repr(C)]
#[derive(Clone, Copy, Debug, OutProject)]
pub struct utsname {
    pub sysname: [c_char; UTSLENGTH],
    pub nodename: [c_char; UTSLENGTH],
    pub release: [c_char; UTSLENGTH],
    pub version: [c_char; UTSLENGTH],
    pub machine: [c_char; UTSLENGTH],
    pub domainname: [c_char; UTSLENGTH],
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/uname.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uname(uts: *mut utsname) -> c_int {
    Sys::uname(unsafe { Out::nonnull(uts) })
        .map(|()| 0)
        .or_minus_one_errno()
}
