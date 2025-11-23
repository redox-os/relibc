// test_app/src/main.rs
#![no_std]
#![no_main]
#![feature(asm)]

#[link(name = "core")]
extern "C" {
    fn get_message_length(a: u8, b: u8) -> u8;
}

#[inline(always)]
unsafe fn syscall_write(fd: u64, buf: *const u8, len: u64) {
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!(
    "syscall",
    in("rax") 1,
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
    in("rax") 60,
    in("rdi") code,
    options(noreturn)
    );
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        // In a real scenario, the dynamic linker would call `run_constructors()`
        // before passing control here or before `dlopen` returns.
        // Since this is a verification harness, we assume the logic in linker.rs
        // would have been executed by the loader.

        // If `get_message_length` returns valid output, it means:
        // 1. Base Relocations worked (MESSAGE accessed)
        // 2. TLS worked (THREAD_ID accessed)
        // 3. Constructors worked (INITIALIZED check passed)

        let result = get_message_length(10, 20);

        // If result is 0, it means initialization failed.
        if result == 0 {
            syscall_exit(1);
        }

        let buf = [result];
        syscall_write(1, buf.as_ptr(), 1);

        syscall_exit(0);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { syscall_exit(101); }
}