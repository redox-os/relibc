// lib_core/src/lib.rs
#![no_std]

// Global Allocator Stub
struct DummyAllocator;

unsafe impl core::alloc::GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

#[global_allocator]
static ALLOCATOR: DummyAllocator = DummyAllocator;

// Panic Handler Stub
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Verification Logic
// ------------------
// Accessing this private static ensures the linker performs a relocation.
static MESSAGE: &[u8] = b"Dynamic Linker Verification";

#[no_mangle]
pub extern "C" fn get_message_length(a: u8, b: u8) -> u8 {
    // Accessing MESSAGE.len() forces a relocation read.
    let len = MESSAGE.len() as u8;

    // Calculation: a + b + 27 + 5
    a.wrapping_add(b).wrapping_add(len).wrapping_add(5)
}