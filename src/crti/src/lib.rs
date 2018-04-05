//! crti

#![no_std]
#![feature(global_asm)]
#![feature(lang_items)]

global_asm!(r#"
    .section .init,"ax",@progbits
    .global _init
    .type _init, @function
    _init:
        push %rbp
        movq %rsp, %rbp

    .section .fini,"ax",@progbits
    .global _fini
    .type _fini, @function
    _fini:
    	push %rbp
    	movq %rsp, %rbp
"#);

#[lang = "panic_fmt"]
pub extern "C" fn rust_begin_unwind(_fmt: ::core::fmt::Arguments, _file: &str, _line: u32) -> ! {
    loop {}
}
