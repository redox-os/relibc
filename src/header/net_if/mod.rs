//! `net/if.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/net_if.h.html>.

use core::ptr::null;

use alloc::ffi::CString;

use crate::{
    c_str::CStr,
    platform::{
        ERRNO,
        types::{c_char, c_int, c_uint},
    },
};

use super::errno::ENXIO;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/net_if.h.html>.
#[repr(C)]
pub struct if_nameindex {
    if_index: c_uint,
    if_name: *const c_char,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/net_if.h.html>.
pub const IF_NAMESIZE: usize = 16;

const IF_STUB_INTERFACE: *const c_char = (c"stub").as_ptr();

const INTERFACES: &[if_nameindex] = &[
    if_nameindex {
        if_index: 1,
        if_name: IF_STUB_INTERFACE,
    },
    if_nameindex {
        if_index: 0,
        if_name: null::<c_char>(),
    },
];

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/if_freenameindex.html>.
///
/// # Safety
/// this is a no-op: the list returned by if_nameindex() is a ref to a constant
#[unsafe(no_mangle)]
pub unsafe extern "C" fn if_freenameindex(s: *mut if_nameindex) {}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/if_indextoname.html>.
///
/// # Safety
/// Returns only static lifetime references to const names, does not reuse the buf pointer.
/// Returns NULL in case of not found + ERRNO being set to ENXIO.
/// Currently only checks against inteface index 1.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn if_indextoname(idx: c_uint, buf: *mut c_char) -> *const c_char {
    if idx == 1 {
        return IF_STUB_INTERFACE;
    }
    ERRNO.set(ENXIO);
    null::<c_char>()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/if_nameindex.html>.
///
/// # Safety
/// Returns a constant pointer to a pre defined const stub list
/// The end of the list is determined by an if_nameindex struct having if_index 0 and if_name NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn if_nameindex() -> *const if_nameindex {
    &INTERFACES[0] as *const if_nameindex
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/if_nametoindex.html>.
///
/// # Safety
/// Compares the name to a constant string and only returns an int as a result.
/// An invalid name string will return an index of 0
#[unsafe(no_mangle)]
pub unsafe extern "C" fn if_nametoindex(name: *const c_char) -> c_uint {
    if name == null::<c_char>() {
        return 0;
    }
    let name = unsafe { CStr::from_ptr(name).to_str().unwrap_or("") };
    if name.eq("stub") {
        return 1;
    }
    0
}

// Nonstandard, used alongside ifaddrs.h
// See https://man7.org/linux/man-pages/man7/netdevice.7.html

pub const IFF_UP: c_int = 0x1;
pub const IFF_BROADCAST: c_int = 0x2;
pub const IFF_DEBUG: c_int = 0x4;
pub const IFF_LOOPBACK: c_int = 0x8;
pub const IFF_POINTOPOINT: c_int = 0x10;
pub const IFF_NOTRAILERS: c_int = 0x20;
pub const IFF_RUNNING: c_int = 0x40;
pub const IFF_NOARP: c_int = 0x80;
pub const IFF_PROMISC: c_int = 0x100;
pub const IFF_ALLMULTI: c_int = 0x200;
pub const IFF_MASTER: c_int = 0x400;
pub const IFF_SLAVE: c_int = 0x800;
pub const IFF_MULTICAST: c_int = 0x1000;
pub const IFF_PORTSEL: c_int = 0x2000;
pub const IFF_AUTOMEDIA: c_int = 0x4000;
pub const IFF_DYNAMIC: c_int = 0x8000;
pub const IFF_LOWER_UP: c_int = 0x10000;
pub const IFF_DORMANT: c_int = 0x20000;
pub const IFF_ECHO: c_int = 0x40000;
pub const IFF_VOLATILE: c_int = (IFF_LOOPBACK
    | IFF_POINTOPOINT
    | IFF_BROADCAST
    | IFF_ECHO
    | IFF_MASTER
    | IFF_SLAVE
    | IFF_RUNNING
    | IFF_LOWER_UP
    | IFF_DORMANT);
