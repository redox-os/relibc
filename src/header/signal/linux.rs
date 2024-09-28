use super::{siginfo_t, sigset_t, stack_t};
use crate::platform::types::*;
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

pub const SIGHUP: usize = 1;
pub const SIGINT: usize = 2;
pub const SIGQUIT: usize = 3;
pub const SIGILL: usize = 4;
pub const SIGTRAP: usize = 5;
pub const SIGABRT: usize = 6;
pub const SIGIOT: usize = SIGABRT;
pub const SIGBUS: usize = 7;
pub const SIGFPE: usize = 8;
pub const SIGKILL: usize = 9;
pub const SIGUSR1: usize = 10;
pub const SIGSEGV: usize = 11;
pub const SIGUSR2: usize = 12;
pub const SIGPIPE: usize = 13;
pub const SIGALRM: usize = 14;
pub const SIGTERM: usize = 15;
pub const SIGSTKFLT: usize = 16;
pub const SIGCHLD: usize = 17;
pub const SIGCONT: usize = 18;
pub const SIGSTOP: usize = 19;
pub const SIGTSTP: usize = 20;
pub const SIGTTIN: usize = 21;
pub const SIGTTOU: usize = 22;
pub const SIGURG: usize = 23;
pub const SIGXCPU: usize = 24;
pub const SIGXFSZ: usize = 25;
pub const SIGVTALRM: usize = 26;
pub const SIGPROF: usize = 27;
pub const SIGWINCH: usize = 28;
pub const SIGIO: usize = 29;
pub const SIGPOLL: usize = SIGIO;
pub const SIGPWR: usize = 30;
pub const SIGSYS: usize = 31;
pub const SIGUNUSED: usize = SIGSYS;
pub const NSIG: usize = 32;

pub const SIGRTMIN: usize = 35; // TODO: decrease to 34
pub const SIGRTMAX: usize = 64;

pub const SA_NOCLDSTOP: usize = 1;
pub const SA_NOCLDWAIT: usize = 2;
pub const SA_SIGINFO: usize = 4;
pub const SA_ONSTACK: usize = 0x0800_0000;
pub const SA_RESTART: usize = 0x1000_0000;
pub const SA_NODEFER: usize = 0x4000_0000;
pub const SA_RESETHAND: usize = 0x8000_0000;
pub const SA_RESTORER: usize = 0x0400_0000;

pub const SS_ONSTACK: usize = 1;
pub const SS_DISABLE: usize = 2;

// Those two should be updated from kernel headers
pub const MINSIGSTKSZ: usize = 2048;
pub const SIGSTKSZ: usize = 8096;

pub const SI_QUEUE: i32 = -1;
pub const SI_USER: i32 = 0;

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
    __private: [u8; 512],
}

#[repr(C)]
pub struct _libc_fpstate {
    pub cwd: u16,
    pub swd: u16,
    pub ftw: u16,
    pub fop: u16,
    pub rip: u64,
    pub rdp: u64,
    pub mxcsr: u32,
    pub mxcr_mask: u32,
    pub _st: [_libc_fpxreg; 8],
    pub _xmm: [_libc_xmmreg; 16],
    __private: [u64; 12],
}
#[repr(C)]
pub struct _libc_fpxreg {
    pub significand: [u16; 4],
    pub exponent: u16,
    __private: [u16; 3],
}

#[repr(C)]
pub struct _libc_xmmreg {
    pub element: [u32; 4],
}
#[repr(C)]
pub struct mcontext {
    pub gregs: [i64; 23], // TODO: greg_t?
    pub fpregs: *mut _libc_fpstate,
    __private: [u64; 8],
}
