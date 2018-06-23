#![no_std]
#![feature(lang_items)]

extern crate compiler_builtins;
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

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn rust_begin_unwind(fmt: ::core::fmt::Arguments, file: &str, line: u32) -> ! {
    use core::fmt::Write;

    let mut w = platform::FileWriter(2);
    let _ = w.write_fmt(format_args!("{}:{}: {}\n", file, line, fmt));

    platform::exit(1);
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    use core::fmt::Write;

    let mut w = platform::FileWriter(2);
    let _ = w.write_str("_Unwind_Resume\n");

    platform::exit(1);
}
