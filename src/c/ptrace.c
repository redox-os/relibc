// TODO: Can be implemented in rust when cbindgen supports "..." syntax

#include <stdarg.h>

int sys_ptrace(int request, va_list ap);

int ptrace(int request, ...) {
    va_list ap;
    va_start(ap, request);
    int ret = sys_ptrace(request, ap);
    va_end(ap);
    return ret;
}
