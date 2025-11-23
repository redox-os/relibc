/*
 * Written by:
 *	J.T. Conklin (jtc@netbsd.org)
 *	Public domain.
 */

#include <i387/bsd_asm.h>
//__FBSDID("$FreeBSD: src/lib/msun/i387/e_sqrt.S,v 1.10 2011/01/07 16:13:12 kib Exp $")

ENTRY(sqrt)
	pushl   %ebp
	movl	%esp,%ebp
	subl	$8,%esp

	fstcw	-4(%ebp)		/* store fpu control word */
	movw    -4(%ebp),%dx
	andw    $0xfeff,%dx 	/* Set precision field to 64 bits (53 bit mantissa).
	                           We assume it's set to 0b11 (extended precision),
							   so zeroing out the low bit of the precision field,
							   will correctly set the precision */
	movw	%dx,-8(%ebp)
	fldcw	-8(%ebp)		/* load modfied control word */

	fldl	8(%ebp)
	fsqrt

	fldcw	-4(%ebp)		/* restore original control word */

	leave
	ret
END(sqrt)


/* Enable stack protection */
#if defined(__ELF__)
.section .note.GNU-stack,"",%progbits
#endif
