.global _longjmp
.global longjmp
.type _longjmp,@function
.type longjmp,@function
_longjmp:
longjmp:
	mov edx, [esp + 4]
	mov eax, [esp + 8]
	cmp eax, 1
	adc al, 0
	mov ebx, [edx]
	mov esi, [edx + 4]
	mov edi, [edx + 8]
	mov ebp, [edx + 12]
	mov esp, [edx + 16]
	jmp [edx + 20]
