#ifndef _BITS_STDIO_H
#define _BITS_STDIO_H

int printf(const char *restrict fmt, ...) {
	int ret;
	va_list ap;
	va_start(ap, fmt);
	ret = vfprintf(stdout, fmt, ap);
	va_end(ap);
	return ret;
}

#endif /* _BITS_STDIO_H */
