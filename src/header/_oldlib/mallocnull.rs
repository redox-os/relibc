use core::mem;
use libc::{c_void, size_t};

/// Malloc memory if pointer is null, and free on drop
pub struct MallocNull<T> {
    value: *mut T,
    free_on_drop: bool
}

impl<T> MallocNull<T> {
    pub fn new(ptr: *mut T, size: usize) -> MallocNull<T> {
        if ptr.is_null() {
            MallocNull {
                value: unsafe { ::malloc(size as size_t) as *mut T },
                free_on_drop: true
            }
	    } else {
            MallocNull {
                value: ptr,
                free_on_drop: false
            }
        }
    }
    
    pub fn into_raw(self) -> *mut T {
        let ptr = self.value;
        mem::forget(self);
        ptr
    }

    pub fn as_ptr(&self) -> *const T {
        self.value
    }

    pub fn as_mut_ptr(&self) -> *mut T {
        self.value
    }
}

impl<T> Drop for MallocNull<T> {
    fn drop(&mut self) {
        if self.free_on_drop {
            unsafe { ::free(self.value as *mut c_void) };
        }
    }
}
