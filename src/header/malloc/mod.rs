//! `malloc.h` implementation.
//!
//! Non-POSIX, see <https://man7.org/linux/man-pages/man3/posix_memalign.3.html>.

// TODO: set this for entire crate when possible
#![deny(unsafe_op_in_unsafe_fn)]

use crate::{
    header::errno::ENOMEM,
    platform::{
        self, ERRNO, Pal, Sys,
        types::{c_void, size_t},
    },
};
use core::ptr;

/// See <https://man7.org/linux/man-pages/man3/posix_memalign.3.html>.
#[deprecated]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pvalloc(size: size_t) -> *mut c_void {
    let page_size = Sys::getpagesize();
    // Find the smallest multiple of the page size in which the requested size
    // will fit. The result of the division will always be less than or equal
    // to size_t::MAX - 1, and the num_pages calculation will therefore never
    // overflow.
    let num_pages = if size != 0 {
        (size - 1) / page_size + 1
    } else {
        0
    };

    match num_pages.checked_mul(page_size) {
        Some(alloc_size) => {
            let ptr = unsafe { platform::alloc_align(alloc_size, page_size) };
            if ptr.is_null() {
                platform::ERRNO.set(ENOMEM);
            }
            ptr
        }
        None => {
            platform::ERRNO.set(ENOMEM);
            ptr::null_mut()
        }
    }
}

/// See <https://man7.org/linux/man-pages/man3/malloc_usable_size.3.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc_usable_size(ptr: *mut c_void) -> size_t {
    unsafe { platform::alloc_usable_size(ptr) }
}
