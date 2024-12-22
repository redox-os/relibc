mod aarch64;
mod x86;
mod x86_64;

#[cfg(target_arch = "aarch64")]
pub use aarch64::native::*;
#[cfg(target_arch = "x86")]
pub use x86::native::*;
#[cfg(target_arch = "x86_64")]
pub use x86_64::native::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod x86_common {
    use crate::platform::types::{c_int, c_uint};

    pub const FE_INVALID: c_int = 1;
    pub const FE_DIVBYZERO: c_int = 4;
    pub const FE_OVERFLOW: c_int = 8;
    pub const FE_UNDERFLOW: c_int = 16;
    pub const FE_INEXACT: c_int = 32;

    pub const FE_ALL_EXCEPT: c_int = 63;

    pub const FE_TONEAREST: c_int = 0;
    pub const FE_DOWNWARD: c_int = 0x400;
    pub const FE_UPWARD: c_int = 0x800;
    pub const FE_TOWARDZERO: c_int = 0xc00;

    pub const ROUND_MASK: c_int = FE_TONEAREST | FE_DOWNWARD | FE_UPWARD | FE_TOWARDZERO;
    pub const SSE_ROUND_SHIFT: c_int = 3;
    pub const SSE_MASK_SHIFT: c_int = 7;

    #[repr(C)]
    #[derive(Default)]
    pub struct fenv_t {
        pub x87: X87Reg,
        pub mxcsr: c_uint,
    }

    #[repr(C)]
    #[derive(Default)]
    pub struct X87Reg {
        pub control: c_uint,
        pub status: c_uint,
        tag: c_uint,
        others: [c_uint; 4],
    }

    pub type fexcept_t = c_uint;
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use x86_common::*;
