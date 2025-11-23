// test_app/src/main.rs
#![no_std]
#![no_main]
#![feature(asm)]

// Import the library function.
// The symbol 'get_message_length' will be in the dynamic symbol table.
#[link(name = "core")]
extern "C" {
    fn get_message_length(a: u8, b: u8) -> u8;
}

// Minimal Syscall Wrapper for x86-64
// Conventions: RAX=Syscall, RDI=Arg1, RSI=Arg2, RDX=Arg3
#[inline(always)]
unsafe fn syscall_write(fd: u64, buf: *const u8, len: u64) {
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!(
        "syscall",
        in("rax") 1, // SYS_write
        in("rdi") fd,
        in("rsi") buf,
        in("rdx") len,
        out("rcx") _,
        out("r11") _,
        options(nostack, preserves_flags)
    );
}

#[inline(always)]
unsafe fn syscall_exit(code: u64) -> ! {
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!(
        "syscall",
        in("rax") 60, // SYS_exit
        in("rdi") code,
        options(noreturn)
    );
    // Fallback loop if not x86-64 (verification suite focuses on x86-64 asm per prompt)
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        // 1. Call the function from the shared object.
        //    Input: 10, 20. Expected len of message is 27.
        //    Expected Result: 10 + 20 + 27 = 57 (ASCII '9')
        let result = get_message_length(10, 20);

        // 2. Create a buffer on stack to print the result byte
        //    We treat the result as a character to print.
        let buf = [result];

        // 3. Write to stdout (fd=1)
        syscall_write(1, buf.as_ptr(), 1);

        // 4. Exit cleanly
        syscall_exit(0);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { syscall_exit(101); }
}