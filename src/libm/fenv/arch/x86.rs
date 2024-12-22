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
//! ARE DISCLAIMED. IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
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

    /// The following constant represents the default floating-point environment
    /// (that is, the one installed at program startup) and has type pointer to
    /// const-qualified fenv_t.
    pub static __fe_dfl_env: fenv_t = fenv_t {
        x87: fxsave_t {
            control: 0xffff0000 | INITIAL_NPXCW,
            status: 0xffff0000,
            tag: 0xffffffff,
            _reserved1: [0; 3],
            fip: 0x00000000,
            fcs: 0x0000,
            foo: 0x00000000,
            fos: 0xffff,
            mxcsr: 0, // Will be initialized below
            _reserved2: 0,
        },
        mxcsr: INITIAL_MXCSR,
    };

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

        // Set the exception flags using fesetexceptflag
        if fesetexceptflag(&(excepts as fexcept_t), excepts) != 0 {
            return -1; // Indicate error if setting flags failed (though fesetexceptflag currently doesn't fail)
        }

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
        asm!("fnstcw [{0}]", out(reg) control, options(preserves_flags));

        // Set the rounding direction in the x87 control word
        control &= !ROUND_MASK;
        control |= round;

        // Load the modified x87 control word register
        asm!("fldcw [{0}]", in(reg) control, options(preserves_flags));

        // Set the rounding direction in the SSE MXCSR register (if SSE is available)
        if has_sse() {
            asm!("stmxcsr [{0}]", in(reg) &mut mxcsr, options(preserves_flags));
            mxcsr &= !(ROUND_MASK << SSE_ROUND_SHIFT);
            mxcsr |= (round << SSE_ROUND_SHIFT) as u32;
            asm!("ldmxcsr [{0}]", in(reg) &mxcsr, options(preserves_flags));
        }
        0
    }

    /// The fegetround() function gets the current rounding direction.
    #[no_mangle]
    pub unsafe extern "C" fn fegetround() -> c_int {
        let mut control = 0;

        // Read the x87 control word to get the rounding mode
        asm!("fnstcw [{}]", out(reg) control, options(preserves_flags));

        (control & ROUND_MASK) as c_int
    }

    /// The fegetenv() function attempts to store the current floating-point
    /// environment in the object pointed to by envp.
    #[no_mangle]
    pub unsafe extern "C" fn fegetenv(envp: *mut fenv_t) -> c_int {
        // Store the current x87 floating-point environment
        asm!("fnstenv [{0}]", in(reg) &mut *envp, options(preserves_flags));

        // Store the MXCSR register state (if SSE is available)
        if has_sse() {
            asm!("stmxcsr [{0}]", in(reg) &mut (*envp).mxcsr, options(preserves_flags));
        }

        // Ensure pending exceptions are handled by reloading the x87 control word
        asm!("fldcw [{0}]", in(reg) &(*envp).x87.control, options(preserves_flags));
        0
    }

    /// The feholdexcept() function saves the current floating-point environment
    /// in the object pointed to by envp, clears the floating-point status flags, and
    /// then installs a non-stop (continue on floating-point exceptions) mode, if
    /// available, for all floating-point exceptions.
    #[no_mangle]
    pub unsafe extern "C" fn feholdexcept(envp: *mut fenv_t) -> c_int {
        let mut mxcsr = 0;

        // Store the current x87 floating-point environment
        asm!("fnstenv [{0}]", in(reg) &mut *envp, options(preserves_flags));

        // Clear all exception flags in FPU
        asm!("fnclex", options(preserves_flags));

        if has_sse() {
            // Store the MXCSR register state
            asm!("stmxcsr [{0}]", in(reg) &mut (*envp).mxcsr, options(preserves_flags));

            // Clear exception flags in MXCSR and mask all exceptions
            mxcsr = (*envp).mxcsr;
            mxcsr &= !(FE_ALL_EXCEPT as u32);
            mxcsr |= (FE_ALL_EXCEPT << SSE_MASK_SHIFT) as u32;

            // Store the modified MXCSR register
            asm!("ldmxcsr [{0}]", in(reg) &mxcsr, options(preserves_flags));
        }

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
        asm!("fnstsw [{}]", out(reg) status, options(preserves_flags));

        // Store the MXCSR register state (if SSE is available)
        if has_sse() {
            asm!("stmxcsr [{0}]", in(reg) &mut mxcsr, options(preserves_flags));
        }

        ((status as c_int) | (mxcsr as c_int)) & excepts
    }

    /// The feenableexcept() function enables the specified floating-point exceptions.
    #[no_mangle]
    pub unsafe extern "C" fn feenableexcept(mask: c_int) -> c_int {
        let mut mxcsr: u32 = 0;
        let mut omask = 0;
        let mut control: u16 = 0;
        let mask = mask & FE_ALL_EXCEPT;

        asm!("fnstcw [{}]", out(reg) control, options(preserves_flags));
        if has_sse() {
            asm!("stmxcsr [{0}]", in(reg) &mut mxcsr, options(preserves_flags));
        }

        omask = (!(control as c_int) | !((mxcsr >> SSE_MASK_SHIFT) as c_int)) & FE_ALL_EXCEPT;
        control &= !(mask as u16);
        asm!("fldcw [{0}]", in(reg) control, options(preserves_flags));

        if has_sse() {
            mxcsr &= !((mask as u32) << SSE_MASK_SHIFT);
            asm!("ldmxcsr [{0}]", in(reg) &mxcsr, options(preserves_flags));
        }

        omask
    }

    /// The fedisableexcept() function disables the specified floating-point exceptions.
    #[no_mangle]
    pub unsafe extern "C" fn fedisableexcept(mask: c_int) -> c_int {
        let mut mxcsr: u32 = 0;
        let mut omask = 0;
        let mut control: u16 = 0;
        let mask = mask & FE_ALL_EXCEPT;

        asm!("fnstcw [{}]", out(reg) control, options(preserves_flags));
        if has_sse() {
            asm!("stmxcsr [{0}]", in(reg) &mut mxcsr, options(preserves_flags));
        }

        omask = (!(control as c_int) | !((mxcsr >> SSE_MASK_SHIFT) as c_int)) & FE_ALL_EXCEPT;
        control |= mask as u16;
        asm!("fldcw [{0}]", in(reg) control, options(preserves_flags));

        if has_sse() {
            mxcsr |= (mask as u32) << SSE_MASK_SHIFT;
            asm!("ldmxcsr [{0}]", in(reg) &mxcsr, options(preserves_flags));
        }

        omask
    }

    /// The fegetexcept() function gets the currently enabled floating-point exceptions.
    #[no_mangle]
    pub unsafe extern "C" fn fegetexcept() -> c_int {
        let mut control: u16 = 0;

        // We assume that the masks for the x87 and the SSE unit are the same.
        asm!("fnstcw [{}]", out(reg) control, options(preserves_flags));

        (!(control as c_int)) & FE_ALL_EXCEPT
    }
}
