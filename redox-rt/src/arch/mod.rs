
#[cfg(target_arch = "aarch64")]
pub use self::aarch64::*;
#[cfg(target_arch = "aarch64")]
mod aarch64;

#[cfg(any(target_arch = "arm", target_arch = "aarch32"))]
pub use self::arm::*;
#[cfg(any(target_arch = "arm", target_arch = "aarch32"))]
mod arm;

#[cfg(target_arch = "x86")]
pub use self::i686::*;
#[cfg(target_arch = "x86")]
mod i686;

#[cfg(target_arch = "x86_64")]
pub use self::x86_64::*;
#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "riscv64")]
pub use self::riscv64::*;
#[cfg(target_arch = "riscv64")]
mod riscv64;

#[cfg(target_arch = "powerpc64le")]
pub use self::powerpc64le::*;
#[cfg(target_arch = "powerpc64le")]
mod powerpc64le;