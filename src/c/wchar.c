#include <stdarg.h>
#include <stddef.h>

typedef struct FILE FILE;

int vwprintf(const wchar_t * fmt, va_list ap);

int wprintf(const wchar_t * fmt, ...) {
    int ret;
    va_list ap;
    va_start(ap, fmt);
    ret = vwprintf(fmt, ap);
    va_end(ap);
    return ret;
}

int vfwprintf(FILE * stream, const wchar_t * fmt, va_list ap);

int fwprintf(FILE * stream, const wchar_t * fmt, ...) {
    int ret;
    va_list ap;
    va_start(ap, fmt);
    ret = vfwprintf(stream, fmt, ap);
    va_end(ap);
    return ret;
}

int vswprintf(wchar_t * s, size_t n, const wchar_t * fmt, va_list ap);

int swprintf(wchar_t *s, size_t n, const wchar_t * fmt, ...) {
    int ret;
    va_list ap;
    va_start(ap, fmt);
    ret = vswprintf(s, n, fmt, ap);
    va_end(ap);
    return ret;
}
