// lib_core/src/lib.rs
#![no_std]

// Global Allocator Stub
// Required for a no_std cdylib even if we don't perform heap allocations explicitly,
// to satisfy the linker's requirement for the allocator symbols.
struct DummyAllocator;

unsafe impl core::alloc::GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        core::ptr::null_mut() // Verify suite should not allocate
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

// We define a private static byte array.
// Because this is a position-independent shared object (PIC), the address of 'MESSAGE'
// is relative to the library load base. Accessing it *requires* the dynamic linker
// to have successfully processed R_*_RELATIVE relocations.
static MESSAGE: &[u8] = b"Dynamic Linker Verification";

#[no_mangle]
pub extern "C" fn get_message_length(a: u8, b: u8) -> u8 {
    // Accessing MESSAGE.len() forces the CPU to read the pointer to MESSAGE.
    // If relocation failed, this pointer will be raw offset (small value) 
    // instead of valid virtual address (large value), likely causing a segfault
    // or reading garbage.
    let len = MESSAGE.len() as u8;
    
    // Return a calculable value to verify execution flow + data access
    // The prompt suggests a test value like a + b + 5, 
    // here we incorporate 'len' to prove relocation success.
    // len of "Dynamic Linker Verification" is 27.
    // a + b + 27
    a.wrapping_add(b).wrapping_add(len)
}