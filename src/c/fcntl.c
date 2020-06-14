#include <stdarg.h>
#include <sys/types_internal.h>

// TODO: Can be implemented in rust when cbindgen supports "..." syntax

int sys_open(const char* filename, int flags, mode_t mode);

int open(const char* filename, int flags, ...) {
    mode_t mode = 0;
    va_list ap;
    va_start(ap, flags);
    mode = va_arg(ap, mode_t);
    va_end(ap);
    return sys_open(filename, flags, mode);
}

int sys_fcntl(int fildes, int cmd, int args);

int fcntl(int fildes, int cmd, ...) {
    int args = 0;
    va_list ap;
    va_start(ap, cmd);
    args = va_arg(ap, int);
    va_end(ap);
    return sys_fcntl(fildes, cmd, args);
}
