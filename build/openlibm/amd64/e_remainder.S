/*
 * Based on the i387 version written by:
 *	J.T. Conklin (jtc@netbsd.org)
 *	Public domain.
 */

#include <amd64/bsd_asm.h>

//RCSID("from: FreeBSD: src/lib/msun/i387/e_remainder.S,v 1.8 2005/02/04 14:08:32 das Exp")
//__FBSDID("$FreeBSD: src/lib/msun/amd64/e_remainder.S,v 1.2 2011/01/07 16:13:12 kib Exp $")

ENTRY(remainder)
	movsd	%xmm0,-8(%rsp)
	movsd	%xmm1,-16(%rsp)
	fldl	-16(%rsp)
	fldl	-8(%rsp)
1:	fprem1
	fstsw	%ax
	testw	$0x400,%ax
	jne	1b
	fstpl	-8(%rsp)
	movsd	-8(%rsp),%xmm0
	fstp	%st
	ret


/* Enable stack protection */
#if defined(__ELF__)
.section .note.GNU-stack,"",%progbits
#endif
