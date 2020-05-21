//! A part of the ptrace compatibility for Redox OS

use crate::platform::types::*;

#[repr(C)]
pub struct user_fpregs_struct {
    pub cwd: u16,
    pub swd: u16,
    pub ftw: u16,
    pub fop: u16,
    pub rip: u64,
    pub rdp: u64,
    pub mxcsr: u32,
    pub mxcr_mask: u32,
    pub st_space: [u32; 32],
    pub xmm_space: [u32; 64],
    pub padding: [u32; 24],
}

#[repr(C)]
pub struct user_regs_struct {
    pub r15: c_ulong,
    pub r14: c_ulong,
    pub r13: c_ulong,
    pub r12: c_ulong,
    pub rbp: c_ulong,
    pub rbx: c_ulong,
    pub r11: c_ulong,
    pub r10: c_ulong,
    pub r9: c_ulong,
    pub r8: c_ulong,
    pub rax: c_ulong,
    pub rcx: c_ulong,
    pub rdx: c_ulong,
    pub rsi: c_ulong,
    pub rdi: c_ulong,
    pub orig_rax: c_ulong,
    pub rip: c_ulong,
    pub cs: c_ulong,
    pub eflags: c_ulong,
    pub rsp: c_ulong,
    pub ss: c_ulong,
    pub fs_base: c_ulong,
    pub gs_base: c_ulong,
    pub ds: c_ulong,
    pub es: c_ulong,
    pub fs: c_ulong,
    pub gs: c_ulong,
}

pub type elf_greg_t = c_ulong;

pub type elf_gregset_t = [c_ulong; 27];
pub type elf_fpregset_t = user_fpregs_struct;
#[repr(C)]
pub struct user {
    pub regs: user_regs_struct,
    pub u_fpvalid: c_int,
    pub i387: user_fpregs_struct,
    pub u_tsize: c_ulong,
    pub u_dsize: c_ulong,
    pub u_ssize: c_ulong,
    pub start_code: c_ulong,
    pub start_stack: c_ulong,
    pub signal: c_long,
    pub reserved: c_int,
    pub u_ar0: *mut user_regs_struct,
    pub u_fpstate: *mut user_fpregs_struct,
    pub magic: c_ulong,
    pub u_comm: [c_char; 32],
    pub u_debugreg: [c_ulong; 8],
}

#[no_mangle]
pub extern "C" fn _cbindgen_only_generates_structs_if_they_are_mentioned_which_is_dumb_x86_user(
    a: user_fpregs_struct,
    b: user_regs_struct,
    c: user,
    d: elf_gregset_t,
    e: elf_greg_t,
    f: elf_fpregset_t,
) {
}
