.global feclearexcept
.type feclearexcept,@function
feclearexcept:
	# Maintain exceptions in the SSE MXCSR, clear x87 exceptions
	mov ecx, edi
	and ecx, 0x3f
	fnstsw ax
	test eax, ecx
	jz 1f
	fnclex
1:
	stmxcsr [esp - 8]
	and eax, 0x3f
	or [esp - 8], eax
	test [esp - 8], ecx
	jz 1f
	not ecx
	and [esp - 8], ecx
	ldmxcsr [esp - 8]
1:
	xor eax, eax
	ret

.global feraiseexcept
.type feraiseexcept,@function
feraiseexcept:
	and edi, 0x3f
	stmxcsr [esp - 8]
	or [esp - 8], edi
	ldmxcsr [esp - 8]
	xor eax, eax
	ret

.global __fesetround
.hidden __fesetround
.type __fesetround,@function
__fesetround:
	push rax
	xor eax, eax
	mov ecx, edi
	fnstcw [esp]
	and byte ptr [esp + 1], 0xf3
	or byte ptr [esp + 1], ch
	fldcw [esp]
	stmxcsr [esp]
	shl ch, 3
	and byte ptr [esp + 1], 0x9f
	or byte ptr [esp + 1], ch
	ldmxcsr [esp]
	pop rcx
	ret

.global fegetround
.type fegetround,@function
fegetround:
	push rax
	stmxcsr [esp]
	pop rax
	shr eax, 3
	and eax, 0xc00
	ret

.global fegetenv
.type fegetenv,@function
fegetenv:
	xor eax, eax
	fnstenv [edi]
	stmxcsr [edi + 28]
	ret

.global fesetenv
.type fesetenv,@function
fesetenv:
	xor eax, eax
	inc edi
	jz 1f
	fldenv [edi - 1]
	ldmxcsr [edi + 27]
	ret
1:
	push rax
	push rax
	push 0xffff
	push 0x37f
	fldenv [esp]
	push 0x1f80
	ldmxcsr [esp]
	add esp, 40
	ret

.global fetestexcept
.type fetestexcept,@function
fetestexcept:
	and edi, 0x3f
	push rax
	stmxcsr [esp]
	pop rsi
	fnstsw ax
	or eax, esi
	and eax, edi
	ret