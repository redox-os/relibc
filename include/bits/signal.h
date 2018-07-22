#define SIG_ERR -1ULL

#define NSIG 64

// darn cbindgen
typedef unsigned long sigset_t[NSIG / (8 * sizeof(unsigned long))];
