/*
 * Written by:
 *	J.T. Conklin (jtc@netbsd.org)
 *	Public domain.
 */

#include <i387/bsd_asm.h>
//__FBSDID("$FreeBSD: src/lib/msun/i387/s_logbl.S,v 1.3 2011/01/07 16:13:12 kib Exp $")

ENTRY(logbl)
	fldt	4(%esp)
	fxtract
	fstp	%st
	ret


/* Enable stack protection */
#if defined(__ELF__)
.section .note.GNU-stack,"",%progbits
#endif
