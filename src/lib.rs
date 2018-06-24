#![no_std]
#![feature(lang_items)]
#![feature(panic_implementation)]

//extern crate compiler_builtins;
extern crate platform;

pub extern crate ctype;
pub extern crate errno;
pub extern crate fcntl;
pub extern crate fenv;
pub extern crate float;
pub extern crate grp;
pub extern crate locale;
pub extern crate netinet;
pub extern crate semaphore;
pub extern crate setjmp;
pub extern crate stdio;
pub extern crate stdlib;
pub extern crate string;
pub extern crate sys_mman;
pub extern crate sys_resource;
pub extern crate sys_socket;
pub extern crate sys_stat;
pub extern crate sys_time;
pub extern crate sys_wait;
pub extern crate time;
pub extern crate unistd;
pub extern crate wctype;

#[panic_implementation]
#[no_mangle]
pub extern "C" fn relibc_panic(pi: &::core::panic::PanicInfo) -> ! {
    use core::fmt::Write;

    let mut w = platform::FileWriter(2);
    let _ = w.write_fmt(format_args!("RELIBC PANIC: {}\n", pi));

    platform::exit(1);
}

#[lang = "oom"]
#[no_mangle]
pub extern fn relibc_oom(layout: ::core::alloc::Layout) -> ! {
    use core::fmt::Write;

    let mut w = platform::FileWriter(2);
    let _ = w.write_fmt(format_args!("RELIBC OOM: {} bytes aligned to {} bytes\n", layout.size(), layout.align()));

    platform::exit(1);
}
