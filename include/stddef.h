#ifndef _STDDEF_H
#define _STDDEF_H
#include <stdint.h>

#define NULL 0

typedef signed long long ptrdiff_t;

typedef int32_t wchar_t;
typedef int32_t wint_t;

typedef unsigned long long size_t;

#define offsetof(type, member) __builtin_offsetof(type, member)

#endif /* _STDDEF_H */
