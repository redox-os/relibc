//! crt0

#![no_std]
#![feature(asm)]
#![feature(linkage)]
#![feature(naked_functions)]
#![feature(panic_implementation)]

extern crate platform;

use platform::types::*;

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
    use core::fmt::Write;

    extern "C" {
        fn main(argc: isize, argv: *const *const u8) -> c_int;
    }

    let argc = sp.argc();
    let argv = sp.argv();

    platform::exit(main(argc, argv));
}

#[panic_implementation]
#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn rust_begin_unwind(pi: &::core::panic::PanicInfo) -> ! {
    use core::fmt::Write;

    let mut w = platform::FileWriter(2);
    let _ = w.write_fmt(format_args!("RELIBC CRT0 PANIC: {}\n", pi));

    platform::exit(1);
}
