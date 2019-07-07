//! A part of the ptrace compatibility for Redox OS

use platform::types::*;

#[repr(C)]
pub struct user_fpregs_struct {
    cwd: u16,
    swd: u16,
    ftw: u16,
    fop: u16,
    rip: u64,
    rdp: u64,
    mxcsr: u32,
    mxcr_mask: u32,
    st_space: [u32; 32],
    xmm_space: [u32; 64],
    padding: [u32; 24],
}

#[repr(C)]
pub struct user_regs_struct {
    r15: c_ulong,
    r14: c_ulong,
    r13: c_ulong,
    r12: c_ulong,
    rbp: c_ulong,
    rbx: c_ulong,
    r11: c_ulong,
    r10: c_ulong,
    r9: c_ulong,
    r8: c_ulong,
    rax: c_ulong,
    rcx: c_ulong,
    rdx: c_ulong,
    rsi: c_ulong,
    rdi: c_ulong,
    orig_rax: c_ulong,
    rip: c_ulong,
    cs: c_ulong,
    eflags: c_ulong,
    rsp: c_ulong,
    ss: c_ulong,
    fs_base: c_ulong,
    gs_base: c_ulong,
    ds: c_ulong,
    es: c_ulong,
    fs: c_ulong,
    gs: c_ulong,
}

#[no_mangle]
pub extern "C" fn _cbindgen_only_generates_structs_if_they_are_mentioned_which_is_dumb(
    a: user_fpregs_struct,
    b: user_regs_struct,
) {
}
