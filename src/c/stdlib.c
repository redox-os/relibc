double strtod(const char *nptr, char **endptr);

long double strtold(const char *nptr, char **endptr) {
    return (long double)strtod(nptr, endptr);
}

double relibc_ldtod(const long double* val) {
    return (double)(*val);
}

void relibc_dtold(double val, long double* out) {
    *out = (long double)val;
}
