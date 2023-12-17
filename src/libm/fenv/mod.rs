//! fenv.h implementation for Redox, following
//! http://pubs.opengroup.org/onlinepubs/9699919799/basedefs/fenv.h.html

use crate::platform::types::*;

#[cfg(target_arch = "x86_64")]
mod  consts {
    use super::c_int;

    pub const  FE_INVALID: c_int = 1;
    pub const  FE_DIVBYZERO: c_int = 4;
    pub const  FE_OVERFLOW: c_int = 8;
    pub const  FE_UNDERFLOW: c_int = 16;
    pub const  FE_INEXACT: c_int = 32;

    pub const  FE_ALL_EXCEPT: c_int = 63;

    pub const  FE_TONEAREST: c_int = 0;
    pub const  FE_DOWNWARD: c_int = 0x400;
    pub const  FE_UPWARD: c_int = 0x800;
    pub const  FE_TOWARDZERO: c_int = 0xc00;
}

#[cfg(target_arch = "aarch64")]
mod  consts {
    use super::c_int;

    pub const  FE_INVALID: c_int = 1;
    pub const  FE_DIVBYZERO: c_int = 2;
    pub const  FE_OVERFLOW: c_int = 4;
    pub const  FE_UNDERFLOW: c_int = 8;
    pub const  FE_INEXACT: c_int = 16;
    pub const  FE_ALL_EXCEPT: c_int = 31;

    pub const  FE_TONEAREST: c_int = 0;
    pub const  FE_DOWNWARD: c_int = 0x800000;
    pub const  FE_UPWARD: c_int = 0x400000;
    pub const  FE_TOWARDZERO: c_int = 0xc00000;
}

#[cfg(target_arch = "riscv64")]
mod  consts {
    use super::c_int;

    pub const  FE_INVALID: c_int = 16;
    pub const  FE_DIVBYZERO: c_int = 8;
    pub const  FE_OVERFLOW: c_int = 4;
    pub const  FE_UNDERFLOW: c_int = 2;
    pub const  FE_INEXACT: c_int = 1;

    pub const  FE_ALL_EXCEPT: c_int = 31;

    pub const  FE_TONEAREST: c_int = 0;
    pub const  FE_DOWNWARD: c_int = 2;
    pub const  FE_UPWARD: c_int = 3;
    pub const  FE_TOWARDZERO: c_int = 1;
}

pub use consts::*;

pub type fexcept_t = u16;
pub type fenv_t = u64;

// #[no_mangle]
pub unsafe extern "C" fn feclearexcept(excepts: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub unsafe extern "C" fn fegenenv(envp: *mut fenv_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub unsafe extern "C" fn fegetexceptflag(flagp: *mut fexcept_t, excepts: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub unsafe extern "C" fn fegetround() -> c_int {
    FE_TONEAREST
}

// #[no_mangle]
pub unsafe extern "C" fn feholdexcept(envp: *mut fenv_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub unsafe extern "C" fn feraiseexcept(except: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub unsafe extern "C" fn fesetenv(envp: *const fenv_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub unsafe extern "C" fn fesetexceptflag(flagp: *const fexcept_t, excepts: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub unsafe extern "C" fn fesetround(round: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub unsafe extern "C" fn fetestexcept(excepts: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub unsafe extern "C" fn feupdateenv(envp: *const fenv_t) -> c_int {
    unimplemented!();
}
