/*
 * Written by:
 *	J.T. Conklin (jtc@netbsd.org)
 *	Public domain.
 */

#include <i387/bsd_asm.h>
//__FBSDID("$FreeBSD: src/lib/msun/i387/s_scalbn.S,v 1.10 2011/01/07 16:13:12 kib Exp $")

ENTRY(scalbn)
	fildl	12(%esp)
	fldl	4(%esp)
	fscale
	fstp	%st(1)
	ret
END(scalbn)

.globl CNAME(ldexp)
.set   CNAME(ldexp),CNAME(scalbn)
/* Enable stack protection */
#if defined(__ELF__)
.section .note.GNU-stack,"",%progbits
#endif
