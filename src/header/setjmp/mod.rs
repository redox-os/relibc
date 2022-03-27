//! setjmp implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/setjmp.h.html

use core::arch::global_asm;

macro_rules! platform_specific {
    ($($arch:expr,$ext:expr;)+) => {
        $(
            #[cfg(target_arch = $arch)]
            global_asm!(include_str!(concat!("impl/", $arch, "/setjmp.", $ext)));
            #[cfg(target_arch = $arch)]
            global_asm!(include_str!(concat!("impl/", $arch, "/longjmp.", $ext)));
        )+
    }
}

platform_specific! {
    "aarch64","s";
    "arm","s";
    "i386","s";
    "m68k","s";
    "microblaze","s";
    "mips","S";
    "mips64","S";
    "mipsn32","S";
    "or1k","s";
    "powerpc","S";
    "powerpc64","s";
    "s390x","s";
    "sh","S";
    "x32","s";
    "x86_64","s";
}
