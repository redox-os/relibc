//! `mqueue.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/mqueue.h.html>.
#![allow(non_camel_case_types)]

use core::ffi::va_list;

use crate::{
    c_str::CStr,
    header::fcntl::O_CREAT,
    platform::types::{c_char, c_int, c_long},
};

use super::sys_types_internal::mode_t;
pub type mqd_t = c_int;

#[repr(C)]
pub struct mq_attr {
    /// Message queue flags
    pub mq_flags: c_long,
    /// Maximum number of messages
    pub mq_maxmsg: c_long,
    /// Maximum message size
    pub mq_msgsize: c_long,
    /// Number of messages currently queued
    pub mq_curmsgs: c_long,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mq_open.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mq_open(name: *const c_char, flags: c_int, mut rest: ...) -> mqd_t {
    let name = unsafe { CStr::from_ptr(name) }.to_chars_with_nul();
    // POSIX declares that "/foo" refers to the same message queue across all processes, as long as
    // the object exists. That is, a leading "/" makes it a "global" message queue of sorts.
    // POSIX leaves the behavior of the name without a leading "/", and a slash anywhere else in
    // the string, as implementation-defined. Musl treats "/foo" the same as "foo".
    let _name = if name[0] == b'/' { &name[1..] } else { name };
    let (_mode, _attr): (mode_t, *mut mq_attr) = if flags & O_CREAT != 0 {
        let mut ap: va_list::VaList = rest.as_va_list();
        let mode = unsafe { ap.arg::<mode_t>() };
        let attr = unsafe { ap.arg::<*mut mq_attr>() };
        (mode, attr)
    } else {
        (0, core::ptr::null_mut())
    };
    todo!("Add Sys::mq_open and other mqueue methods")
}
