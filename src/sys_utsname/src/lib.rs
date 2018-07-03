//! sys/utsname implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/sysutsname.h.html

#![no_std]

#[cfg(target_os = "linux")]
mod inner {
    extern crate platform;

    use self::platform::types::*;
    use core::ptr;

    const LENGTH: usize = 65;

    #[allow(non_camel_case)]
    #[no_mangle]
    #[repr(C)]
    pub struct utsname {
        pub sysname: [c_char; LENGTH],
        pub nodename: [c_char; LENGTH],
        pub release: [c_char; LENGTH],
        pub version: [c_char; LENGTH],
        pub machine: [c_char; LENGTH],
        pub domainname: [c_char; LENGTH],
    }

    #[no_mangle]
    pub unsafe extern "C" fn uname(uts: *mut utsname) -> c_int {
        platform::uname(uts as usize)
    }
}
#[cfg(target_os = "linux")]
pub use inner::*;
