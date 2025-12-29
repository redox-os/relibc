#[cfg(target_arch = "aarch64")]
pub use self::aarch64::*;
#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "x86")]
pub use self::i686::*;
#[cfg(target_arch = "x86")]
pub mod i686;

#[cfg(target_arch = "x86_64")]
pub use self::x86_64::*;
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "riscv64")]
pub use self::riscv64::*;
#[cfg(target_arch = "riscv64")]
pub mod riscv64;

#[derive(Debug)]
#[repr(C)]
pub struct ForkScratchpad {
    pub cur_filetable_fd: usize,
    pub new_proc_fd: usize,
    pub new_thr_fd: usize,
}
