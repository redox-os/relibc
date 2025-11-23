/*
 * Written by:
 *	J.T. Conklin (jtc@netbsd.org)
 *	Public domain.
 */

#include <amd64/bsd_asm.h>
//__FBSDID("$FreeBSD: src/lib/msun/amd64/s_logbl.S,v 1.4 2011/01/07 16:13:12 kib Exp $")

ENTRY(logbl)
#ifndef _WIN64
	fldt	8(%rsp)
#else
    fldt    (%rdx)
#endif
	fxtract
	fstp	%st
#ifdef _WIN64
    mov     %rcx,%rax
    movq    $0x0,0x8(%rcx)
	fstpt	(%rcx)
#endif
	ret


/* Enable stack protection */
#if defined(__ELF__)
.section .note.GNU-stack,"",%progbits
#endif
