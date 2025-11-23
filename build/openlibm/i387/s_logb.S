/*
 * Written by:
 *	J.T. Conklin (jtc@netbsd.org)
 *	Public domain.
 */

#include <i387/bsd_asm.h>
//__FBSDID("$FreeBSD: src/lib/msun/i387/s_logb.S,v 1.10 2011/01/07 16:13:12 kib Exp $")

ENTRY(logb)
	fldl	4(%esp)
	fxtract
	fstp	%st
	ret
END(logb)


/* Enable stack protection */
#if defined(__ELF__)
.section .note.GNU-stack,"",%progbits
#endif
