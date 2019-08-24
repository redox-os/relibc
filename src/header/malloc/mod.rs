use core::{convert::TryFrom, ptr};
use crate::{
    header::{
        errno::*,
        unistd::{sysconf, _SC_PAGESIZE},
    },
    platform::{self, types::*},
};

// GNU extension, always declared in malloc.h.
#[no_mangle]
pub unsafe extern "C" fn pvalloc(size: size_t) -> *mut c_void {
    /* The conversion from sysconf(_SC_PAGESIZE) (which is of type
     * c_long) may fail. */
    match size_t::try_from(sysconf(_SC_PAGESIZE)) {
        Ok(page_size) => {
            /* Find the smallest multiple of the page size in which the
             * requested size will fit. We must deal with size 0
             * separately to avoid underflow in (size - 1). The result
             * of the division will always be less than or equal to
             * size_t::max_value() - 1, and the num_pages calculation
             * will therefore never overflow. */
            let num_pages = if size != 0 {
                (size - 1) / page_size + 1
            } else {
                0
            };
            
            match num_pages.checked_mul(page_size) {
                Some(alloc_size) => {
                    // Perform the allocation.
                    let ptr = platform::alloc_align(alloc_size, page_size);
                    if ptr.is_null() {
                        platform::errno = ENOMEM;
                    }
                    ptr
                }
                None => {
                    /* Integer overflow occurred when computing
                     * necessary allocation size. */
                    platform::errno = ENOMEM;
                    ptr::null_mut()
                }
            }
        }
        Err(_) => {
            // Conversion of sysconf(_SC_PAGESIZE) to size_t failed.
            platform::errno = ENOMEM;
            ptr::null_mut()
        }
    }
}
