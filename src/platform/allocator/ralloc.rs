extern crate ralloc;

pub use ralloc::Allocator;

unsafe fn alloc_inner(size: usize, offset: usize, align: usize) -> *mut c_void {
    let ptr = ralloc::alloc(size + offset, align);
    if !ptr.is_null() {
        *(ptr as *mut u64) = (size + offset) as u64;
        *(ptr as *mut u64).offset(1) = align as u64;
        ptr.offset(offset as isize) as *mut c_void
    } else {
        ptr as *mut c_void
    }
}

pub unsafe fn alloc(size: usize) -> *mut c_void {
    alloc_inner(size, 16, 8)
}

pub unsafe fn alloc_align(size: usize, alignment: usize) -> *mut c_void {
    let mut align = 32;
    while align <= alignment {
        align *= 2;
    }

    alloc_inner(size, align / 2, align)
}

pub unsafe fn realloc(ptr: *mut c_void, size: size_t) -> *mut c_void {
    let old_ptr = (ptr as *mut u8).offset(-16);
    let old_size = *(old_ptr as *mut u64);
    let align = *(old_ptr as *mut u64).offset(1);
    let ptr = ralloc::realloc(old_ptr, old_size as usize, size + 16, align as usize);
    if !ptr.is_null() {
        *(ptr as *mut u64) = (size + 16) as u64;
        *(ptr as *mut u64).offset(1) = align;
        ptr.offset(16) as *mut c_void
    } else {
        ptr as *mut c_void
    }
}

pub unsafe fn free(ptr: *mut c_void) {
    let ptr = (ptr as *mut u8).offset(-16);
    let size = *(ptr as *mut u64);
    let _align = *(ptr as *mut u64).offset(1);
    ralloc::free(ptr, size as usize);
}
