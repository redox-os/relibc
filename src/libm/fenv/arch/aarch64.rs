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

pub type fenv_t = c_ulonglong;
pub type fexcept_t = c_ulonglong;

#[no_mangle]
pub unsafe extern "C" fn feclearexcept(excepts: c_int) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn feraiseexcept(except: c_int) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn fesetenv(mut envp: *const fenv_t) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn fesetround(round: c_int) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn fegetround() -> c_int {
    FE_TONEAREST
}

#[no_mangle]
pub unsafe extern "C" fn fegetenv(envp: *const fenv_t) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn fetestexcept(excepts: c_int) -> c_int {
    0
}