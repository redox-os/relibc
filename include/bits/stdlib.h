#ifndef _BITS_STDLIB_H
#define _BITS_STDLIB_H

#ifdef __cplusplus
extern "C" {
#endif

static inline long double strtold(const char *nptr, char **endptr) {
  return (long double)strtod(nptr, endptr);
}

#ifdef __cplusplus
} // extern "C"
#endif

#endif /* _BITS_STDLIB_H */
