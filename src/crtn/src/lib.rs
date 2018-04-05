//! crtn

#![no_std]
#![feature(global_asm)]
#![feature(lang_items)]

global_asm!(r#"
    .section .init,"ax",@progbits
        popq %rbp
        ret

    .section .fini,"ax",@progbits
        popq %rbp
        ret
"#);

#[lang = "panic_fmt"]
pub extern "C" fn rust_begin_unwind(_fmt: ::core::fmt::Arguments, _file: &str, _line: u32) -> ! {
    loop {}
}
