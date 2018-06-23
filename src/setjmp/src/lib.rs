//! setjmp implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/setjmp.h.html

#![no_std]

extern crate platform;

use platform::types::*;

#[cfg(target_arch = "aarch64")]
#[allow(non_camel_case_types)]
type jmp_buf = [c_ulong; 22];
#[cfg(target_arch = "arm")]
#[allow(non_camel_case_types)]
type jmp_buf = [c_ulonglong; 32];
#[cfg(target_arch = "i386")]
#[allow(non_camel_case_types)]
type jmp_buf = [c_ulong; 6];
#[cfg(target_arch = "m68k")]
#[allow(non_camel_case_types)]
type jmp_buf = [c_ulong; 39];
#[cfg(target_arch = "microblaze")]
#[allow(non_camel_case_types)]
type jmp_buf = [c_ulong; 18];
#[cfg(target_arch = "mips")]
#[allow(non_camel_case_types)]
type jmp_buf = [c_ulonglong; 13];
#[cfg(target_arch = "mips64")]
#[allow(non_camel_case_types)]
type jmp_buf = [c_ulonglong; 23];
#[cfg(target_arch = "mipsn32")]
#[allow(non_camel_case_types)]
type jmp_buf = [c_ulonglong; 23];
#[cfg(target_arch = "or1k")]
#[allow(non_camel_case_types)]
type jmp_buf = [c_ulong; 13];
#[cfg(target_arch = "powerpc")]
#[allow(non_camel_case_types)]
type jmp_buf = [c_ulonglong; 56];
#[cfg(target_arch = "powerpc64")]
#[allow(non_camel_case_types)]
type jmp_buf = [u128; 32];
#[cfg(target_arch = "s390x")]
#[allow(non_camel_case_types)]
type jmp_buf = [c_ulong; 18];
#[cfg(target_arch = "sh")]
#[allow(non_camel_case_types)]
type jmp_buf = [c_ulong; 15];
#[cfg(target_arch = "x32")]
#[allow(non_camel_case_types)]
type jmp_buf = [c_ulonglong; 8];
#[cfg(target_arch = "x86_64")]
#[allow(non_camel_case_types)]
type jmp_buf = [c_ulong; 8];

#[link(name = "setjmp", kind = "static")]
extern "C" {
    fn setjmp(buf: jmp_buf) -> c_int;
}
#[link(name = "longjmp", kind = "static")]
extern "C" {
    fn longjmp(buf: jmp_buf, val: c_int);
}
