.global sigsetjmp
.global __sigsetjmp
.type sigsetjmp,@function
.type __sigsetjmp,@function
sigsetjmp:
__sigsetjmp:
	test esi, esi
	jz 1f

	pop [rdi + 64]
	mov qword ptr [rdi * 8 + 72], rbx
	mov rbx, rdi

	call setjmp

	push [rbx + 64]
	mov rdi, rbx
	mov esi, eax
	mov rbx, qword ptr [rbx * 8 + 72]

.hidden __sigsetjmp_tail
	jmp __sigsetjmp_tail

1:	jmp setjmp
