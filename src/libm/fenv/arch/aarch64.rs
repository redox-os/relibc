#![allow(dead_code)]

//! $OpenBSD: fenv.c,v 1.6 2022/12/27 17:10:07 jmc Exp $
//! Copyright (c) 2004-2005 David Schultz <das@FreeBSD.ORG>
//! All rights reserved.
//! Redistribution and use in source and binary forms, with or without
//! modification, are permitted provided that the following conditions
//! are met:
//! 1. Redistributions of source code must retain the above copyright
//!    notice, this list of conditions and the following disclaimer.
//! 2. Redistributions in binary form must reproduce the above copyright
//!    notice, this list of conditions and the following disclaimer in the
//!    documentation and/or other materials provided with the distribution.
//! THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS ``AS IS'' AND
//! ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
//! IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
//! ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
//! FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
//! DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
//! OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
//! HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
//! LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
//! OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
//! SUCH DAMAGE.
//! $FreeBSD: head/lib/msun/aarch64/fenv.h 280857 2015-03-30 16:42:08Z emaste $

#[cfg(target_arch = "aarch64")]
pub mod native {
    use crate::platform::types::*;
    use core::arch::asm;

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

    const ROUND_MASK: c_int = FE_TONEAREST | FE_UPWARD | FE_DOWNWARD | FE_TOWARDZERO;
    const ROUND_SHIFT: c_int = 22;
    const FPUSW_SHIFT: c_int = 8;
    const ENABLE_MASK: c_int = FE_ALL_EXCEPT << FPUSW_SHIFT;

    pub type fenv_t = c_ulonglong;
    pub type fexcept_t = c_ulonglong;

    #[inline(always)]
    unsafe fn mrs_fpcr(mut r: fexcept_t) {
        asm!("mrs {0:e}, fpcr", in (reg) & mut r, options(preserves_flags));
    }

    #[inline(always)]
    unsafe fn msr_fpcr(r: fexcept_t) {
        asm!("msr fpcr, {0:r}", inlateout(reg) r => _, options(preserves_flags));
    }

    #[inline(always)]
    unsafe fn mrs_fpsr(mut r: fexcept_t) {
        asm!("mrs {0:r}, fpsr", in (reg) & mut r, options(preserves_flags));
    }

    #[inline(always)]
    unsafe fn msr_fpsr(r: fexcept_t) {
        asm!("msr fpsr, {0:r}", inlateout(reg) r => _, options(preserves_flags));
    }

    /// The feclearexcept() function clears the supported floating-point exceptions
    /// represented by `excepts'.
    #[no_mangle]
    pub unsafe extern "C" fn feclearexcept(excepts: c_int) -> c_int {
        let mut r = 0;
        let excepts = (excepts & FE_ALL_EXCEPT) as fexcept_t;
        mrs_fpsr(r);
        r &= !excepts;
        msr_fpsr(r);
        0
    }

    /// The feraiseexcept() function raises the supported floating-point exceptions
    /// represented by the argument `excepts'.
    /// The standard explicitly allows us to execute an instruction that has the
    /// exception as a side effect, but we choose to manipulate the status register
    /// directly.
    /// The validation of input is being deferred to fesetexceptflag().
    #[no_mangle]
    pub unsafe extern "C" fn feraiseexcept(excepts: c_int) -> c_int {
        let mut r = 0;
        let excepts = (excepts & FE_ALL_EXCEPT) as fexcept_t;
        mrs_fpsr(r);
        r |= excepts;
        msr_fpsr(r);
        0
    }

    /// This function sets the floating-point status flags indicated by the argument
    /// `excepts' to the states stored in the object pointed to by `flagp'. It does
    /// NOT raise any floating-point exceptions, but only sets the state of the flags.
    #[no_mangle]
    pub unsafe extern "C" fn fesetexceptflag(flagp: *const fexcept_t, excepts: c_int) -> c_int {
        let mut r = 0;
        let excepts = (excepts & FE_ALL_EXCEPT) as fexcept_t;
        mrs_fpsr(r);
        r |= !excepts;
        r |= *flagp & excepts;
        msr_fpsr(r);
        0
    }

    /// The fesetenv() function attempts to establish the floating-point environment
    /// represented by the object pointed to by envp. The argument `envp' points
    /// to an object set by a call to fegetenv() or feholdexcept(), or equal a
    /// floating-point environment macro. The fesetenv() function does not raise
    /// floating-point exceptions, but only installs the state of the floating-point
    /// status flags represented through its argument.
    #[no_mangle]
    pub unsafe extern "C" fn fesetenv(envp: *const fenv_t) -> c_int {
        msr_fpcr(*envp & 0xffffffff);
        msr_fpsr(*envp >> 32);
        0
    }

    /// The fesetround() function establishes the rounding direction represented by
    /// its argument `round'. If the argument is not equal to the value of a rounding
    /// direction macro, the rounding direction is not changed.
    #[no_mangle]
    pub unsafe extern "C" fn fesetround(round: c_int) -> c_int {
        let mut r = 0;
        if (round & !ROUND_MASK) != 0 {
            return -(1 as c_int);
        }

        mrs_fpcr(r);
        r &= (!ROUND_MASK << ROUND_SHIFT) as fenv_t;
        r |= (round << ROUND_SHIFT) as fenv_t;
        msr_fpcr(r);
        0
    }

    /// The fegetround() function gets the current rounding direction.
    #[no_mangle]
    pub unsafe extern "C" fn fegetround() -> c_int {
        let r = 0;
        mrs_fpcr(r);
        (r as c_int >> ROUND_SHIFT) & ROUND_MASK
    }

    /// The fegetenv() function attempts to store the current floating-point
    /// environment in the object pointed to by envp.
    #[no_mangle]
    pub unsafe extern "C" fn fegetenv(envp: *mut fenv_t) -> c_int {
        let r = 0;
        mrs_fpcr(r);
        *envp = r;
        mrs_fpsr(r);
        *envp |= r << 32;

        0
    }

    /// The fetestexcept() function determines which of a specified subset of the
    /// floating-point exception flags are currently set. The `excepts' argument
    /// specifies the floating-point status flags to be queried.
    #[no_mangle]
    pub unsafe extern "C" fn fetestexcept(excepts: c_int) -> c_int {
        let r = 0;

        let excepts = excepts & FE_ALL_EXCEPT;
        mrs_fpsr(r);
        r as c_int & excepts
    }
}
