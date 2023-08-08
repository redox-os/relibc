use crate::ALLOCATOR;
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::null_mut,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::types::*;

extern "C" {
    fn _dragonos_free(ptr: *mut c_void) -> *mut c_void;
    fn _dragonos_malloc(size: usize) -> *mut c_void;
    fn _dragonos_chunk_length(ptr: *mut c_void) -> usize;
}

pub struct Allocator {
    mstate: AtomicUsize,
}

pub const NEWALLOCATOR: Allocator = Allocator {
    mstate: AtomicUsize::new(0),
};

impl Allocator {
    pub fn set_book_keeper(&self, mstate: usize) {
        self.mstate.store(mstate, Ordering::Relaxed);
    }
    pub fn get_book_keeper(&self) -> usize {
        self.mstate.load(Ordering::Relaxed)
    }
}

unsafe impl<'a> GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        alloc(layout.size()) as *mut u8
        //alloc_align(layout.size(), layout.align()) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr as *mut c_void);
    }
}

pub unsafe fn alloc(size: usize) -> *mut c_void {
    // println!("alloc size: {}", size);
    _dragonos_malloc(size)
    //mspace_malloc(ALLOCATOR.get_book_keeper(), size)
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
pub unsafe fn alloc_align(mut size: usize, alignment: usize) -> *mut c_void {
    // println!("alloc align size: {}, alignment: {}", size, alignment);
    size = align_up(size, alignment);

    // TODO: 实现对齐分配
    _dragonos_malloc(size)
    //mspace_memalign(ALLOCATOR.get_book_keeper(), alignment, size)
}

pub unsafe fn realloc(ptr: *mut c_void, size: size_t) -> *mut c_void {
    if ptr.is_null() {
        return alloc(size);
    }
    if size == 0 {
        free(ptr);
        return null_mut();
    }

    let old_len = _dragonos_chunk_length(ptr) - 16;

    // 暴力实现

    let new_ptr = alloc(size);
    if new_ptr.is_null() {
        return null_mut();
    }

    let copy_len = if old_len < size { old_len } else { size };
    core::ptr::copy_nonoverlapping(ptr, new_ptr, copy_len);

    free(ptr);

    return new_ptr;
}

pub unsafe fn free(ptr: *mut c_void) {
    // println!("free ptr: {:#018x}", ptr as usize);
    _dragonos_free(ptr);
    //mspace_free(ALLOCATOR.get_book_keeper(), ptr)
}

#[cfg(target_os = "dragonos")]
pub fn new_mspace() -> usize {
    // dbg!("new_mspace");
    1
}
