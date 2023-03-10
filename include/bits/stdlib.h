#ifndef _BITS_STDLIB_H
#define _BITS_STDLIB_H

// C++ needs abort to be a function, and cannot use this
#ifndef __cplusplus
// Override abort function with detailed abort in C
#define abort() __abort(__func__, __FILE__, __LINE__)
#endif

#ifdef __cplusplus
extern "C" {
#endif

long double strtold(const char *nptr, char **endptr);

#ifdef __cplusplus
} // extern "C"
#endif

#endif /* _BITS_STDLIB_H */
