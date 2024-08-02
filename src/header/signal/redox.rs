use core::arch::global_asm;

use super::{sigset_t, stack_t};

pub const SIGHUP: usize = 1;
pub const SIGINT: usize = 2;
pub const SIGQUIT: usize = 3;
pub const SIGILL: usize = 4;
pub const SIGTRAP: usize = 5;
pub const SIGABRT: usize = 6;
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
pub const SIGPWR: usize = 30;
pub const SIGSYS: usize = 31;
pub const NSIG: usize = 32;

pub const SIGRTMIN: usize = 35;
pub const SIGRTMAX: usize = 64;

pub const SA_NOCLDWAIT: usize = 0x0000_0002;
pub const SA_RESTORER: usize = 0x0000_0004; // TODO: remove
pub const SA_SIGINFO: usize = 0x0200_0000;
pub const SA_ONSTACK: usize = 0x0400_0000;
pub const SA_RESTART: usize = 0x0800_0000;
pub const SA_NODEFER: usize = 0x1000_0000;
pub const SA_RESETHAND: usize = 0x2000_0000;
pub const SA_NOCLDSTOP: usize = 0x4000_0000;

pub const SS_ONSTACK: usize = 0x00000001;
pub const SS_DISABLE: usize = 0x00000002;

const _: () = {
    if SS_ONSTACK != redox_rt::signal::SS_ONSTACK {
        panic!();
    }
    if SS_DISABLE != redox_rt::signal::SS_DISABLE {
        panic!();
    }
};

// TODO: It's just a guess based on Linux
pub const MINSIGSTKSZ: usize = 2048;
pub const SIGSTKSZ: usize = 8096;

pub const SI_QUEUE: i32 = -1;
pub const SI_USER: i32 = 0;

pub(crate) type ucontext_t = ucontext;
pub(crate) type mcontext_t = mcontext;

#[repr(C)]
pub struct ucontext {
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    _pad: [usize; 1], // pad from 7*8 to 64

    #[cfg(target_arch = "x86")]
    _pad: [usize; 3], // pad from 9*4 to 12*4

    pub uc_link: *mut ucontext_t,
    pub uc_stack: stack_t,
    pub uc_sigmask: sigset_t,
    _sival: usize,
    _sigcode: u32,
    _signum: u32,
    pub uc_mcontext: mcontext_t,
}

#[repr(C)]
pub struct mcontext {
    #[cfg(target_arch = "x86")]
    _opaque: [u8; 512],
    #[cfg(target_arch = "x86-64")]
    _opaque: [u8; 864],
    #[cfg(target_arch = "aarch64")]
    _opaque: [u8; 272],
}
#[no_mangle]
pub extern "C" fn __completely_unused_cbindgen_workaround_fn_ucontext_mcontext(
    a: *const ucontext_t,
    b: *const mcontext_t,
) {
}
