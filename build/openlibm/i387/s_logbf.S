/*
 * Written by J.T. Conklin <jtc@netbsd.org>.
 * Public domain.
 */

#include <i387/bsd_asm.h>

//__FBSDID("$FreeBSD: src/lib/msun/i387/s_logbf.S,v 1.3 2011/01/07 16:13:12 kib Exp $");
/* RCSID("$NetBSD: s_logbf.S,v 1.3 1995/05/09 00:15:12 jtc Exp $") */

ENTRY(logbf)
	flds	4(%esp)
	fxtract
	fstp	%st
	ret
END(logbf)


/* Enable stack protection */
#if defined(__ELF__)
.section .note.GNU-stack,"",%progbits
#endif
