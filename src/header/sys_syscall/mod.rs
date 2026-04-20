// copied from sc crate
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub mod aarch64;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub mod x86_64;
