//! `ifaddrs.h` implementation

use core::ptr;

use crate::{
    header::{errno, stdlib, sys_socket::sockaddr},
    platform::{self, types::*},
};

#[repr(C)]
union ifaddrs_ifa_ifu {
    ifu_broadaddr: *mut sockaddr,
    ifu_dstaddr: *mut sockaddr,
}

#[repr(C)]
pub struct ifaddrs {
    ifa_next: *mut ifaddrs,
    ifa_name: *mut c_char,
    ifa_flags: c_uint,
    ifa_addr: *mut sockaddr,
    ifa_netmask: *mut sockaddr,
    ifa_ifu: ifaddrs_ifa_ifu,
    ifa_data: *mut c_void,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn freeifaddrs(mut ifa: *mut ifaddrs) {
    while !ifa.is_null() {
        let next = (*ifa).ifa_next;
        stdlib::free(ifa.cast());
        ifa = next;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getifaddrs(ifap: *mut *mut ifaddrs) -> c_int {
    //TODO: implement getifaddrs
    platform::ERRNO.set(errno::ENOSYS);
    -1
}
