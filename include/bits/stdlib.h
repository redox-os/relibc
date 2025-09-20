#ifndef _BITS_STDLIB_H
#define _BITS_STDLIB_H

#ifdef __cplusplus
extern "C" {
#endif

long double strtold(const char *nptr, char **endptr);

#ifdef __cplusplus
} // extern "C"
#endif

#if defined (_LARGEFILE64_SOURCE)
#define mkstemp64 mkstemp
#endif

#endif /* _BITS_STDLIB_H */
