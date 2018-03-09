#ifndef _BITS_FCNTL_H
#define _BITS_FCNTL_H

int open(const char* filename, int flags, ...) {
    mode_t mode = 0;
	va_list ap;
	va_start(ap, flags);
    mode = va_arg(ap, mode_t);
	va_end(ap);
    return sys_open(filename, flags, mode);
}

#endif
