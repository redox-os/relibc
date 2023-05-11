#ifndef _BITS_WCHAR_H
#define _BITS_WCHAR_H

// NULL, size_t
#define __need_size_t
#define __need_NULL
#include <stddef.h>

// int32_t, uint32_t, WCHAR_MIN, WCHAR_MAX
#include <stdint.h>

#define WEOF (0xffffffffu)

typedef int32_t wchar_t;
typedef uint32_t wint_t;

int wprintf(const wchar_t * fmt, ...);
int fwprintf(FILE * stream, const wchar_t * fmt, ...);
int swprintf(wchar_t *s, size_t n, const wchar_t * fmt, ...);

#endif /* _BITS_WCHAR_H */
