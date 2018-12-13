#ifndef _BITS_WCHAR_H
#define _BITS_WCHAR_H
#include <stdint.h>

#define WEOF (0xffffffffu)
#define WCHAR_MIN (0)
#define WCHAR_MAX (0x7fffffff)

typedef int32_t wchar_t;
typedef uint32_t wint_t;

#endif /* _BITS_WCHAR_H */
