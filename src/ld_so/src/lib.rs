#![no_std]
#![feature(asm)]
#![feature(linkage)]
#![feature(llvm_asm)]
#![feature(naked_functions)]

#[naked]
#[no_mangle]
pub unsafe extern "C" fn _start() {
    #[cfg(target_arch = "aarch64")]
    llvm_asm!("
        mov x0, sp
        bl relibc_ld_so_start
        # TODO: aarch64
        "
        :
        :
        :
        : "volatile"
    );
    #[cfg(target_arch = "x86_64")]
    llvm_asm!("
        # rsi = _start + 5
        call next
next:   pop rsi

        # Save original stack and align stack to 16 bytes
        mov rbp, rsp
        and rsp, 0xFFFFFFFFFFFFFFF0

        # Call ld_so_start(stack, entry)
        mov rdi, rbp
        sub rsi, 5
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
        "
        :
        :
        :
        : "intel", "volatile"
    );
}

#[naked]
#[no_mangle]
pub unsafe extern "C" fn main(_argc: isize, _argv: *const *const i8) -> usize {
    // LD
    0x1D
}

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
