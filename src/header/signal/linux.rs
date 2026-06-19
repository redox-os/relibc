use super::{sigset_t, stack_t};
use crate::platform::types::{c_longlong, c_uchar, c_uint, c_ulong, c_ulonglong, c_ushort};
use core::arch::global_asm;

// Needs to be defined in assembly because it can't have a function prologue
// rax is register, 15 is RT_SIGRETURN
#[cfg(target_arch = "x86_64")]
global_asm!(
    "
    .global __restore_rt
    __restore_rt:
        mov rax, 15
        syscall
"
);
// x8 is register, 139 is RT_SIGRETURN
#[cfg(target_arch = "aarch64")]
global_asm!(
    "
    .global __restore_rt
    __restore_rt:
        mov x8, #139
        svc 0
"
);

#[cfg(target_arch = "riscv64")]
global_asm!(
    "
    .global __restore_rt
    __restore_rt:
        li a7, 139
        ecall
"
);

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/signal.7.html>.
///
/// IOT trap. A synonym for `SIGABRT`.
pub const SIGIOT: usize = super::constants::SIGABRT;
// TODO mark #[deprecated]?
/// Obsolete in issue 7, removed in issue 8.
///
/// Pollable event.
/// Default action: T
pub const SIGPOLL: usize = super::constants::SIGIO;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/signal.7.html>.
///
/// Synonymous with `SIGSYS`.
pub const SIGUNUSED: usize = super::constants::SIGSYS;

// TODO why are these SA_* constants different to Redox?
pub const SA_NOCLDSTOP: usize = 1;
pub const SA_SIGINFO: usize = 4;
pub const SA_ONSTACK: usize = 0x0800_0000;
pub const SA_RESTART: usize = 0x1000_0000;
pub const SA_NODEFER: usize = 0x4000_0000;
pub const SA_RESETHAND: usize = 0x8000_0000;
pub const SA_RESTORER: usize = 0x0400_0000;

// Mirrors the ucontext_t struct from the libc crate on Linux.

pub(crate) type ucontext_t = ucontext;
pub(crate) type mcontext_t = mcontext;

#[repr(C)]
pub struct ucontext {
    pub uc_flags: c_ulong,
    pub uc_link: *mut ucontext_t,
    pub uc_stack: stack_t,
    pub uc_mcontext: mcontext_t,
    pub uc_sigmask: sigset_t,
    __private: [c_uchar; 512],
}

#[repr(C)]
pub struct _libc_fpstate {
    pub cwd: c_ushort,
    pub swd: c_ushort,
    pub ftw: c_ushort,
    pub fop: c_ushort,
    pub rip: c_ulonglong,
    pub rdp: c_ulonglong,
    pub mxcsr: c_uint,
    pub mxcr_mask: c_uint,
    pub _st: [_libc_fpxreg; 8],
    pub _xmm: [_libc_xmmreg; 16],
    __private: [c_ulonglong; 12],
}
#[repr(C)]
pub struct _libc_fpxreg {
    pub significand: [c_ushort; 4],
    pub exponent: c_ushort,
    __private: [c_ushort; 3],
}

#[repr(C)]
pub struct _libc_xmmreg {
    pub element: [c_uint; 4],
}
#[repr(C)]
pub struct mcontext {
    pub gregs: [c_longlong; 23], // TODO: greg_t?
    pub fpregs: *mut _libc_fpstate,
    __private: [c_ulonglong; 8],
}
