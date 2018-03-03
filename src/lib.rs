#![no_std]
#![feature(lang_items)]

extern crate compiler_builtins;
extern crate platform;

extern crate fcntl;
extern crate stdio;
extern crate stdlib;
extern crate string;
extern crate unistd;

use core::fmt;

struct PanicWriter;

impl fmt::Write for PanicWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        platform::write(2, s.as_bytes());
        Ok(())
    }
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn rust_begin_unwind(fmt: fmt::Arguments, file: &str, line: u32) -> ! {
    use fmt::Write;

    let _ = PanicWriter.write_fmt(format_args!("{}:{}: {}\n", file, line, fmt));

    platform::exit(1);
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    use fmt::Write;

    let _ = PanicWriter.write_str("_Unwind_Resume\n");

    platform::exit(1);
}
