//! fenv.h implementation for Redox, following
//! http://pubs.opengroup.org/onlinepubs/9699919799/basedefs/fenv.h.html

use crate::platform::types::c_int;

mod arch;

#[cfg(target_arch = "x86_64")]
pub use arch::x86_64::*;
#[cfg(target_arch = "aarch64")]
pub use arch::aarch64::*;
#[cfg(target_arch = "riscv64")]
pub use arch::riscv64::*;

pub type fexcept_t = u16;
pub type fenv_t = u64;

// #[no_mangle]
pub unsafe extern "C" fn fegenenv(envp: *mut fenv_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub unsafe extern "C" fn fegetexceptflag(flagp: *mut fexcept_t, excepts: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub unsafe extern "C" fn feholdexcept(envp: *mut fenv_t) -> c_int {
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
pub unsafe extern "C" fn feupdateenv(envp: *const fenv_t) -> c_int {
    unimplemented!();
}
