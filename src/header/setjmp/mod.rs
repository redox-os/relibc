//! setjmp implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/setjmp.h.html

use core::arch::global_asm;

macro_rules! platform_specific {
    ($($rust_arch:expr,$c_arch:expr,$ext:expr;)+) => {
        $(
            #[cfg(target_arch = $rust_arch)]
            global_asm!(include_str!(concat!("impl/", $c_arch, "/setjmp.", $ext)));
            #[cfg(target_arch = $rust_arch)]
            global_asm!(include_str!(concat!("impl/", $c_arch, "/longjmp.", $ext)));
        )+
    }
}

platform_specific! {
    "aarch64","aarch64", "s";
    "x86","i386","s";
    "x86_64","x86_64","s";
    "riscv64", "riscv64", "S";
}

//Each platform has different sizes for sigjmp_buf, currently only x86_64 is supported
extern "C" {
    pub fn setjmp(jb: *mut u64) -> i32;
    pub fn longjmp(jb: *mut u64, ret: i32);
}
