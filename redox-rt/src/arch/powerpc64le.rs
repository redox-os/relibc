// redox-rt/src/arch/powerpc64le.rs
//! PowerPC64 Little Endian syscall handling

#![allow(unused_imports)]

use core::arch::asm;

#[inline(always)]
pub unsafe fn syscall0(n: usize) -> usize {
    let ret: usize;
    // r0: syscall number
    // r3: return value (and first arg)
    asm!(
        "sc",
        in("r0") n,
        lateout("r3") ret,
        out("r4") _, out("r5") _, out("r6") _, out("r7") _,
        out("r8") _, out("r9") _, out("r10") _, out("r11") _,
        out("r12") _, out("r13") _,
        options(nostack)
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall1(n: usize, a1: usize) -> usize {
    let ret: usize;
    asm!(
        "sc",
        in("r0") n,
        in("r3") a1,
        lateout("r3") ret,
        out("r4") _, out("r5") _, out("r6") _, out("r7") _,
        out("r8") _, out("r9") _, out("r10") _, out("r11") _,
        out("r12") _, out("r13") _,
        options(nostack)
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall2(n: usize, a1: usize, a2: usize) -> usize {
    let ret: usize;
    asm!(
        "sc",
        in("r0") n,
        in("r3") a1,
        in("r4") a2,
        lateout("r3") ret,
        out("r5") _, out("r6") _, out("r7") _,
        out("r8") _, out("r9") _, out("r10") _, out("r11") _,
        out("r12") _, out("r13") _,
        options(nostack)
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall3(n: usize, a1: usize, a2: usize, a3: usize) -> usize {
    let ret: usize;
    asm!(
        "sc",
        in("r0") n,
        in("r3") a1,
        in("r4") a2,
        in("r5") a3,
        lateout("r3") ret,
        out("r6") _, out("r7") _,
        out("r8") _, out("r9") _, out("r10") _, out("r11") _,
        out("r12") _, out("r13") _,
        options(nostack)
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall4(n: usize, a1: usize, a2: usize, a3: usize, a4: usize) -> usize {
    let ret: usize;
    asm!(
        "sc",
        in("r0") n,
        in("r3") a1,
        in("r4") a2,
        in("r5") a3,
        in("r6") a4,
        lateout("r3") ret,
        out("r7") _,
        out("r8") _, out("r9") _, out("r10") _, out("r11") _,
        out("r12") _, out("r13") _,
        options(nostack)
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall5(n: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize) -> usize {
    let ret: usize;
    asm!(
        "sc",
        in("r0") n,
        in("r3") a1,
        in("r4") a2,
        in("r5") a3,
        in("r6") a4,
        in("r7") a5,
        lateout("r3") ret,
        out("r8") _, out("r9") _, out("r10") _, out("r11") _,
        out("r12") _, out("r13") _,
        options(nostack)
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall6(n: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize, a6: usize) -> usize {
    let ret: usize;
    asm!(
        "sc",
        in("r0") n,
        in("r3") a1,
        in("r4") a2,
        in("r5") a3,
        in("r6") a4,
        in("r7") a5,
        in("r8") a6,
        lateout("r3") ret,
        out("r9") _, out("r10") _, out("r11") _,
        out("r12") _, out("r13") _,
        options(nostack)
    );
    ret
}