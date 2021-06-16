/* Copyright 2011-2012 Nicholas J. Kain, licensed under standard MIT license */
.global _longjmp
.global longjmp
.type _longjmp,@function
.type longjmp,@function
_longjmp:
longjmp:
	mov rax,rsi								/* val will be longjmp return */
	test rax,rax
	jnz 1f
	inc rax										/* if val==0, val=1 per longjmp semantics */
1:
	mov rbx, [rdi]						/* rdi is the jmp_buf, restore regs from it */
	mov rbp, [rdi + 8]
	mov r12, [rdi + 16]
	mov r13, [rdi + 24]
	mov r14, [rdi + 32]
	mov r15, [rdi + 40]
	mov rdx, [rdi + 48] 			/* this ends up being the stack pointer */
	mov rsp, rdx 
	mov rdx, [rdi + 56]   		/* this is the instruction pointer */
	jmp rdx									  /* goto saved address without altering rsp */
