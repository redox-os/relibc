#include <amd64/bsd_asm.h>
//__FBSDID("$FreeBSD: src/lib/msun/amd64/s_llrintf.S,v 1.3 2011/02/04 21:54:06 kib Exp $")

ENTRY(llrintf)
	cvtss2si %xmm0, %rax
	ret
END(llrintf)

/* Enable stack protection */
#if defined(__ELF__)
.section .note.GNU-stack,"",%progbits
#endif
