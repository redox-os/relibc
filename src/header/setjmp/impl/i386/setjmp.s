.global ___setjmp
.hidden ___setjmp
.global __setjmp
.global _setjmp
.global setjmp
.type __setjmp,@function
.type _setjmp,@function
.type setjmp,@function
___setjmp:
__setjmp:
_setjmp:
setjmp:
	mov eax, [esp + 4]
	mov [eax], ebx
	mov [eax + 4], esi
	mov [eax + 8], edi
	mov [eax + 12], ebp
	lea ecx, [esp + 4]
	mov [eax + 16], ecx
	mov ecx, [esp]
	mov [eax + 20], ecx
	xor eax, eax
	ret
