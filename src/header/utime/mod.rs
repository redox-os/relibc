//! utime implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/utime.h.html

use crate::{
    c_str::CStr,
    header::time::timespec,
    platform::{types::*, Pal, Sys},
};

#[repr(C)]
#[derive(Clone)]
pub struct utimbuf {
    pub actime: time_t,
    pub modtime: time_t,
}

#[no_mangle]
pub unsafe extern "C" fn utime(filename: *const c_char, times: *const utimbuf) -> c_int {
    let filename = CStr::from_ptr(filename);
    let times_spec = [
        timespec {
            tv_sec: (*times).actime,
            tv_nsec: 0,
        },
        timespec {
            tv_sec: (*times).modtime,
            tv_nsec: 0,
        },
    ];
    Sys::utimens(filename, times_spec.as_ptr())
}
