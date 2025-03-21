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
//! ARE DISCLAIMED. IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
//! FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
//! DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
//! OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
//! HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
//! LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
//! OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
//! SUCH DAMAGE.
//! $FreeBSD: head/lib/msun/aarch64/fenv.h 280857 2015-03-30 16:42:08Z emaste $

use core::arch::global_asm;

#[cfg(target_arch = "x86_64")]
global_asm!(include_str!("x86_64.S"));

#[cfg(target_arch = "x86")]
global_asm!(include_str!("x86.S"));

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod native {
    use crate::platform::types::{c_int, c_uint};

    // Exception flags
    pub const FE_INVALID: i32 = 0x01;
    pub const FE_DENORMAL: i32 = 0x02;
    pub const FE_DIVBYZERO: i32 = 0x04;
    pub const FE_OVERFLOW: i32 = 0x08;
    pub const FE_UNDERFLOW: i32 = 0x10;
    pub const FE_INEXACT: i32 = 0x20;
    pub const FE_ALL_EXCEPT: i32 = 0x3F;

    // Rounding modes
    pub const FE_TONEAREST: i32 = 0x0000;
    pub const FE_DOWNWARD: i32 = 0x0400;
    pub const FE_UPWARD: i32 = 0x0800;
    pub const FE_TOWARDZERO: i32 = 0x0C00;

    pub const ROUND_MASK: c_int = FE_TONEAREST | FE_DOWNWARD | FE_UPWARD | FE_TOWARDZERO;
    pub const SSE_ROUND_SHIFT: c_int = 3;
    pub const SSE_MASK_SHIFT: c_int = 7;

    const INITIAL_NPXCW: c_uint = 0x037f;
    const INITIAL_MXCSR: c_uint = 0x1f80;

    #[repr(C, align(16))]
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

    /// The following constant represents the default floating-point environment
    /// (that is, the one installed at program startup) and has type pointer to
    /// const-qualified fenv_t.
    pub static __fe_dfl_env: fenv_t = fenv_t {
        x87: X87Reg {
            control: 0xffff0000 | INITIAL_NPXCW,
            status: 0xffff0000,
            tag: 0xffffffff,
            others: [0x00000000, 0x00000000, 0x00000000, 0xffff0000],
        },
        mxcsr: INITIAL_MXCSR,
    };
}
