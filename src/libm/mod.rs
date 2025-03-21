// Enable support for complex numbers only on architectures where the builtin
// C complex type has the same calling convention rules as a struct containing
// two scalars. Notably, this excludes 32-bit "x86".
#[cfg(any(
    target_arch = "arm",
    target_arch = "aarch64",
    target_arch = "riscv64",
    target_arch = "x86_64"
))]
pub mod complex;
pub mod math;
