/* Copyright 2011-2012 Nicholas J. Kain, licensed under standard MIT license */
.global __setjmp
.global _setjmp
.global setjmp
.type __setjmp,@function
.type _setjmp,@function
.type setjmp,@function
__setjmp:
_setjmp:
setjmp:
	mov [rdi], rbx						/* rdi is jmp_buf, move registers onto it */
	mov [rdi + 8], rbp
	mov [rdi + 16], r12
	mov [rdi + 24], r13
	mov [rdi + 32], r14
	mov [rdi + 40], r15
	lea rdx, [rsp + 8]				/* this is our rsp WITHOUT current ret addr */
	mov [rdi + 48], rdx
	mov rdx, [rsp]						/* save return addr ptr for new rip */
	mov [rdi + 56], rdx
	xor rax, rax							/* always return 0 */
	ret
