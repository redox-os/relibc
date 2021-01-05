#include <stdio.h>

int global_var = 42;
_Thread_local int tls_var = 21;

void print()
{
    fprintf(stdout, "sharedlib: global_var == %d\n", global_var);
    fprintf(stdout, "sharedlib: tls_var == %d\n", tls_var);
}
