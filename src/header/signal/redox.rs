use redox_rt::signal::SiginfoAbi;

#[cfg(any(target_arch = "x86", target_arch = "aarch64", target_arch = "riscv64"))]
use crate::platform::types::c_uchar;
use crate::platform::types::{c_uint, c_ulong};

use super::{siginfo_t, sigset_t, stack_t};

// The below SA_* constants are different to Linux for implementation reasons.
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/sigaction.2.html>.
///
/// Not intended for application use. Used by C libraries to indicate that the
/// `sa_restorer` field contains the address of a "signal trampoline".
pub const SA_RESTORER: usize = 0x0000_0004; // TODO: remove
/// Causes extra information to be passed to signal handlers at the time of
/// receipt of a signal.
pub const SA_SIGINFO: usize = 0x0200_0000;
/// Process is executing on an alternate signal stack.
pub const SA_ONSTACK: usize = 0x0400_0000;
/// Causes certain functions to become restartable.
pub const SA_RESTART: usize = 0x0800_0000;
/// Causes signal not to be automatically blocked on entry to signal handler.
pub const SA_NODEFER: usize = 0x1000_0000;
/// Causes signal dispositions to be set to `SIG_DFL` on entry to signal
/// handlers.
pub const SA_RESETHAND: usize = 0x2000_0000;
/// Do not generate `SIGCHLD` when children stop or stopped children continue.
pub const SA_NOCLDSTOP: usize = 0x4000_0000;

const _: () = {
    if super::constants::SS_ONSTACK != redox_rt::signal::SS_ONSTACK {
        panic!();
    }
    if super::constants::SS_DISABLE != redox_rt::signal::SS_DISABLE {
        panic!();
    }
    if super::constants::MINSIGSTKSZ != redox_rt::signal::MIN_SIGALTSTACK_SIZE {
        panic!();
    }
};

pub(crate) type ucontext_t = ucontext;
/// A machine-specific representation of the saved context.
pub(crate) type mcontext_t = mcontext;

//TODO: share definition with SigStack?
#[repr(C)]
pub struct ucontext {
    #[cfg(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64"
    ))]
    _pad: [c_ulong; 1], // pad from 7*8 to 64

    #[cfg(target_arch = "x86")]
    _pad: [c_ulong; 3], // pad from 9*4 to 12*4

    /// Pointer to the context that is resumed when this context returns.
    pub uc_link: *mut ucontext_t,
    /// The stack used by this context.
    pub uc_stack: stack_t,
    /// The set of signals that are blocked when this context is active.
    pub uc_sigmask: sigset_t,
    _sival: c_ulong,
    _sigcode: c_uint,
    _signum: c_uint,
    /// A machine-specific representation of the saved context.
    pub uc_mcontext: mcontext_t,
}

#[cfg(target_arch = "x86")]
#[repr(C)]
pub struct mcontext {
    _opaque: [c_uchar; 512],
}

//TODO: share definition with ArchIntRegs?
//TODO: repr(align(16))?
#[cfg(target_arch = "x86_64")]
#[repr(C)]
pub struct mcontext {
    pub ymm_upper: [[c_ulong; 2]; 16],
    pub fxsave: [[c_ulong; 2]; 29],
    pub r15: c_ulong, // fxsave "available" +0
    pub r14: c_ulong, // available +8
    pub r13: c_ulong, // available +16
    pub r12: c_ulong, // available +24
    pub rbp: c_ulong, // available +32
    pub rbx: c_ulong, // available +40
    pub r11: c_ulong, // outside fxsave, and so on
    pub r10: c_ulong,
    pub r9: c_ulong,
    pub r8: c_ulong,
    pub rax: c_ulong,
    pub rcx: c_ulong,
    pub rdx: c_ulong,
    pub rsi: c_ulong,
    pub rdi: c_ulong,
    pub rflags: c_ulong,
    pub rip: c_ulong,
    pub rsp: c_ulong,
}

#[cfg(target_arch = "aarch64")]
#[repr(C)]
pub struct mcontext {
    _opaque: [c_uchar; 272],
}

#[cfg(target_arch = "riscv64")]
#[repr(C)]
pub struct mcontext {
    _opaque: [c_uchar; 520],
}

impl From<SiginfoAbi> for siginfo_t {
    fn from(value: SiginfoAbi) -> Self {
        unsafe { core::mem::transmute(value) }
    }
}
