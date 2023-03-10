#ifndef _BITS_STDLIB_H
#define _BITS_STDLIB_H

#ifdef __cplusplus
// C++ needs abort to be a function, define backup function
void abort(void);
#else
// C uses detailed abort macro
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
