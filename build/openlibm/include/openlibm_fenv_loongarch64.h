/*-
 * Copyright (c) 2023 Yifan An <me@anyi.fan>
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS ``AS IS'' AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
 * OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
 * LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
 * OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
 */

#ifndef	_FENV_H_
#define	_FENV_H_

#include <stdint.h>
#include "cdefs-compat.h"

#ifndef	__fenv_static
#define	__fenv_static	static
#endif

typedef	uint32_t	fenv_t;
typedef	uint32_t	fexcept_t;

/* Exception flags */
#define	FE_INVALID	0x100000
#define	FE_DIVBYZERO	0x080000
#define	FE_OVERFLOW	0x040000
#define	FE_UNDERFLOW	0x020000
#define	FE_INEXACT	0x010000
#define	FE_ALL_EXCEPT	(FE_DIVBYZERO | FE_INEXACT | \
			 FE_INVALID | FE_OVERFLOW | FE_UNDERFLOW)

/* Rounding modes */
#define	FE_TONEAREST	0x0000
#define	FE_TOWARDZERO	0x0100
#define	FE_DOWNWARD	0x0200
#define	FE_UPWARD	0x0300
#define	_ROUND_MASK	(FE_TONEAREST | FE_DOWNWARD | \
			 FE_UPWARD | FE_TOWARDZERO)

__BEGIN_DECLS

/* Default floating-point environment */
extern const fenv_t	__fe_dfl_env;
#define	FE_DFL_ENV	(&__fe_dfl_env)

#define	_FPU_MASK_V	0x10
#define	_FPU_MASK_Z	0x08
#define	_FPU_MASK_O	0x04
#define	_FPU_MASK_U	0x02
#define	_FPU_MASK_I	0x01

#define _FPUSW_SHIFT	16
#define	_ENABLE_MASK	(_FPU_MASK_V | _FPU_MASK_Z | _FPU_MASK_O | _FPU_MASK_U | _FPU_MASK_I)

#define __rfs(__fpsr)   __asm __volatile("movfcsr2gr %0,$r0" : "=r"(__fpsr))
#define __wfs(__fpsr)   __asm __volatile("movgr2fcsr $r0,%0" : : "r"(__fpsr))

__fenv_static inline int
feclearexcept(int __excepts)
{
	fexcept_t __fpsr;

	__rfs(__fpsr);
	__fpsr &= ~__excepts;
	__wfs(__fpsr);
	return (0);
}

__fenv_static inline int
fegetexceptflag(fexcept_t *__flagp, int __excepts)
{
	fexcept_t __fpsr;

	__rfs(__fpsr);
	*__flagp = __fpsr & __excepts;
	return (0);
}

__fenv_static inline int
fesetexceptflag(const fexcept_t *__flagp, int __excepts)
{
	fexcept_t __fpsr;

	__rfs(__fpsr);
	__fpsr &= ~__excepts;
	__fpsr |= *__flagp & __excepts;
	__wfs(__fpsr);
	return (0);
}

__fenv_static inline int
feraiseexcept(int __excepts)
{
	fexcept_t __ex = __excepts;

	fesetexceptflag(&__ex, __excepts);	/* XXX */
	return (0);
}

__fenv_static inline int
fetestexcept(int __excepts)
{
	fexcept_t __fpsr;

	__rfs(__fpsr);
	return (__fpsr & __excepts);
}

__fenv_static inline int
fegetround(void)
{
	fexcept_t __fpsr;

	__rfs(__fpsr);
	return __fpsr & _ROUND_MASK;
}

__fenv_static inline int
fesetround(int __round)
{
	fexcept_t __fpsr;
	if ((__round & ~_ROUND_MASK) != 0)
		return 1;

	__rfs(__fpsr);
	__fpsr &= ~_ROUND_MASK;
	__fpsr |= __round;
	__wfs(__fpsr);

	return (0);
}

__fenv_static inline int
fegetenv(fenv_t *__envp)
{
	__rfs(*__envp);
	return (0);
}

__fenv_static inline int
feholdexcept(fenv_t *__envp)
{
	fenv_t __env;

	__rfs(__env);
	*__envp = __env;
	__env &= ~(FE_ALL_EXCEPT | _FPU_MASK_V | _FPU_MASK_Z | _FPU_MASK_O | _FPU_MASK_U | _FPU_MASK_I);
	__wfs(__env);
	return (0);
}

__fenv_static inline int
fesetenv(const fenv_t *__envp)
{
	__wfs(*__envp);
	return (0);
}

__fenv_static inline int
feupdateenv(const fenv_t *__envp)
{
	fexcept_t __fpsr;

	__rfs(__fpsr);
	__wfs(*__envp);
	feraiseexcept(__fpsr & FE_ALL_EXCEPT);
	return (0);
}

#if __BSD_VISIBLE

static inline int
feenableexcept(int __mask)
{
	fenv_t __old_fpsr, __new_fpsr;

	__rfs(__new_fpsr);
	__old_fpsr = (__new_fpsr & _ENABLE_MASK) << _FPUSW_SHIFT;
	__new_fpsr |= (__mask & FE_ALL_EXCEPT) >> _FPUSW_SHIFT;
	__wfs(__new_fpsr);
	return __old_fpsr;
}

static inline int
fedisableexcept(int __mask)
{
	fenv_t __old_fpsr, __new_fpsr;

	__rfs(__new_fpsr);
	__old_fpsr = (__new_fpsr & _ENABLE_MASK) << _FPUSW_SHIFT;
	__new_fpsr &= ~((__mask & FE_ALL_EXCEPT) >> _FPUSW_SHIFT);
	__wfs(__new_fpsr);
	return __old_fpsr;
}

static inline int
fegetexcept(void)
{
	fenv_t __fpsr;

	__rfs(__fpsr);
	return ((__fpsr & _ENABLE_MASK) << _FPUSW_SHIFT);
}

#endif /* __BSD_VISIBLE */

__END_DECLS

#endif	/* !_FENV_H_ */