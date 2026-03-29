.global sigsetjmp
.global __sigsetjmp
.type sigsetjmp,@function
.type __sigsetjmp,@function
sigsetjmp:
__sigsetjmp:
	mov ecx, dword ptr [esp + 8]
	jecxz 1f

	mov eax, dword ptr [esp + 4]
	pop [eax + 24]
	mov dword ptr [eax * 8 + 28], ebx
	mov ebx, eax

.hidden ___setjmp
	call ___setjmp

	push [ebx + 24]
	mov dword ptr [esp + 4], ebx
	mov dword ptr [esp + 4], eax
	mov ebx, dword ptr [ebx * 8 + 28]

.hidden __sigsetjmp_tail
	jmp __sigsetjmp_tail

1:	jmp ___setjmp
