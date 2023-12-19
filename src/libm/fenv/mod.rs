//! fenv.h implementation for Redox, following
//! http://pubs.opengroup.org/onlinepubs/9699919799/basedefs/fenv.h.html

use crate::platform::types::c_int;

mod arch;

#[cfg(target_arch = "aarch64")]
pub use arch::aarch64::*;
#[cfg(target_arch = "x86")]
pub use arch::x86::*;
#[cfg(target_arch = "x86_64")]
pub use arch::x86_64::*;

#[no_mangle]
pub unsafe extern "C" fn fegetexceptflag(flagp: *mut fexcept_t, mut excepts: c_int) -> c_int {
    *flagp = fetestexcept(excepts) as fexcept_t;
    0
}

#[no_mangle]
pub unsafe extern "C" fn feholdexcept(envp: *mut fenv_t) -> c_int {
    fegetenv(envp);
    feclearexcept(FE_ALL_EXCEPT);
    0
}

#[no_mangle]
pub unsafe extern "C" fn fesetexceptflag(flagp: *const fexcept_t, excepts: c_int) -> c_int {
    feclearexcept(!(*flagp as c_int) & excepts);
    feraiseexcept(*flagp as c_int & excepts);
    0
}

#[no_mangle]
pub unsafe extern "C" fn feupdateenv(envp: *mut fenv_t) -> c_int {
    let mut ex = fetestexcept(FE_ALL_EXCEPT);
    fesetenv(envp);
    feraiseexcept(ex);
    0
}
