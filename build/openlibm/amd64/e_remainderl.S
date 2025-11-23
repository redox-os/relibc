/*
 * Based on the i387 version written by:
 *	J.T. Conklin (jtc@netbsd.org)
 *	Public domain.
 */

#include <amd64/bsd_asm.h>

//__FBSDID("$FreeBSD: src/lib/msun/amd64/e_remainderl.S,v 1.2 2011/01/07 16:13:12 kib Exp $")

ENTRY(remainderl)
#ifndef _WIN64
	fldt	24(%rsp)
	fldt	8(%rsp)
#else
    fldt    (%r8)
    fldt    (%rdx)
#endif
1:	fprem1
	fstsw	%ax
	testw	$0x400,%ax
	jne	1b
	fstp	%st(1)
#ifdef _WIN64
    mov     %rcx,%rax
    movq    $0x0,0x8(%rcx)
    fstpt   (%rcx)
#endif
	ret


/* Enable stack protection */
#if defined(__ELF__)
.section .note.GNU-stack,"",%progbits
#endif
