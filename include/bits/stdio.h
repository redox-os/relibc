#ifndef _BITS_STDIO_H
#define _BITS_STDIO_H

#define EOF (-1)
#define BUFSIZ 1024
#define stdin __stdin()
#define stdout __stdout()
#define stderr __stderr()

int fprintf(FILE * stream, const char * fmt, ...);
int printf(const char * fmt, ...);
int snprintf(char *s, size_t n, const char * fmt, ...);
int sprintf(char *s, const char * fmt, ...);
int fscanf(FILE * stream, const char * fmt, ...);
int scanf(const char * fmt, ...);
int sscanf(const char * input, const char * fmt, ...);

#endif /* _BITS_STDIO_H */
