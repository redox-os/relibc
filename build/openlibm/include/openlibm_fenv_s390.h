/*-
 * Copyright (c) 2016 Dan Horák <dan[at]danny.cz>
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
 *
 * $FreeBSD$
 */

#ifndef	_FENV_H_
#define	_FENV_H_

#include <stdint.h>
#include <sys/types.h>
#include "cdefs-compat.h"

#ifndef	__fenv_static
#define	__fenv_static	static
#endif

typedef	uint32_t	fenv_t;
typedef	uint32_t	fexcept_t;

/* Exception flags */
#define	FE_INEXACT	0x080000
#define	FE_UNDERFLOW	0x100000
#define	FE_OVERFLOW	0x200000
#define	FE_DIVBYZERO	0x400000
#define	FE_INVALID	0x800000	/* all types of invalid FP ops */

#define	FE_ALL_EXCEPT	(FE_INVALID | FE_DIVBYZERO | FE_INEXACT | FE_OVERFLOW | FE_UNDERFLOW)

/* Rounding modes */
#define	FE_TONEAREST	0x0000
#define	FE_TOWARDZERO	0x0001
#define	FE_UPWARD	0x0002
#define	FE_DOWNWARD	0x0003
#define	_ROUND_MASK	(FE_TONEAREST | FE_DOWNWARD | \
			 FE_UPWARD | FE_TOWARDZERO)

__BEGIN_DECLS

/* Default floating-point environment */
extern const fenv_t	__fe_dfl_env;
#define	FE_DFL_ENV	(&__fe_dfl_env)

/* We need to be able to map status flag positions to mask flag positions */
#define	_FPC_EXC_MASK_SHIFT	8
#define	_ENABLE_MASK	((FE_DIVBYZERO | FE_INEXACT | FE_INVALID | \
			 FE_OVERFLOW | FE_UNDERFLOW) << _FPC_EXC_MASK_SHIFT)

/* Macros for accessing the hardware control word.  */
#define _FPU_GETCW(cw)  __asm__ __volatile__ ("efpc %0,0" : "=d" (cw))
#define _FPU_SETCW(cw)  __asm__ __volatile__ ("sfpc  %0,0" : : "d" (cw))

__fenv_static inline int
feclearexcept(int __excepts)
{
	fexcept_t __r;

	if (__excepts & FE_INVALID)
		__excepts |= FE_ALL_EXCEPT;
	_FPU_GETCW(__r);
	__r &= ~__excepts;
	_FPU_SETCW(__r);
	return (0);
}

__fenv_static inline int
fegetexceptflag(fexcept_t *__flagp, int __excepts)
{
	fexcept_t __r;

	_FPU_GETCW(__r);
	*__flagp = __r & __excepts;
	return (0);
}

__fenv_static inline int
fesetexceptflag(const fexcept_t *__flagp, int __excepts)
{
	fexcept_t __r;

	if (__excepts & FE_INVALID)
		__excepts |= FE_ALL_EXCEPT;
	_FPU_GETCW(__r);
	__r &= ~__excepts;
	__r |= *__flagp & __excepts;
	_FPU_SETCW(__r);
	return (0);
}

__fenv_static inline int
feraiseexcept(int __excepts)
{
	fexcept_t __r;

	_FPU_GETCW(__r);
	__r |= __excepts;
	_FPU_SETCW(__r);
	return (0);
}

__fenv_static inline int
fetestexcept(int __excepts)
{
	fexcept_t __r;

	_FPU_GETCW(__r);
	return (__r & __excepts);
}

__fenv_static inline int
fegetround(void)
{
	fexcept_t __r;

	_FPU_GETCW(__r);
	return (__r & _ROUND_MASK);
}

__fenv_static inline int
fesetround(int __round)
{
	fexcept_t __r;

	if (__round & ~_ROUND_MASK)
		return (-1);

	_FPU_GETCW(__r);
	__r &= ~_ROUND_MASK;
	__r |= __round;
	_FPU_SETCW(__r);
	return (0);
}

__fenv_static inline int
fegetenv(fenv_t *__envp)
{
	_FPU_GETCW(*__envp);
	return (0);
}

__fenv_static inline int
feholdexcept(fenv_t *__envp)
{
	fexcept_t __r;

	_FPU_GETCW(__r);
	*__envp = __r;
	__r &= ~(FE_ALL_EXCEPT | _ENABLE_MASK);
	_FPU_SETCW(__r);
	return (0);
}

__fenv_static inline int
fesetenv(const fenv_t *__envp)
{
	_FPU_SETCW(*__envp);
	return (0);
}

__fenv_static inline int
feupdateenv(const fenv_t *__envp)
{
	fexcept_t __r;

	_FPU_GETCW(__r);
	__r &= FE_ALL_EXCEPT;
	__r |= *__envp;
	_FPU_SETCW(__r);
	return (0);
}

#if __BSD_VISIBLE

/* We currently provide no external definitions of the functions below. */

static inline int
feenableexcept(int __mask)
{
	fenv_t __r;
	fenv_t __oldmask;

	_FPU_GETCW(__r);
	__oldmask = __r;
	__r |= (__mask & FE_ALL_EXCEPT) << _FPC_EXC_MASK_SHIFT;
	_FPU_SETCW(__r);
	return ((__oldmask & _ENABLE_MASK) >> _FPC_EXC_MASK_SHIFT);
}

static inline int
fedisableexcept(int __mask)
{
	fenv_t __r;
	fenv_t __oldmask;

	_FPU_GETCW(__r);
	__oldmask = __r;
	__r &= ~((__mask & FE_ALL_EXCEPT) << _FPC_EXC_MASK_SHIFT);
	_FPU_SETCW(__r);
	return ((__oldmask & _ENABLE_MASK) >> _FPC_EXC_MASK_SHIFT);
}

static inline int
fegetexcept(void)
{
	fexcept_t __r;

	_FPU_GETCW(__r);
	return (__r & (_ENABLE_MASK >> _FPC_EXC_MASK_SHIFT));
}

#endif /* __BSD_VISIBLE */

__END_DECLS

#endif	/* !_FENV_H_ */
