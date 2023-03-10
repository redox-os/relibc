double strtod(const char *nptr, char **endptr);

long double strtold(const char *nptr, char **endptr) {
    return (long double)strtod(nptr, endptr);
}
