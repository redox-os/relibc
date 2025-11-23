// lib_core/src/lib.rs
#![no_std]
#![feature(thread_local)]

struct BumpAllocator {
    heap: core::cell::UnsafeCell<[u8; 4096]>,
    offset: core::cell::UnsafeCell<usize>,
}

unsafe impl Sync for BumpAllocator {}

unsafe impl core::alloc::GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let offset = *self.offset.get();
        let heap_ptr = self.heap.get() as *mut u8;
        let align_offset = (heap_ptr.add(offset) as usize) % layout.align();
        let padding = if align_offset == 0 { 0 } else { layout.align() - align_offset };
        let start = offset + padding;
        let end = start + layout.size();
        if end <= 4096 {
            *self.offset.get() = end;
            heap_ptr.add(start)
        } else {
            core::ptr::null_mut()
        }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator {
    heap: core::cell::UnsafeCell::new([0; 4096]),
    offset: core::cell::UnsafeCell::new(0),
};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }

static MESSAGE: &[u8] = b"Linker with Dependencies & TLS";

#[thread_local]
static mut THREAD_ID: u64 = 0;

// Verification Flag for Constructors
static mut INITIALIZED: bool = false;

// Constructor Function
// This section name forces the linker (compile-time) to put a pointer to this function
// into the .init_array section of the ELF.
#[link_section = ".init_array"]
pub static INIT_FUNC: extern "C" fn() = my_constructor;

extern "C" fn my_constructor() {
    unsafe {
        INITIALIZED = true;
    }
}

#[no_mangle]
pub extern "C" fn get_message_length(a: u8, b: u8) -> u8 {
    unsafe {
        THREAD_ID += 1;
        // Check if constructor ran
        if !INITIALIZED {
            return 0; // Fail if not initialized
        }
    }
    let len = MESSAGE.len() as u8;
    a.wrapping_add(b).wrapping_add(len)
}