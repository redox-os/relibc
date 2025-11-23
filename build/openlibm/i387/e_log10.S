/*
 * Written by:
 *	J.T. Conklin (jtc@netbsd.org)
 *	Public domain.
 */

#include <i387/bsd_asm.h>
//__FBSDID("$FreeBSD: src/lib/msun/i387/e_log10.S,v 1.10 2011/01/07 16:13:12 kib Exp $")

ENTRY(log10)
	fldlg2
	fldl	4(%esp)
	fyl2x
	ret
END(log10)


/* Enable stack protection */
#if defined(__ELF__)
.section .note.GNU-stack,"",%progbits
#endif
