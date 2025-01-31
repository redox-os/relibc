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
	stmxcsr [rsp - 8]
	and eax, 0x3f
	or [rsp - 8], eax
	test [rsp - 8], ecx
	jz 1f
	not ecx
	and [rsp - 8], ecx
	ldmxcsr [rsp - 8]
1:
	xor eax, eax
	ret

.global feraiseexcept
.type feraiseexcept,@function
feraiseexcept:
	and edi, 0x3f
	stmxcsr [rsp - 8]
	or [rsp - 8], edi
	ldmxcsr [rsp - 8]
	xor eax, eax
	ret

.global __fesetround
.hidden __fesetround
.type __fesetround,@function
__fesetround:
	push rax
	xor eax, eax
	mov ecx, edi
	fnstcw [rsp]
	and byte ptr [rsp + 1], 0xf3
	or byte ptr [rsp + 1], ch
	fldcw [rsp]
	stmxcsr [rsp]
	shl ch, 3
	and byte ptr [rsp + 1], 0x9f
	or byte ptr [rsp + 1], ch
	ldmxcsr [rsp]
	pop rcx
	ret

.global fegetround
.type fegetround,@function
fegetround:
	push rax
	stmxcsr [rsp]
	pop rax
	shr eax, 3
	and eax, 0xc00
	ret

.global fegetenv
.type fegetenv,@function
fegetenv:
	xor eax, eax
	fnstenv [rdi]
	stmxcsr [rdi + 28]
	ret

.global fesetenv
.type fesetenv,@function
fesetenv:
	xor eax, eax
	inc rdi
	jz 1f
	fldenv [rdi - 1]
	ldmxcsr [rdi + 27]
	ret
1:
	push rax
	push rax
	push 0xffff
	push 0x37f
	fldenv [rsp]
	push 0x1f80
	ldmxcsr [rsp]
	add rsp, 40
	ret

.global fetestexcept
.type fetestexcept,@function
fetestexcept:
	and edi, 0x3f
	push rax
	stmxcsr [rsp]
	pop rsi
	fnstsw ax
	or eax, esi
	and eax, edi
	ret