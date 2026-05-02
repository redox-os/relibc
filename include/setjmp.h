#ifndef _SETJMP_H
#define _SETJMP_H

#ifdef __aarch64__
typedef unsigned long long jmp_buf[22];
#endif

#ifdef __i386__
typedef unsigned long long jmp_buf[6];
#endif

#ifdef __x86_64__
typedef unsigned long long jmp_buf[16];
#endif

#ifdef __riscv
typedef unsigned long long jmp_buf[26];
#endif

typedef jmp_buf sigjmp_buf;

#ifdef __cplusplus
extern "C" {
#endif

int setjmp(jmp_buf buf);
void longjmp(jmp_buf buf, int value);

#ifdef __cplusplus
} // extern "C"
#endif

#endif /* _SETJMP_H */
