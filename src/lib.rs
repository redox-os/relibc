#![no_std]
#![feature(lang_items)]
#![feature(linkage)]
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
pub extern crate signal;
pub extern crate stdio;
pub extern crate stdlib;
pub extern crate string;
pub extern crate sys_mman;
pub extern crate sys_resource;
pub extern crate sys_socket;
pub extern crate sys_stat;
pub extern crate sys_time;
pub extern crate sys_utsname;
pub extern crate sys_wait;
pub extern crate time;
pub extern crate unistd;
pub extern crate wctype;

#[cfg(not(any(test, target_os = "redox")))]
#[panic_implementation]
#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn rust_begin_unwind(pi: &::core::panic::PanicInfo) -> ! {
    use core::fmt::Write;

    let mut w = platform::FileWriter(2);
    let _ = w.write_fmt(format_args!("RELIBC PANIC: {}\n", pi));

    platform::exit(1);
}

#[cfg(not(test))]
#[lang = "eh_personality"]
#[no_mangle]
#[linkage = "weak"]
pub extern "C" fn rust_eh_personality() {}

#[cfg(not(any(test, target_os = "redox")))]
#[lang = "oom"]
#[linkage = "weak"]
#[no_mangle]
pub extern fn rust_oom(layout: ::core::alloc::Layout) -> ! {
    use core::fmt::Write;

    let mut w = platform::FileWriter(2);
    let _ = w.write_fmt(format_args!("RELIBC OOM: {} bytes aligned to {} bytes\n", layout.size(), layout.align()));

    platform::exit(1);
}

#[cfg(not(test))]
#[allow(non_snake_case)]
#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    use core::fmt::Write;

    let mut w = platform::FileWriter(2);
    let _ = w.write_str("_Unwind_Resume\n");

    platform::exit(1);
}
