//! `setjmp.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/setjmp.h.html>.

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
unsafe extern "C" {
    /// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setjmp.html>.
    pub fn setjmp(jb: *mut u64) -> i32;
    /// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/longjmp.html>.
    pub fn longjmp(jb: *mut u64, ret: i32);
}
