#ifndef _STDDEF_H
#define _STDDEF_H
#include <stdint.h>
#include <bits/size-t.h>

#define NULL 0

#ifndef __PTRDIFF_TYPE__
#define __PTRDIFF_TYPE__ long int
#endif
typedef __PTRDIFF_TYPE__ ptrdiff_t;

typedef struct { long long __ll; long double __ld; } max_align_t;

#define offsetof(type, member) __builtin_offsetof(type, member)

#endif /* _STDDEF_H */
