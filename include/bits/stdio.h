#ifndef _BITS_STDIO_H
#define _BITS_STDIO_H

int fprintf(FILE * stream, const char * fmt, ...) {
	int ret;
	va_list ap;
	va_start(ap, fmt);
	ret = vfprintf(stream, fmt, ap);
	va_end(ap);
	return ret;
}

int printf(const char * fmt, ...) {
	int ret;
	va_list ap;
	va_start(ap, fmt);
	ret = vprintf(fmt, ap);
	va_end(ap);
	return ret;
}

#endif /* _BITS_STDIO_H */
