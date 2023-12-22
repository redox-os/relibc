mod aarch64;
mod x86;
mod x86_64;

#[cfg(target_arch = "aarch64")]
pub use aarch64::native::*;
#[cfg(target_arch = "x86")]
pub use x86::native::*;
#[cfg(target_arch = "x86_64")]
pub use x86_64::native::*;
