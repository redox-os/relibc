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
