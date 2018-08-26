use core::alloc::{GlobalAlloc, Layout};

use types::*;

extern "C" {
    fn dlmalloc(bytes: size_t) -> *mut c_void;
    fn dlmemalign(alignment: size_t, bytes: size_t) -> *mut c_void;
    fn dlrealloc(oldmem: *mut c_void, bytes: size_t) -> *mut c_void;
    fn dlfree(mem: *mut c_void);
}

pub struct Allocator;

unsafe impl<'a> GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        dlmemalign(layout.align(), layout.size()) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        dlfree(ptr as *mut c_void)
    }
}

pub unsafe fn alloc(size: usize) -> *mut c_void {
    dlmalloc(size)
}

pub unsafe fn alloc_align(size: usize, alignment: usize) -> *mut c_void {
    dlmemalign(alignment, size)
}

pub unsafe fn realloc(ptr: *mut c_void, size: size_t) -> *mut c_void {
    dlrealloc(ptr, size)
}

pub unsafe fn free(ptr: *mut c_void) {
    dlfree(ptr)
}
