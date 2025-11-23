/*
 * Written by:
 *	J.T. Conklin (jtc@netbsd.org)
 *	Public domain.
 */

#include <i387/bsd_asm.h>
//__FBSDID("$FreeBSD: src/lib/msun/i387/e_sqrtl.S,v 1.3 2011/01/07 16:13:12 kib Exp $")

ENTRY(sqrtl)
	fldt	4(%esp)
	fsqrt
	ret


/* Enable stack protection */
#if defined(__ELF__)
.section .note.GNU-stack,"",%progbits
#endif
