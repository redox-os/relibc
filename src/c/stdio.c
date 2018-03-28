#include <stdarg.h>
#include <stddef.h>

typedef struct FILE FILE;

int vfprintf(FILE * stream, const char * fmt, va_list ap);

int fprintf(FILE * stream, const char * fmt, ...) {
    int ret;
    va_list ap;
    va_start(ap, fmt);
    ret = vfprintf(stream, fmt, ap);
    va_end(ap);
    return ret;
}

int vprintf(const char * fmt, va_list ap);

int printf(const char * fmt, ...) {
    int ret;
    va_list ap;
    va_start(ap, fmt);
    ret = vprintf(fmt, ap);
    va_end(ap);
    return ret;
}

int vsnprintf(char * s, size_t n, const char * fmt, va_list ap);

int snprintf(char * s, size_t n, const char * fmt, ...) {
    int ret;
    va_list ap;
    va_start(ap, fmt);
    ret = vsnprintf(s, n, fmt, ap);
    va_end(ap);
    return ret;
}

int vsprintf(char * s, const char * fmt, va_list ap);

int sprintf(char *s, const char * fmt, ...) {
    int ret;
    va_list ap;
    va_start(ap, fmt);
    ret = vsprintf(s, fmt, ap);
    va_end(ap);
    return ret;
}
