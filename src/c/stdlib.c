#include <stdio.h>

double strtod(const char *nptr, char **endptr);

/* Basically the same as musl's implementation. */
char *gcvt(double value, int ndigit, char *buf) {
    sprintf(buf, "%.*g", ndigit, value);
    return buf;
}

long double strtold(const char *nptr, char **endptr) {
    return (long double)strtod(nptr, endptr);
}
