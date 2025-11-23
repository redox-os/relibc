// test_app/src/main.rs
#![no_std]
#![no_main]
#![feature(asm)]

// Import the library function.
#[link(name = "core")]
extern "C" {
    fn get_message_length(a: u8, b: u8) -> u8;
}

// Minimal Syscall Wrapper
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
    #[cfg(not(target_arch = "x86_64"))]
    { let _ = (fd, buf, len); }
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
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        // 1. Call the function from the shared object.
        //    Input: 10, 20.
        //    Expected: 10 + 20 + 27 + 5 = 62 (ASCII '>')
        let result = get_message_length(10, 20);

        // 2. Print result
        let buf = [result];
        syscall_write(1, buf.as_ptr(), 1);

        // 3. Exit
        syscall_exit(0);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { syscall_exit(101); }
}