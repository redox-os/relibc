#ifndef _STDINT_H
#define _STDINT_H

#define INT8_C(value) ((int8_t) value)
#define INT8_MIN -0x80
#define INT8_MAX 0x7F
typedef signed char int8_t;
typedef signed char int_least8_t;
typedef signed char int_fast8_t;

#define UINT8_C(value) ((uint8_t) value ## U)
#define UINT8_MIN 0x00
#define UINT8_MAX 0xFF
typedef unsigned char uint8_t;
typedef unsigned char uint_least8_t;
typedef unsigned char uint_fast8_t;

#define INT16_C(value) value
#define INT16_MIN -0x8000
#define INT16_MAX 0x7FFF
typedef signed short int16_t;
typedef signed short int_least16_t;
typedef signed short int_fast16_t;

#define UINT16_C(value) value ## U
#define UINT16_MIN 0x0000
#define UINT16_MAX 0xFFFF
typedef unsigned short uint16_t;
typedef unsigned short uint_least16_t;
typedef unsigned short uint_fast16_t;

#define INT32_C(value) value ## L
#define INT32_MIN -0x80000000
#define INT32_MAX 0x7FFFFFFF
typedef signed long int32_t;
typedef signed long int_least32_t;
typedef signed long int_fast32_t;

#define UINT32_C(value) value ## UL
#define UINT32_MIN 0x00000000
#define UINT32_MAX 0xFFFFFFFF
typedef unsigned long uint32_t;
typedef unsigned long uint_least32_t;
typedef unsigned long uint_fast32_t;

#define INT64_C(value) value ## LL
#define INT64_MIN -0x8000000000000000
#define INT64_MAX 0x7FFFFFFFFFFFFFFF
typedef signed long long int64_t;
typedef signed long long int_least64_t;
typedef signed long long int_fast64_t;

#define UINT64_C(value) value ## ULL
#define UINT64_MIN 0x0000000000000000
#define UINT64_MAX 0xFFFFFFFFFFFFFFFF
typedef unsigned long long uint64_t;
typedef unsigned long long uint_least64_t;
typedef unsigned long long uint_fast64_t;

#define INTMAX_C(value) value ## LL
#define INTMAX_MIN INT64_MIN
#define INTMAX_MAX INT64_MAX
typedef int64_t intmax_t;

#define UINTMAX_C(value) value ## ULL
#define UINTMAX_MIN UINT64_MIN
#define UINTMAX_MAX UINT64_MAX
typedef uint64_t uintmax_t;

#define INTPTR_MIN INT64_MIN
#define INTPTR_MAX INT64_MAX
typedef int64_t intptr_t;

#define UINTPTR_MIN UINT64_MIN
#define UINTPTR_MAX UINT64_MAX
typedef uint64_t uintptr_t;

#define SIZE_MAX UINT64_MAX

typedef int sig_atomic_t;

#endif /* _STDINT_H */
