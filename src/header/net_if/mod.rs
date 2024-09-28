use core::ptr::null;

use alloc::ffi::CString;

use crate::{
    c_str::CStr,
    platform::{types::*, ERRNO},
};

use super::errno::ENXIO;

#[repr(C)]
pub struct if_nameindex {
    if_index: c_uint,
    if_name: *const c_char,
}

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

/// # Safety
/// Returns a constant pointer to a pre defined const stub list
/// The end of the list is determined by an if_nameindex struct having if_index 0 and if_name NULL
#[no_mangle]
pub unsafe extern "C" fn if_nameindex() -> *const if_nameindex {
    &INTERFACES[0] as *const if_nameindex
}

/// # Safety
/// this is a no-op: the list returned by if_nameindex() is a ref to a constant
#[no_mangle]
pub unsafe extern "C" fn if_freenameindex(s: *mut if_nameindex) {}

/// # Safety
/// Compares the name to a constant string and only returns an int as a result.
/// An invalid name string will return an index of 0
#[no_mangle]
pub unsafe extern "C" fn if_nametoindex(name: *const c_char) -> c_uint {
    if name == null::<c_char>() {
        return 0;
    }
    let name = CStr::from_ptr(name).to_str().unwrap_or("");
    if name.eq("stub") {
        return 1;
    }
    0
}

/// # Safety
/// Returns only static lifetime references to const names, does not reuse the buf pointer.
/// Returns NULL in case of not found + ERRNO being set to ENXIO.
/// Currently only checks against inteface index 1.
#[no_mangle]
pub unsafe extern "C" fn if_indextoname(idx: c_uint, buf: *mut c_char) -> *const c_char {
    if idx == 1 {
        return IF_STUB_INTERFACE;
    }
    ERRNO.set(ENXIO);
    null::<c_char>()
}
