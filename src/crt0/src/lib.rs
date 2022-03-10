//! crt0

#![no_std]
#![feature(linkage)]

use core::arch::global_asm;

#[cfg(target_arch = "x86_64")]
global_asm!("
    .globl _start
_start:
    mov rdi, rsp
    and rsp, 0xFFFFFFFFFFFFFFF0
    call relibc_start
");
#[cfg(target_arch = "aarch64")]
global_asm!("
    .globl _start
_start:
    mov x0, sp
    bl relibc_start
");

#[linkage = "weak"]
#[no_mangle]
extern "C" fn relibc_panic(pi: &::core::panic::PanicInfo) -> ! {
    loop {}
}

#[panic_handler]
#[linkage = "weak"]
#[no_mangle]
pub unsafe extern "C" fn rust_begin_unwind(pi: &::core::panic::PanicInfo) -> ! {
    relibc_panic(pi)
}
