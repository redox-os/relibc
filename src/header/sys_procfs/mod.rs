#[cfg(target_arch = "aarch64")]
use crate::header::arch_aarch64_user::*;
#[cfg(target_arch = "x86_64")]
use crate::header::arch_x64_user::*;
use crate::platform::types::*;

pub const ELF_PRARGSZ: size_t = 80;

#[repr(C)]
pub struct elf_siginfo {
    pub si_signo: c_int,
    pub si_code: c_int,
    pub si_errno: c_int,
}

#[repr(C)]
pub struct time {
    pub tv_sec: c_long,
    pub tv_usec: c_long,
}

#[repr(C)]
pub struct elf_prstatus {
    pub pr_info: elf_siginfo,
    pub pr_cursig: c_short,
    pub pr_sigpend: c_ulong,
    pub pr_sighold: c_ulong,
    pub pr_pid: pid_t,
    pub pr_ppid: pid_t,
    pub pr_pgrp: pid_t,
    pub pr_sid: pid_t,
    pub pr_utime: time,
    pub pr_stime: time,
    pub pr_cutime: time,
    pub pr_cstime: time,
    pub pr_reg: elf_gregset_t,
    pub pr_fpvalid: c_int,
}

#[repr(C)]
pub struct elf_prpsinfo {
    pub pr_state: c_char,
    pub pr_sname: c_char,
    pub pr_zomb: c_char,
    pub pr_nice: c_char,
    pub pr_flag: c_uint,
    pub pr_uid: c_uint,
    pub pr_gid: c_uint,
    pub pr_pid: c_int,
    pub pr_ppid: c_int,
    pub pr_pgrp: c_int,
    pub pr_sid: c_int,
    pub pr_fname: [c_char; 16],
    pub pr_psargs: [c_char; ELF_PRARGSZ],
}

pub type psaddr_t = *mut c_void;
pub type prgregset_t = elf_gregset_t;
pub type prfpregset_t = elf_fpregset_t;
pub type lwpid_t = pid_t;
pub type prstatus_t = elf_prstatus;
pub type prpsinfo_t = elf_prpsinfo;

#[no_mangle]
pub extern "C" fn _cbindgen_only_generates_structs_if_they_are_mentioned_which_is_dumb_procfs(
    a: psaddr_t,
    b: prgregset_t,
    c: prfpregset_t,
    d: lwpid_t,
    e: prstatus_t,
    f: prpsinfo_t,
) {
}
