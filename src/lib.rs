#![no_std]
#![feature(lang_items)]

extern crate compiler_builtins;
extern crate platform;

extern crate ctype;
extern crate fcntl;
extern crate grp;
extern crate stdio;
extern crate stdlib;
extern crate string;
extern crate unistd;

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
