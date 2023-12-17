use crate::platform::types::c_int;
use crate::libm::fenv::{fexcept_t, fenv_t};

pub const FE_INVALID: c_int = 1;
pub const FE_DIVBYZERO: c_int = 2;
pub const FE_OVERFLOW: c_int = 4;
pub const FE_UNDERFLOW: c_int = 8;
pub const FE_INEXACT: c_int = 16;
pub const FE_ALL_EXCEPT: c_int = 31;

pub const FE_TONEAREST: c_int = 0;
pub const FE_DOWNWARD: c_int = 0x800000;
pub const FE_UPWARD: c_int = 0x400000;
pub const FE_TOWARDZERO: c_int = 0xc00000;


// #[no_mangle]
pub unsafe extern "C" fn feclearexcept(excepts: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub unsafe extern "C" fn feraiseexcept(except: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub unsafe extern "C" fn fesetround(round: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub unsafe extern "C" fn fegetround() -> c_int {
    FE_TONEAREST
}

// #[no_mangle]
pub unsafe extern "C" fn fegetenv(envp: *const fenv_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub unsafe extern "C" fn fetestexcept(excepts: c_int) -> c_int {
    unimplemented!();
}

