.global sigsetjmp
.global _sigsetjmp
.global __sigsetjmp
.type sigsetjmp,@function
.type _sigsetjmp,@function
.type __sigsetjmp,@function
sigsetjmp:
_sigsetjmp:
__sigsetjmp:
	test esi,esi
	jz 1f

	pop qword ptr [rdi+64]
	mov qword ptr [rdi+80],rbx
	mov rbx,rdi

	call setjmp

	push qword ptr [rbx+64]
	mov rdi,rbx
	mov esi,eax
	mov rbx, qword ptr[rbx+80]

	jmp __sigsetjmp_tail

1:	jmp setjmp
