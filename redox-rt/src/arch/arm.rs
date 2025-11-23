// redox-rt/src/arch/arm.rs
//! ARM (32-bit) syscall handling
//! Covers armhf, armv7, and aarch32

#![allow(unused_imports)]

use core::arch::asm;

#[inline(always)]
pub unsafe fn syscall0(n: usize) -> usize {
    let ret: usize;
    asm!(
        "svc 0",
        in("r7") n,
        lateout("r0") ret,
        options(nostack, preserves_flags),
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall1(n: usize, a1: usize) -> usize {
    let ret: usize;
    asm!(
        "svc 0",
        in("r7") n,
        in("r0") a1,
        lateout("r0") ret,
        options(nostack, preserves_flags),
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall2(n: usize, a1: usize, a2: usize) -> usize {
    let ret: usize;
    asm!(
        "svc 0",
        in("r7") n,
        in("r0") a1,
        in("r1") a2,
        lateout("r0") ret,
        options(nostack, preserves_flags),
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall3(n: usize, a1: usize, a2: usize, a3: usize) -> usize {
    let ret: usize;
    asm!(
        "svc 0",
        in("r7") n,
        in("r0") a1,
        in("r1") a2,
        in("r2") a3,
        lateout("r0") ret,
        options(nostack, preserves_flags),
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall4(n: usize, a1: usize, a2: usize, a3: usize, a4: usize) -> usize {
    let ret: usize;
    asm!(
        "svc 0",
        in("r7") n,
        in("r0") a1,
        in("r1") a2,
        in("r2") a3,
        in("r3") a4,
        lateout("r0") ret,
        options(nostack, preserves_flags),
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall5(n: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize) -> usize {
    let ret: usize;
    asm!(
        "svc 0",
        in("r7") n,
        in("r0") a1,
        in("r1") a2,
        in("r2") a3,
        in("r3") a4,
        in("r4") a5,
        lateout("r0") ret,
        options(nostack, preserves_flags),
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall6(n: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize, a6: usize) -> usize {
    let ret: usize;
    asm!(
        "svc 0",
        in("r7") n,
        in("r0") a1,
        in("r1") a2,
        in("r2") a3,
        in("r3") a4,
        in("r4") a5,
        in("r5") a6,
        lateout("r0") ret,
        options(nostack, preserves_flags),
    );
    ret
}