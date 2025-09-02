use crate::{
    header::sys_mman::{self, MAP_FAILED, MREMAP_MAYMOVE},
    platform::{types::*, Pal, Sys},
    sync::Mutex,
};
use core::ptr;

use dlmalloc::Allocator;

/// System setting for Redox/Linux
pub struct System {
    _priv: (),
}

impl System {
    pub const fn new() -> System {
        System { _priv: () }
    }
}

unsafe impl Allocator for System {
    fn alloc(&self, size: usize) -> (*mut u8, usize, u32) {
        let Ok(addr) = (unsafe {
            Sys::mmap(
                0 as *mut _,
                size,
                sys_mman::PROT_WRITE | sys_mman::PROT_READ,
                sys_mman::MAP_ANON | sys_mman::MAP_PRIVATE,
                -1,
                0,
            )
        }) else {
            return (ptr::null_mut(), 0, 0);
        };
        (addr as *mut u8, size, 0)
    }

    fn remap(&self, ptr: *mut u8, oldsize: usize, newsize: usize, can_move: bool) -> *mut u8 {
        let flags = if can_move { MREMAP_MAYMOVE } else { 0 };
        let Ok(ptr) =
            (unsafe { Sys::mremap(ptr as *mut _, oldsize, newsize, flags, ptr::null_mut()) })
        else {
            return ptr::null_mut();
        };
        ptr as *mut u8
    }

    fn free_part(&self, ptr: *mut u8, oldsize: usize, newsize: usize) -> bool {
        unsafe {
            if Sys::mremap(ptr as *mut _, oldsize, newsize, 0, ptr::null_mut()).is_ok() {
                return true;
            }
            Sys::munmap(ptr.add(newsize) as *mut _, oldsize - newsize).is_ok()
        }
    }

    fn free(&self, ptr: *mut u8, size: usize) -> bool {
        unsafe { Sys::munmap(ptr as *mut _, size).is_ok() }
    }

    fn can_release_part(&self, _flags: u32) -> bool {
        true
    }

    fn allocates_zeros(&self) -> bool {
        true
    }

    fn page_size(&self) -> usize {
        4096
    }
}
