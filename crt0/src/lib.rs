//! crt0

#![no_std]
#![feature(asm)]
#![feature(lang_items)]
#![feature(naked_functions)]

extern crate platform;

use platform::types::*;

#[no_mangle]
#[naked]
pub unsafe extern "C" fn _start() {
    asm!("mov rdi, rsp
        call _start_rust"
        :
        :
        :
        : "intel"
    );
}

#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _start_rust(sp: usize) -> ! {
    extern "C" {
        fn main(argc: c_int, argv: *const *const c_char) -> c_int;
    }

    platform::exit(main(0, 0 as *const *const c_char));
}

#[lang = "panic_fmt"]
pub extern "C" fn rust_begin_unwind(fmt: ::core::fmt::Arguments, file: &str, line: u32) -> ! {
    loop {}
}
