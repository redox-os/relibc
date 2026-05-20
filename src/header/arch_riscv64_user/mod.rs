use crate::platform::types::{c_double, c_float, c_uint, c_ulong};

#[repr(C)]
pub struct user_regs_struct {
    pub regs: [c_ulong; 31], // x1-x31
    pub pc: c_ulong,
}

#[repr(C)]
pub struct user_fpregs_f_struct {
    pub fpregs: [c_float; 32],
    pub fcsr: c_uint,
}

#[repr(C)]
pub struct user_fpregs_g_struct {
    pub fpregs: [c_double; 32],
    pub fcsr: c_uint,
}

#[repr(C)]
pub struct user_fpregs_struct {
    pub f_regs: user_fpregs_f_struct,
    pub g_regs: user_fpregs_g_struct,
}

pub type elf_greg_t = c_ulong;
pub type elf_gregset_t = user_regs_struct;
pub type elf_fpregset_t = user_fpregs_struct;
