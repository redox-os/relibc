//! A part of the ptrace compatibility for Redox OS

use crate::platform::types::*;

#[repr(C)]
pub struct user_regs_struct {
    pub pc: c_ulong,
    pub ra: c_ulong,
    pub sp: c_ulong,
    pub gp: c_ulong,
    pub tp: c_ulong,
    pub t0: c_ulong,
    pub t1: c_ulong,
    pub t2: c_ulong,
    pub s0: c_ulong,
    pub s1: c_ulong,
    pub a0: c_ulong,
    pub a1: c_ulong,
    pub a2: c_ulong,
    pub a3: c_ulong,
    pub a4: c_ulong,
    pub a5: c_ulong,
    pub a6: c_ulong,
    pub a7: c_ulong,
    pub s2: c_ulong,
    pub s3: c_ulong,
    pub s4: c_ulong,
    pub s5: c_ulong,
    pub s6: c_ulong,
    pub s7: c_ulong,
    pub s8: c_ulong,
    pub s9: c_ulong,
    pub s10: c_ulong,
    pub s11: c_ulong,
    pub t3: c_ulong,
    pub t4: c_ulong,
    pub t5: c_ulong,
    pub t6: c_ulong,
}

#[repr(C)]
#[derive(Clone, Copy)] // Required for use in union
pub struct user_fpsimd_struct_f {
    pub vregs: [c_uint; 32],
    pub fcsr: c_uint,
}

#[repr(C)]
#[derive(Clone, Copy)] // Required for use in union
pub struct user_fpsimd_struct_d {
    pub vregs: [c_ulonglong; 32],
    pub fcsr: c_uint,
}

#[repr(C)]
#[derive(Clone, Copy)] // Required for use in union
pub struct user_fpsimd_struct_q {
    pub vregs: [c_ulonglong; 64],
    pub fcsr: c_uint,
    pub rsvd: [c_uint; 3],
}

#[repr(C)]
pub union user_fpsimd_struct {
    pub f: user_fpsimd_struct_f,
    pub d: user_fpsimd_struct_d,
    pub q: user_fpsimd_struct_q,
}

pub type elf_greg_t = c_ulong;
pub type elf_gregset_t = [c_ulong; 32];
pub type elf_fpregset_t = user_fpsimd_struct;

#[no_mangle]
pub extern "C" fn _cbindgen_only_generates_structs_if_they_are_mentioned_which_is_dumb_riscv64_user(
    a: user_regs_struct,
    b: user_fpsimd_struct,
    c: elf_gregset_t,
    d: elf_greg_t,
    e: elf_fpregset_t,
) {
}
