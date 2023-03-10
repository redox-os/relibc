#ifndef _BITS_STDLIB_H
#define _BITS_STDLIB_H

# define abort() __abort(__func__, __FILE__, __LINE__)

#ifdef __cplusplus
extern "C" {
#endif

long double strtold(const char *nptr, char **endptr);

#ifdef __cplusplus
} // extern "C"
#endif

#endif /* _BITS_STDLIB_H */
