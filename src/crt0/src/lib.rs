//! crt0

#![no_std]
#![feature(asm)]
#![feature(lang_items)]
#![feature(linkage)]
#![feature(naked_functions)]
#![feature(panic_implementation)]

#[no_mangle]
#[naked]
pub unsafe extern "C" fn _start() {
    #[cfg(target_arch = "x86_64")]
    asm!("mov rdi, rsp
        and rsp, 0xFFFFFFFFFFFFFFF0
        call _start_rust"
        :
        :
        :
        : "intel", "volatile"
    );
    #[cfg(target_arch = "aarch64")]
    asm!("mov x0, sp
        bl _start_rust"
        :
        :
        :
        : "volatile"
    );
}

#[panic_implementation]
#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn rust_begin_unwind(_pi: &::core::panic::PanicInfo) -> ! {
    extern "C" {
        fn exit(status: i32) -> !;
    }

    unsafe {
        exit(1)
    }
}
