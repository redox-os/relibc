use crate::platform::types::{c_double, c_uint, c_ulong, c_ulonglong};

#[repr(C)]
pub struct user_regs_struct {
    pub regs: [c_ulonglong; 31],
    pub sp: c_ulonglong,
    pub pc: c_ulonglong,
    pub pstate: c_ulonglong,
}

#[repr(C)]
pub struct user_fpsimd_struct {
    pub vregs: [c_double; 32], // BUG: rust doesn't have f128 which is equivalent for long double
    pub fpsr: c_uint,
    pub fpcr: c_uint,
}

pub type elf_greg_t = c_ulong;
pub type elf_gregset_t = *mut [c_ulong; 34];
pub type elf_fpregset_t = user_fpsimd_struct;
