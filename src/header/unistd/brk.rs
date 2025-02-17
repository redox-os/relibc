use core::ptr;

use crate::{
    error::ResultExtPtrMut,
    header::errno::ENOMEM,
    platform::{self, types::*, Pal, Sys},
};

static mut BRK: *mut c_void = ptr::null_mut();

/// See <https://pubs.opengroup.org/onlinepubs/7908799/xsh/brk.html>.
///
/// # Deprecation
/// The `brk()` function was marked legacy in the System Interface & Headers
/// Issue 5, and removed in Issue 6.
#[deprecated]
#[no_mangle]
pub unsafe extern "C" fn brk(addr: *mut c_void) -> c_int {
    let brk_val = unsafe { &mut BRK };

    *brk_val = unsafe { Sys::brk(addr) }.or_errno_null_mut();

    if *brk_val < addr {
        platform::ERRNO.set(ENOMEM);
        return -1;
    }

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/7908799/xsh/brk.html>.
///
/// # Deprecation
/// The `sbrk()` function was marked legacy in the System Interface & Headers
/// Issue 5, and removed in Issue 6.
#[deprecated]
#[no_mangle]
pub unsafe extern "C" fn sbrk(incr: intptr_t) -> *mut c_void {
    let brk_val = unsafe { &mut BRK };

    if brk_val.is_null() {
        *brk_val = unsafe { Sys::brk(ptr::null_mut()) }.or_errno_null_mut();
    }

    let old_brk = *brk_val;

    if incr != 0 {
        let addr = unsafe { old_brk.offset(incr) };

        *brk_val = unsafe { Sys::brk(addr) }.or_errno_null_mut();

        if *brk_val < addr {
            platform::ERRNO.set(ENOMEM);
            return -1isize as *mut c_void;
        }
    }

    old_brk as *mut c_void
}
