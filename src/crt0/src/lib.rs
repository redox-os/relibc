//! crt0

#![no_std]
#![feature(linkage)]

use core::{
    arch::global_asm,
    ffi::{c_char, c_int},
};

#[cfg(target_arch = "aarch64")]
global_asm!(
    "
    .globl _start
_start:
    mov x0, sp
    and sp, x0, #0xfffffffffffffff0 //align sp
    bl relibc_crt0
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
    call relibc_crt0
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

    call relibc_crt0
    .size _start, . - _start
"
);

#[cfg(target_arch = "riscv64")]
global_asm!(
    "
    .globl _start
_start:
    mv a0, sp
    la t0, relibc_crt0
    jalr ra, t0
"
);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn relibc_crt0(sp: usize) -> ! {
    // This wrapper ensures a dynamic libc.so can access a hidden main function
    //TODO: common definition of types
    unsafe extern "C" {
        fn main(argc: isize, argv: *mut *mut c_char, envp: *mut *mut c_char) -> c_int;
        fn relibc_start_v1(
            sp: usize,
            main: unsafe extern "C" fn(
                argc: isize,
                argv: *mut *mut c_char,
                envp: *mut *mut c_char,
            ) -> c_int,
        ) -> !;
    }
    unsafe { relibc_start_v1(sp, main) }
}

#[linkage = "weak"]
#[unsafe(no_mangle)]
pub extern "C" fn relibc_panic(_pi: &::core::panic::PanicInfo) -> ! {
    loop {}
}

#[panic_handler]
#[linkage = "weak"]
pub unsafe fn rust_begin_unwind(pi: &::core::panic::PanicInfo) -> ! {
    relibc_panic(pi)
}
