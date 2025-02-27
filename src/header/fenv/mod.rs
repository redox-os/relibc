//! ----------------------------------------------------------------------
//! Copyright © 2005-2020 Rich Felker, et al.
//!
//! Permission is hereby granted, free of charge, to any person obtaining
//! a copy of this software and associated documentation files (the
//! "Software"), to deal in the Software without restriction, including
//! without limitation the rights to use, copy, modify, merge, publish,
//! distribute, sublicense, and/or sell copies of the Software, and to
//! permit persons to whom the Software is furnished to do so, subject to
//! the following conditions:
//!
//! The above copyright notice and this permission notice shall be
//! included in all copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
//! EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
//! MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
//! IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
//! CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
//! TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
//! SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//! ----------------------------------------------------------------------

use crate::platform::types::c_int;

mod arch;

#[cfg(target_arch = "aarch64")]
pub use arch::aarch64::native::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use arch::x86::native::*;

extern "C" {
    pub fn feclearexcept(excepts: c_int) -> c_int;
    pub fn feraiseexcept(excepts: c_int) -> c_int;
    fn __fesetround(r: c_int) -> c_int;
    pub fn fegetround() -> c_int;
    pub fn fegetenv(envp: *mut fenv_t);
    pub fn fesetenv(envp: *const fenv_t) -> c_int;
    pub fn fetestexcept(excepts: c_int) -> c_int;
}

#[no_mangle]
pub unsafe extern "C" fn fegetexceptflag(flagp: *mut fexcept_t, excepts: c_int) -> c_int {
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
pub unsafe extern "C" fn fesetexceptflag(fp: *const fexcept_t, mask: c_int) -> c_int {
    feclearexcept(!(*fp as c_int) & mask);
    feraiseexcept(*fp as c_int & mask);
    0
}

#[no_mangle]
pub unsafe extern "C" fn feupdateenv(envp: *mut fenv_t) -> c_int {
    let ex = fetestexcept(FE_ALL_EXCEPT);
    fesetenv(envp);
    feraiseexcept(ex);
    0
}

pub unsafe extern "C" fn fesetround(r: c_int) -> c_int {
    if r != FE_TONEAREST && r != FE_DOWNWARD && r != FE_UPWARD && r != FE_TOWARDZERO {
        return -1;
    }

    __fesetround(r)
}
