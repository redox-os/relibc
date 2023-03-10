double strtod(const char *nptr, char **endptr);

long double strtold(const char *nptr, char **endptr) {
    return (long double)strtod(nptr, endptr);
}

// manually define detailed abort function
void __abort(const char *func, const char *file, int line);

// backup definition of abort for programs that link it directly
void abort(void) {
    // call detailed abort function
    __abort(__func__, __FILE__, __LINE__);
}
