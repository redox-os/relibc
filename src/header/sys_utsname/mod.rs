//! sys/utsname implementation, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/sysutsname.h.html

use crate::{
    error::ResultExt,
    out::Out,
    platform::{Pal, Sys, types::*},
};

pub const UTSLENGTH: usize = 65;

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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uname(uts: *mut utsname) -> c_int {
    Sys::uname(Out::nonnull(uts))
        .map(|()| 0)
        .or_minus_one_errno()
}
