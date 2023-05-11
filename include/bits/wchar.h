#ifndef _BITS_WCHAR_H
#define _BITS_WCHAR_H

#define WEOF (0xffffffffu)
#define WCHAR_MIN (0)
#define WCHAR_MAX (0x7fffffff)

#define __need_size_t
#define __need_wchar_t
#define __need_wint_t
#define __need_NULL

#include <stdint.h>

int wprintf(const wchar_t * fmt, ...);
int fwprintf(FILE * stream, const wchar_t * fmt, ...);
int swprintf(wchar_t *s, size_t n, const wchar_t * fmt, ...);

#endif /* _BITS_WCHAR_H */
