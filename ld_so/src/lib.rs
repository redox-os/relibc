#![no_std]
#![feature(linkage)]
#![deny(unsafe_op_in_unsafe_fn)]

use core::arch::global_asm;

#[cfg(target_arch = "aarch64")]
global_asm!(
    "
.weak _DYNAMIC
.hidden _DYNAMIC

.global _start
_start:
    mov x28, sp
    // align stack to 16 bytes
    and sp, x28, #0xfffffffffffffff0
    adr x1, _start
    mov x0, x28
    adrp x2, _DYNAMIC
    add x2, x2, #:lo12:_DYNAMIC
    // ld_so_start(stack=x0, ld_entry=x1, dynamic=x2)
    bl relibc_ld_so_start
    // restore original stack, clear registers, and jump to the new start function
    mov sp, x28
    mov x1, xzr
    mov x2, xzr
    mov x3, xzr
    mov x4, xzr
    mov x5, xzr
    mov x6, xzr
    mov x7, xzr
    mov x8, xzr
    mov x9, xzr
    mov x10, xzr
    mov x11, xzr
    mov x12, xzr
    mov x13, xzr
    mov x14, xzr
    mov x15, xzr
    mov x16, xzr
    mov x17, xzr
    mov x18, xzr
    mov x19, xzr
    mov x20, xzr
    mov x21, xzr
    mov x22, xzr
    mov x23, xzr
    mov x24, xzr
    mov x25, xzr
    mov x26, xzr
    mov x27, xzr
    mov x28, xzr
    mov x29, xzr
    mov x30, xzr
    br x0
    udf #0
"
);

#[cfg(target_arch = "x86")]
global_asm!(
    "
.globl _start
_start:
    push esp
    call relibc_ld_so_start
    pop esp
    # TODO: x86
    ud2
"
);

#[cfg(target_arch = "x86_64")]
global_asm!(
    "
.weak _DYNAMIC
.hidden _DYNAMIC

.globl _start
_start:
    lea rsi, [rip + _start]

    # Save original stack and align stack to 16 bytes
    mov rbp, rsp
    and rsp, 0xfffffffffffffff0

    # Call ld_so_start(stack=rdi, ld_entry=rsi, dynamic=rdx)
    mov rdi, rbp
    lea rdx, [rip + _DYNAMIC]
    call relibc_ld_so_start

    # Restore original stack, clear registers, and jump to new start function
    mov rsp, rbp
    xor rcx, rcx
    xor rdx, rdx
    xor rdi, rdi
    xor rsi, rsi
    xor r8, r8
    xor r9, r9
    xor r10, r10
    xor r11, r11
    fninit
    jmp rax
    ud2
"
);

#[cfg(target_arch = "riscv64")]
global_asm!(
    "
.globl _start
_start:
    mv a0, sp
    jal relibc_ld_so_start
    unimp
"
);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn main(_argc: isize, _argv: *const *const i8) -> usize {
    // LD
    0x1D
}

#[linkage = "weak"]
#[unsafe(no_mangle)]
extern "C" fn relibc_panic(_pi: &::core::panic::PanicInfo) -> ! {
    loop {}
}

#[panic_handler]
#[linkage = "weak"]
pub unsafe fn rust_begin_unwind(pi: &::core::panic::PanicInfo) -> ! {
    relibc_panic(pi)
}
