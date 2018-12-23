#ifndef _SETJMP_H
#define _SETJMP_H

#ifdef __aarch64__
typedef unsigned long jmp_buf[22];
#endif

#ifdef __arm__
typedef unsigned long long jmp_buf[32];
#endif

#ifdef __i386__
typedef unsigned long jmp_buf[6];
#endif

#ifdef __m68k__
typedef unsigned long jmp_buf[39];
#endif

#ifdef __microblaze__
typedef unsigned long jmp_buf[18];
#endif

#ifdef __mips__
typedef unsigned long long jmp_buf[13];
#endif

#ifdef __mips64__
typedef unsigned long long jmp_buf[23];
#endif

#ifdef __mipsn32__
typedef unsigned long long jmp_buf[23];
#endif

#ifdef __or1k__
typedef unsigned long jmp_buf[13];
#endif

#ifdef __powerpc__
typedef unsigned long long jmp_buf[56];
#endif

#ifdef __powerpc64__
typedef uint128_t jmp_buf[32];
#endif

#ifdef __s390x__
typedef unsigned long jmp_buf[18];
#endif

#ifdef __sh__
typedef unsigned long jmp_buf[15];
#endif

#ifdef __x32__
typedef unsigned long long jmp_buf[8];
#endif

#ifdef __x86_64__
typedef unsigned long jmp_buf[8];
#endif

#ifdef __cplusplus
extern "C" {
#endif

int setjmp(jmp_buf buf);
void longjmp(jmp_buf buf, int value);

#ifdef __cplusplus
} // extern "C"
#endif

#endif /* _SETJMP_H */
