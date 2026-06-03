//! `ifaddrs.h` implementation.
//!
//! Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/getifaddrs.3.html>.

use crate::{
    header::{errno, stdlib, sys_socket::sockaddr},
    platform::{
        self,
        types::{c_char, c_int, c_uint, c_void},
    },
};

/// Either the broadcast address associated with `ifa_addr` (if applicable
/// for the address family) or the destination address of the
/// point-to-point interface.
#[repr(C)]
union ifaddrs_ifa_ifu {
    /// Broadcast address of interface.
    ifu_broadaddr: *mut sockaddr,
    /// Point-to-point destination address.
    ifu_dstaddr: *mut sockaddr,
}

/// An entry in a linked list describing the network interfaces of the local
/// system.
#[repr(C)]
pub struct ifaddrs {
    /// Next item in list.
    ifa_next: *mut ifaddrs,
    /// Name of interface.
    ifa_name: *mut c_char,
    /// Flags from `SIOCGIFFLAGS`.
    ifa_flags: c_uint,
    /// Address of interface.
    ifa_addr: *mut sockaddr,
    /// Netmask of interface.
    ifa_netmask: *mut sockaddr,
    /// Depends on the bit `IFF_BROADCAST` or `IFF_POINTOPOINT` being set in
    /// `ifa_flags`. The bits are mutually exclusive.
    ifa_ifu: ifaddrs_ifa_ifu,
    /// Address-specific data.
    ifa_data: *mut c_void,
}

/// Frees the dynamically allocated memory used by `ifa`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn freeifaddrs(mut ifa: *mut ifaddrs) {
    while !ifa.is_null() {
        let next = unsafe { (*ifa).ifa_next };
        unsafe { stdlib::free(ifa.cast()) };
        ifa = next;
    }
}

/// Creates a linked list of structures describing the network interfaces of
/// the local system, and stores the address of the first item of the list
/// in `ifap`.
///
/// The data returned by `getifaddrs()` is dynamically allocated and should
/// be freed using `freeifaddrs()` when no longer needed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getifaddrs(ifap: *mut *mut ifaddrs) -> c_int {
    //TODO: implement getifaddrs
    platform::ERRNO.set(errno::ENOSYS);
    -1
}
