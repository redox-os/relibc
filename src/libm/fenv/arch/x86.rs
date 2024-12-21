#![allow(dead_code)]

//!	$OpenBSD: fenv.c,v 1.7 2022/12/27 17:10:07 jmc Exp $
//!	$NetBSD: fenv.c,v 1.3 2010/08/01 06:34:38 taca Exp $
//! Copyright (c) 2004-2005 David Schultz <das@FreeBSD.ORG>
//! All rights reserved.
//!
//! Redistribution and use in source and binary forms, with or without
//! modification, are permitted provided that the following conditions
//! are met:
//! 1. Redistributions of source code must retain the above copyright
//!    notice, this list of conditions and the following disclaimer.
//! 2. Redistributions in binary form must reproduce the above copyright
//!    notice, this list of conditions and the following disclaimer in the
//!    documentation and/or other materials provided with the distribution.
//!
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

#[cfg(target_arch = "x86")]
pub mod native {
    use crate::{libm::fenv::x86_common::*, platform::types::*};
    use core::arch::asm;
    use raw_cpuid::CpuId;

    fn has_sse() -> bool {
        let cpuid = CpuId::new();
        cpuid
            .get_feature_info()
            .map_or(false, |finfo| finfo.has_sse())
    }

    /// The feclearexcept() function clears the supported floating-point exceptions
    /// represented by `excepts'.
    #[no_mangle]
    pub unsafe extern "C" fn feclearexcept(excepts: c_int) -> c_int {
        let mut fenv: fenv_t = Default::default();
        let mut mxcsr = 0;
        let excepts = excepts & FE_ALL_EXCEPT;

        // Store the current x87 floating-point environment
        asm!("fnstenv [{0}]", in(reg) &mut fenv, options(preserves_flags));

        // Clear the requested floating-point exceptions in the x87 FPU status word
        fenv.x87.status &= !(excepts as c_uint);

        // Load the modified x87 floating-point environment
        asm!("fldenv [{0}]", in(reg) &fenv, options(preserves_flags));

        // Clear the requested floating-point exceptions in the SSE MXCSR register (if SSE is available)
        if has_sse() {
            asm!("stmxcsr [{0}]", in(reg) &mut mxcsr, options(preserves_flags));
            mxcsr &= !excepts;
            asm!("ldmxcsr [{0}]", in(reg) &mxcsr, options(preserves_flags));
        }
        0
    }

    /// The feraiseexcept() function raises the supported floating-point exceptions
    /// represented by the argument `excepts'.
    /// The standard explicitly allows us to execute an instruction that has the
    /// exception as a side effect, but we choose to manipulate the status register
    /// directly.
    /// The validation of input is being deferred to fesetexceptflag().
    #[no_mangle]
    pub unsafe extern "C" fn feraiseexcept(except: c_int) -> c_int {
        let excepts = except & FE_ALL_EXCEPT;

        // Validate input to ensure only supported exceptions are raised
        if excepts == 0 {
            return -1;
        }

        // Set the exception flags using fesetexceptflag
        fesetexceptflag(excepts as *const fexcept_t, excepts);

        // Ensure pending exceptions are handled
        asm!("fwait", options(preserves_flags));

        0
    }

    /// This function sets the floating-point status flags indicated by the argument
    /// `excepts' to the states stored in the object pointed to by `flagp'. It does
    /// NOT raise any floating-point exceptions, but only sets the state of the flags.
    #[no_mangle]
    pub unsafe extern "C" fn fesetexceptflag(flagp: *const fexcept_t, excepts: c_int) -> c_int {
        let mut fenv: fenv_t = Default::default();
        let mut mxcsr = 0;
        let excepts = excepts & FE_ALL_EXCEPT;

        // Store the current x87 floating-point environment
        asm!("fnstenv [{0}]", in(reg) &mut fenv, options(preserves_flags));

        // Set the requested status flags in the x87 FPU status word
        fenv.x87.status &= !(excepts as c_uint);
        fenv.x87.status |= *flagp & (excepts as c_uint);

        // Load the modified x87 floating-point environment
        asm!("fldenv [{0}]", in(reg) &fenv, options(preserves_flags));

        // Set the requested status flags in the SSE MXCSR register (if SSE is available)
        if has_sse() {
            asm!("stmxcsr [{0}]", in(reg) &mut mxcsr, options(preserves_flags));
            mxcsr &= !(excepts as c_uint);
            mxcsr |= *flagp & (excepts as c_uint);
            asm!("ldmxcsr [{0}]", in(reg) &mxcsr, options(preserves_flags));
        }
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
        // Load the x87 floating-point environment
        asm!("fldenv [{0}]", in(reg) &*envp, options(preserves_flags));

        // Load the MXCSR register (if SSE is available)
        if has_sse() {
            asm!("ldmxcsr [{0}]", in(reg) &(*envp).mxcsr, options(preserves_flags));
        }
        0
    }

    /// The fesetround() function establishes the rounding direction represented by
    /// its argument `round'. If the argument is not equal to the value of a rounding
    /// direction macro, the rounding direction is not changed.
    #[no_mangle]
    pub unsafe extern "C" fn fesetround(round: c_int) -> c_int {
        let mut control = 0;
        let mut mxcsr = 0;

        // Check whether the requested rounding direction is supported
        if round & !ROUND_MASK != 0 {
            return -1;
        }

        // Store the current x87 control word register
        asm!("fnstcw [{0}]", in(reg) &mut control, options(preserves_flags));

        // Set the rounding direction in the x87 control word
        control &= !ROUND_MASK;
        control |= round;

        // Load the modified x87 control word register
        asm!("fldcw [{0}]", in(reg) &control, options(preserves_flags));

        // Set the rounding direction in the SSE MXCSR register (if SSE is available)
        if has_sse() {
            asm!("stmxcsr [{0}]", in(reg) &mut mxcsr, options(preserves_flags));
            mxcsr &= !(ROUND_MASK << SSE_ROUND_SHIFT);
            mxcsr |= round << SSE_ROUND_SHIFT;
            asm!("ldmxcsr [{0}]", in(reg) &mxcsr, options(preserves_flags));
        }
        0
    }

    /// The fegetround() function gets the current rounding direction.
    #[no_mangle]
    pub unsafe extern "C" fn fegetround() -> c_int {
        let mut control = 0;

        // Read the x87 control word to get the rounding mode
        asm!("fnstcw [{0}]", in(reg) &mut control, options(preserves_flags));

        control & ROUND_MASK
    }

    /// The fegetenv() function attempts to store the current floating-point
    /// environment in the object pointed to by envp.
    #[no_mangle]
    pub unsafe extern "C" fn fegetenv(envp: *mut fenv_t) -> c_int {
        // Store the current x87 floating-point environment
        asm!("fnstenv [{0}]", in(reg) &mut *envp, options(preserves_flags));

        // Store the MXCSR register state (if SSE is available)
        if has_sse() {
            asm!("stmxcsr [{0}]", in(reg) &(*envp).mxcsr, options(preserves_flags));
        }

        // Ensure pending exceptions are handled by reloading the x87 control word
        asm!("fldcw [{0}]", in(reg) &(*envp).x87.control, options(preserves_flags));
        0
    }

    /// The fetestexcept() function determines which of a specified subset of the
    /// floating-point exception flags are currently set. The `excepts' argument
    /// specifies the floating-point status flags to be queried.
    #[no_mangle]
    pub unsafe extern "C" fn fetestexcept(excepts: c_int) -> c_int {
        let mut status = 0;
        let mut mxcsr = 0;
        let excepts = excepts & FE_ALL_EXCEPT;

        // Store the current x87 status register
        asm!("fnstsw [{}]", in(reg) &mut status);

        // Store the MXCSR register state (if SSE is available)
        if has_sse() {
            asm!("stmxcsr [{0}]", in(reg) &mut mxcsr, options(preserves_flags));
        }

        (status | mxcsr) & excepts
    }
}
