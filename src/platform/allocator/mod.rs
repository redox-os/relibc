use core::{
    alloc::{GlobalAlloc, Layout},
    cell::SyncUnsafeCell,
    cmp,
    mem::align_of,
    ptr::{self, copy_nonoverlapping, write_bytes},
    sync::atomic::{AtomicPtr, Ordering},
};

mod sys;
use super::types::*;
use crate::{ALLOCATOR, sync::Mutex};
use dlmalloc::DlmallocCApi;

pub type Dlmalloc = DlmallocCApi<sys::System>;

pub const NEWALLOCATOR: Allocator = Allocator::new();

pub struct Allocator {
    inner: SyncUnsafeCell<Mutex<Dlmalloc>>,
    pub ptr: AtomicPtr<Mutex<Dlmalloc>>,
}

impl Allocator {
    pub const fn new() -> Self {
        Allocator {
            inner: SyncUnsafeCell::new(Mutex::new(Dlmalloc::new(sys::System::new()))),
            ptr: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn get(&self) -> *const Mutex<Dlmalloc> {
        let ptr = self.ptr.load(Ordering::Acquire);
        if !ptr.is_null() {
            return ptr;
        }

        self.inner.get()
    }

    pub fn set(&self, mspace: *const Mutex<Dlmalloc>) {
        self.ptr.store(mspace.cast_mut(), Ordering::Release);
    }
}

unsafe impl GlobalAlloc for Allocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= align_of::<max_align_t>() {
            unsafe { (*self.get()).lock().malloc(layout.size()) }
        } else {
            unsafe { (*self.get()).lock().memalign(layout.align(), layout.size()) }
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { (*self.get()).lock().free(ptr) }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.alloc(layout) };
        if !ptr.is_null() && unsafe { (*self.get()).lock().calloc_must_clear(ptr) } {
            unsafe { write_bytes(ptr, 0, layout.size()) };
        }
        ptr
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if layout.align() <= align_of::<max_align_t>() {
            unsafe { (*self.get()).lock().realloc(ptr, new_size) }
        } else {
            let new =
                unsafe { self.alloc(Layout::from_size_align_unchecked(new_size, layout.align())) };
            let old_size = layout.size();
            let old_align = layout.align();

            if !new.is_null() {
                let size = cmp::min(old_size, new_size);
                unsafe { copy_nonoverlapping(ptr, new, size) };
            }

            drop((old_size, old_align)); // drop does nothing with Copy types
            unsafe { (*self.get()).lock().free(ptr) };

            new
        }
    }
}

pub unsafe fn alloc(size: size_t) -> *mut c_void {
    unsafe { (*ALLOCATOR.get()).lock().malloc(size) }.cast()
}

pub unsafe fn alloc_align(size: size_t, alignment: size_t) -> *mut c_void {
    unsafe { (*ALLOCATOR.get()).lock().memalign(alignment, size) }.cast()
}

pub unsafe fn realloc(ptr: *mut c_void, size: size_t) -> *mut c_void {
    if ptr.is_null() {
        unsafe { (*ALLOCATOR.get()).lock().malloc(size) }.cast()
    } else {
        unsafe { (*ALLOCATOR.get()).lock().realloc(ptr.cast(), size) }.cast()
    }
}

pub unsafe fn free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe { (*ALLOCATOR.get()).lock().free(ptr.cast()) }
}

pub unsafe fn alloc_usable_size(ptr: *mut c_void) -> size_t {
    if ptr.is_null() {
        return 0;
    }
    unsafe { (*ALLOCATOR.get()).lock().usable_size(ptr.cast()) }
}
