/*
 * Based on code written by J.T. Conklin <jtc@netbsd.org>.
 * Public domain.
 */

#include <amd64/bsd_asm.h>
//__FBSDID("$FreeBSD: src/lib/msun/amd64/s_scalbnl.S,v 1.3 2011/01/07 16:13:12 kib Exp $")
/* //RCSID("$NetBSD: s_scalbnf.S,v 1.4 1999/01/02 05:15:40 kristerw Exp $") */

ENTRY(scalbnl)
#ifndef _WIN64
	movl	%edi,-4(%rsp)
	fildl	-4(%rsp)
	fldt	8(%rsp)
#else
    mov     %r8,%rax
	movl	%eax,-4(%rsp)
	fildl	-4(%rsp)
	fldt	(%rdx)
#endif
	fscale
	fstp	%st(1)
#ifdef _WIN64
    mov     %rcx,%rax
    movq    $0x0,0x8(%rcx)
    fstpt   (%rcx)
#endif
	ret
#ifndef _WIN64
END(scalbnl)
.globl CNAME(ldexpl)
#else
.globl CNAME(ldexpl); .section .drectve; .ascii " -export:ldexpl"
#endif
.set   CNAME(ldexpl),CNAME(scalbnl)

/* Enable stack protection */
#if defined(__ELF__)
.section .note.GNU-stack,"",%progbits
#endif
