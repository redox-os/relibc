//! crt0

#![no_std]
#![feature(alloc)]
#![feature(asm)]
#![feature(linkage)]
#![feature(naked_functions)]
#![feature(panic_implementation)]
#![feature(lang_items)]

extern crate alloc;
extern crate platform;
extern crate stdio;

use alloc::Vec;
use core::ptr;
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

    fn envp(&self) -> *const *const u8 {
        unsafe { self.argv().offset(self.argc() + 1) }
    }
}

#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _start_rust(sp: &'static Stack) -> ! {
    extern "C" {
        fn main(argc: isize, argv: *const *const c_char, envp: *const *const c_char) -> c_int;
    }

    let argc = sp.argc();
    let argv = sp.argv();

    let envp = sp.envp();
    let mut len = 0;
    while *envp.offset(len) != ptr::null() {
        len += 1;
    }
    platform::inner_environ = Vec::with_capacity(len as usize + 1);
    for i in 0..len {
        let mut item = *envp.offset(i);
        let mut len = 0;
        while *item.offset(len) != 0 {
            len += 1;
        }

        let buf = platform::alloc(len as usize + 1) as *mut c_char;
        for i in 0..=len {
            *buf.offset(i) = *item.offset(i) as c_char;
        }
        platform::inner_environ.push(buf);
    }
    platform::inner_environ.push(ptr::null_mut());
    platform::environ = platform::inner_environ.as_mut_ptr();

    // Initialize stdin/stdout/stderr, see https://github.com/rust-lang/rust/issues/51718
    stdio::stdin = stdio::default_stdin.get();
    stdio::stdout = stdio::default_stdout.get();
    stdio::stderr = stdio::default_stderr.get();

    platform::exit(main(
        argc,
        argv as *const *const c_char,
        envp as *const *const c_char,
    ));
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

#[lang = "oom"]
#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn rust_oom(layout: ::core::alloc::Layout) -> ! {
    use core::fmt::Write;

    let mut w = platform::FileWriter(2);
    let _ = w.write_fmt(format_args!(
        "RELIBC OOM: {} bytes aligned to {} bytes\n",
        layout.size(),
        layout.align()
    ));

    platform::exit(1);
}
