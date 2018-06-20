#ifndef _BITS_STDIO_H
#define _BITS_STDIO_H

#define EOF (-1)

int fprintf(FILE * stream, const char * fmt, ...);
int printf(const char * fmt, ...);
int snprintf(char *s, size_t n, const char * fmt, ...);
int sprintf(char *s, const char * fmt, ...);

#endif /* _BITS_STDIO_H */
