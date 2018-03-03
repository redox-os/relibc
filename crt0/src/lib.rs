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
        : "intel", "volatile"
    );
}

#[repr(C)]
pub struct Stack {
    argc: isize,
    argv0: *const u8,
}

impl Stack {
    fn argc(&self) -> isize {
        self.argc
    }

    fn argv(&self) -> *const *const u8 {
        &self.argv0 as *const *const u8
    }
}

#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _start_rust(sp: &'static Stack) -> ! {
    extern "C" {
        fn main(argc: isize, argv: *const *const u8) -> c_int;
    }

    let argc = sp.argc();
    let argv = sp.argv();

    platform::exit(main(argc, argv));
}

#[lang = "panic_fmt"]
pub extern "C" fn rust_begin_unwind(fmt: ::core::fmt::Arguments, file: &str, line: u32) -> ! {
    loop {}
}
