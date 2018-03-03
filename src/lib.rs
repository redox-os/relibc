#![no_std]
#![feature(lang_items)]

extern crate fcntl;
extern crate unistd;

pub use fcntl::*;
pub use unistd::*;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn rust_begin_unwind(fmt: ::core::fmt::Arguments, file: &str, line: u32) -> ! {
    loop {}
}
