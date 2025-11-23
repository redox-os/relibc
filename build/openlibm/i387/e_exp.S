/*
 * Written by:
 *	J.T. Conklin (jtc@netbsd.org)
 *	Public domain.
 */

#include <i387/bsd_asm.h>

/* e^x = 2^(x * log2(e)) */

ENTRY(exp)
	/*
	 * If x is +-Inf, then the subtraction would give Inf-Inf = NaN.
	 * Avoid this.  Also avoid it if x is NaN for convenience.
	 */
	movl	8(%esp),%eax
	andl	$0x7fffffff,%eax
	cmpl	$0x7ff00000,%eax
	jae	x_Inf_or_NaN

	fldl	4(%esp)

	/*
	 * Extended precision is needed to reduce the maximum error from
	 * hundreds of ulps to less than 1 ulp.  Switch to it if necessary.
	 * We may as well set the rounding mode to to-nearest and mask traps
	 * if we switch.
	 */
	fstcw	4(%esp)
	movl	4(%esp),%eax
	andl	$0x0300,%eax
	cmpl	$0x0300,%eax		/* RC == 0 && PC == 3? */
	je	1f			/* jump if mode is good */
	movl	$0x137f,8(%esp)
	fldcw	8(%esp)
1:
	fldl2e
	fmulp				/* x * log2(e) */
	fst	%st(1)
	frndint				/* int(x * log2(e)) */
	fst	%st(2)
	fsubrp				/* fract(x * log2(e)) */
	f2xm1				/* 2^(fract(x * log2(e))) - 1 */
	fld1
	faddp				/* 2^(fract(x * log2(e))) */
	fscale				/* e^x */
	fstp	%st(1)
	je	1f
	fldcw	4(%esp)
1:
	ret

x_Inf_or_NaN:
	/*
	 * Return 0 if x is -Inf.  Otherwise just return x; when x is Inf
	 * this gives Inf, and when x is a NaN this gives the same result
	 * as (x + x) (x quieted).
	 */
	cmpl	$0xfff00000,8(%esp)
	jne	x_not_minus_Inf
	cmpl	$0,4(%esp)
	jne	x_not_minus_Inf
	fldz
	ret

x_not_minus_Inf:
	fldl	4(%esp)
	ret
END(exp)

    //

/* Enable stack protection */
#if defined(__ELF__)
.section .note.GNU-stack,"",%progbits
#endif
