//! crti

#![no_std]
#![feature(linkage)]

use core::arch::global_asm;

// https://wiki.osdev.org/Creating_a_C_Library#crtbegin.o.2C_crtend.o.2C_crti.o.2C_and_crtn.o
#[cfg(target_arch = "x86_64")]
global_asm!(
    r#"
    .section .init
    .global _init
    _init:
        push rbp
        mov rbp, rsp
        // Created a new stack frame and updated the stack pointer
        // Body will be filled in by gcc and ended by crtn.o

    .section .fini
    .global _fini
    _fini:
        push rbp
        mov rbp, rsp
        // Created a new stack frame and updated the stack pointer
        // Body will be filled in by gcc and ended by crtn.o
"#
);
// https://git.musl-libc.org/cgit/musl/tree/crt/aarch64/crti.s
#[cfg(target_arch = "aarch64")]
global_asm!(
    r#"
    .section .init
    .global _init
    .type _init,%function
    _init:
        stp x29,x30,[sp,-16]!
        mov x29,sp
        // stp: "stores two doublewords from the first and second argument to memory addressed by addr"
        // Body will be filled in by gcc and ended by crtn.o

    .section .fini
    .global _fini
    .type _fini,%function
    _fini:
        stp x29,x30,[sp,-16]!
        mov x29,sp
        // stp: "stores two doublewords from the first and second argument to memory addressed by addr"
        // Body will be filled in by gcc and ended by crtn.o
"#
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
