//! fenv.h implementation for Redox, following
//! http://pubs.opengroup.org/onlinepubs/9699919799/basedefs/fenv.h.html

use crate::platform::types::*;

pub const FE_ALL_EXCEPT: c_int = 0;
pub const FE_TONEAREST: c_int = 0;

pub type fexcept_t = u64;

#[repr(C)]
pub struct fenv_t {
    pub cw: u64,
}

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

#[no_mangle]
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
