//! crt0

#![no_std]
#![feature(linkage)]

use core::arch::global_asm;

#[cfg(target_arch = "aarch64")]
global_asm!(
    "
    .globl _start
_start:
    mov x0, sp
    and sp, x0, #0xfffffffffffffff0 //align sp
    bl relibc_start
"
);

#[cfg(target_arch = "x86")]
global_asm!(
    "
    .globl _start
    .type _start, @function
_start:
    sub esp, 8

    mov DWORD PTR [esp], 0x00001F80
    # ldmxcsr [esp]
    mov WORD PTR [esp], 0x037F
    fldcw [esp]

    add esp, 8

    push esp
    call relibc_start
    .size _start, . - _start
"
);

#[cfg(target_arch = "x86_64")]
global_asm!(
    "
    .globl _start
    .type _start, @function
_start:
    mov rdi, rsp
    and rsp, 0xFFFFFFFFFFFFFFF0

    sub rsp, 8

    mov DWORD PTR [rsp], 0x00001F80
    ldmxcsr [rsp]
    mov WORD PTR [rsp], 0x037F
    fldcw [rsp]

    add rsp, 8

    call relibc_start
    .size _start, . - _start
"
);

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
