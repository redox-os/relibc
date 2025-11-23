/*
 * Written by:
 *	J.T. Conklin (jtc@netbsd.org)
 *	Public domain.
 */

#include <i387/bsd_asm.h>
//__FBSDID("$FreeBSD: src/lib/msun/i387/s_rintl.S,v 1.3 2011/01/07 16:13:12 kib Exp $")

ENTRY(rintl)
	fldt	4(%esp)
	frndint
	ret


/* Enable stack protection */
#if defined(__ELF__)
.section .note.GNU-stack,"",%progbits
#endif
